use dfb::*;

use super::{AnySender, AnyRecvError};
use core::any::*;
use std::sync::mpsc::{self, Receiver};

/// An [mpsc::channel] that supports dynamic typing and contains a buffer to 
/// prevent the need for dynamic types to be exposed. 
#[inline]
pub fn buffered_channel() -> (AnySender, BufferedReceiver)
{
    let (tx, rx) = mpsc::channel();
    (AnySender(tx), BufferedReceiver { rx, buf: Dfb::new() })
}

/// Wraps an [mpsc::Receiver] to support dynamic typing and buffered results.
#[derive(Debug)]
pub struct BufferedReceiver
{
    pub rx: Receiver<Box<dyn Any>>,
    pub buf: Dfb
}
unsafe impl Send for BufferedReceiver {}

impl BufferedReceiver
{
    /// Wraps [mpsc::Receiver::recv]. See [BufRecvError] for details on the 
    /// return value. Will continuously recv until the supplied type is found. 
    /// Any unmatching types received will be placed in the buffer.
    pub fn recv_until<T: 'static>(&mut self) -> Result<T, AnyRecvError>
    {
        loop
        {
            match self.recv::<T>()
            {
                Err(err) => match err
                {
                    AnyRecvError::BufRecvError(_type_id) => continue,
                    err => break Err(err),
                },
                ok => break ok
            }
        }
    }

    /// Wraps [mpsc::Receiver::recv]. See [BufRecvError] for details on the 
    /// return value. Will attempt to take from the internal buffer before
    /// performing an actual channel recv.
    #[inline]
    pub fn recv<T: 'static>(&mut self) -> Result<T, AnyRecvError>
    {
        match self.buf.remove::<T>()
        {
            Some(t) => Ok(t),
            None => self.rx
                .recv()
                .map_err(AnyRecvError::RecvError)
                .and_then(|r| match r.downcast()
                {
                    Ok(r) => Ok(*r),
                    Err(r) => 
                    {
                        let err = Err(AnyRecvError::BufRecvError(r.as_ref().type_id()));
                        self.buf.insert_dyn(r);
                        err
                    },
                })
        }
    }

    /// Wraps [mpsc::Receiver::recv]. See [BufRecvError] for details on the 
    /// return value. Will perform a channel recv regardless of whether or not
    /// anything is contained in the buffer.
    #[inline]
    pub fn recv_live<T: 'static>(&mut self) -> Result<T, AnyRecvError>
    {
        self.rx
            .recv()
            .map_err(AnyRecvError::RecvError)
            .and_then(|r| match r.downcast()
            {
                Ok(r) => Ok(*r),
                Err(r) => 
                {
                    let err = Err(AnyRecvError::BufRecvError(r.as_ref().type_id()));
                    self.buf.insert_dyn(r);
                    err
                },
            })
    }

    /// Wraps [mpsc::Receiver::recv_timeout]. See [BufRecvError] for 
    /// details on the return value. Will attempt to take from the internal 
    /// buffer before performing an actual channel recv_timeout.
    #[inline]
    pub fn recv_timeout<T: 'static>(&mut self, timeout: std::time::Duration) -> Result<T, AnyRecvError>
    {
        match self.buf.remove::<T>()
        {
            Some(t) => Ok(t),
            None => self.rx
                .recv_timeout(timeout)
                .map_err(AnyRecvError::RecvTimeoutError)
                .and_then(|r| match r.downcast()
                {
                    Ok(r) => Ok(*r),
                    Err(r) => 
                    {
                        let err = Err(AnyRecvError::BufRecvError(r.as_ref().type_id()));
                        self.buf.insert_dyn(r);
                        err
                    }
                })
        }
    }

    /// Wraps [mpsc::Receiver::recv_timeout]. See [BufRecvError] for 
    /// details on the return value. Will perform a channel recv_timeout 
    /// regardless of whether or not anything is contained in the buffer.
    #[inline]
    pub fn recv_timeout_live<T: 'static>(&mut self, timeout: std::time::Duration) -> Result<T, AnyRecvError>
    {
        match self.buf.remove::<T>()
        {
            Some(t) => Ok(t),
            None => self.rx
                .recv_timeout(timeout)
                .map_err(AnyRecvError::RecvTimeoutError)
                .and_then(|r| match r.downcast()
                {
                    Ok(r) => Ok(*r),
                    Err(r) => 
                    {
                        let err = Err(AnyRecvError::BufRecvError(r.as_ref().type_id()));
                        self.buf.insert_dyn(r);
                        err
                    }
                })
        }
    }

    /// Wraps [mpsc::Receiver::try_recv]. See [BufRecvError] for 
    /// details on the return value. Will attempt to take from the internal 
    /// buffer before performing an actual channel recv_timeout.
    #[inline]
    pub fn try_recv<T: 'static>(&mut self) -> Result<T, AnyRecvError>
    {
        match self.buf.remove::<T>()
        {
            Some(t) => Ok(t),
            None => self.rx
                .try_recv()
                .map_err(AnyRecvError::TryRecvError)
                .and_then(|r| match r.downcast()
                {
                    Ok(r) => Ok(*r),
                    Err(r) => 
                    {
                        let err = Err(AnyRecvError::BufRecvError(r.as_ref().type_id()));
                        self.buf.insert_dyn(r);
                        err
                    }
                })
        }
    }

    /// Wraps [mpsc::Receiver::try_recv]. See [BufRecvError] for 
    /// details on the return value. Will perform a channel recv_timeout 
    /// regardless of whether or not anything is contained in the buffer.
    #[inline]
    pub fn try_recv_live<T: 'static>(&mut self) -> Result<T, AnyRecvError>
    {
        match self.buf.remove::<T>()
        {
            Some(t) => Ok(t),
            None => self.rx
                .try_recv()
                .map_err(AnyRecvError::TryRecvError)
                .and_then(|r| match r.downcast()
                {
                    Ok(r) => Ok(*r),
                    Err(r) => 
                    {
                        let err = Err(AnyRecvError::BufRecvError(r.as_ref().type_id()));
                        self.buf.insert_dyn(r);
                        err
                    }
                })
        }
    }

    /// Wraps [mpsc::Receiver::recv]. See [crate::AnyRecvError] for details on the 
    /// return value. Bypasses the buffer entirely.
    #[inline]
    pub fn recv_nobuf<T: 'static>(&self) -> Result<T, AnyRecvError>
    {
        self.rx
            .recv()
            .map_err(AnyRecvError::RecvError)
            .and_then(|r| match r.downcast()
            {
                Ok(r) => Ok(*r),
                Err(r) => Err(AnyRecvError::WrongType(r)),
            })
    }

    /// Wraps [mpsc::Receiver::recv_timeout]. See [crate::AnyRecvError] for 
    /// details on the return value. Bypasses the buffer entirely.
    #[inline]
    pub fn recv_timeout_nobuf<T: 'static>(&self, timeout: std::time::Duration) -> Result<T, AnyRecvError>
    {
        self.rx
            .recv_timeout(timeout)
            .map_err(AnyRecvError::RecvTimeoutError)
            .and_then(|r| match r.downcast()
            {
                Ok(r) => Ok(*r),
                Err(r) => Err(AnyRecvError::WrongType(r)),
            })
    }

    /// Wraps [mpsc::Receiver::try_recv]. See [crate::AnyRecvError] for 
    /// details on the return value. Bypasses the buffer entirely.
    #[inline]
    pub fn try_recv_nobuf<T: 'static>(&self) -> Result<T, AnyRecvError>
    {
        self.rx
            .try_recv()
            .map_err(AnyRecvError::TryRecvError)
            .and_then(|r| match r.downcast()
            {
                Ok(r) => Ok(*r),
                Err(r) => Err(AnyRecvError::WrongType(r)),
            })
    }

    /// Will attempt to read a value from the internal buffer. Will not do a
    /// channel recv of any kind even if the buffer is empty.
    #[inline]
    pub fn recv_buf<T: 'static>(&mut self) -> Result<T, AnyRecvError>
    {
        match self.buf.remove::<T>()
        {
            Some(t) => Ok(t),
            None => Err(AnyRecvError::EmptyBuffer)
        }
    }
}

