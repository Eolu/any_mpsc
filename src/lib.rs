#[cfg(feature = "buf_recv")]
mod buffered_receiver;
#[cfg(feature = "buf_recv")]
pub use buffered_receiver::*;

use core::any::*;
use std::sync::mpsc::{self, Sender, Receiver};

/// An [mpsc::channel] that supports dynamic typing.
pub fn channel() -> (AnySender, AnyReceiver)
{
    let (tx, rx) = mpsc::channel();
    (AnySender(tx), AnyReceiver(rx))
}

/// Wraps an [mpsc::Sender] to support dynamic typing.
#[derive(Debug)]
pub struct AnySender(pub Sender<Box<dyn Any>>);

impl AnySender
{
    /// Wraps [mpsc::Sender::send].
    pub fn send<T: Any>(&self, t: T) -> Result<(), mpsc::SendError<Box<dyn Any>>>
    {
        self.0.send(Box::new(t))
    }
}

/// Wraps an [mpsc::Receiver] to support dynamic typing.
#[derive(Debug)]
pub struct AnyReceiver(pub Receiver<Box<dyn Any>>);

impl AnyReceiver
{
    /// Wraps [mpsc::Receiver::recv]. See [crate::AnyRecvError] for details on the 
    /// return value.
    pub fn recv<T: 'static>(&self) -> Result<T, AnyRecvError>
    {
        self.0
            .recv()
            .map_err(AnyRecvError::RecvError)
            .and_then(|r| match r.downcast()
            {
                Ok(r) => Ok(*r),
                Err(r) => Err(AnyRecvError::WrongType(r)),
            })
    }

    /// Wraps [mpsc::Receiver::recv_timeout]. See [crate::AnyRecvError] for 
    /// details on the return value.
    pub fn recv_timeout<T: 'static>(&self, timeout: std::time::Duration) -> Result<T, AnyRecvError>
    {
        self.0
            .recv_timeout(timeout)
            .map_err(AnyRecvError::RecvTimeoutError)
            .and_then(|r| match r.downcast()
            {
                Ok(r) => Ok(*r),
                Err(r) => Err(AnyRecvError::WrongType(r)),
            })
    }

    /// Wraps [mpsc::Receiver::try_recv]. See [crate::AnyRecvError] for 
    /// details on the return value.
    pub fn try_recv<T: 'static>(&self) -> Result<T, AnyRecvError>
    {
        self.0
            .try_recv()
            .map_err(AnyRecvError::TryRecvError)
            .and_then(|r| match r.downcast()
            {
                Ok(r) => Ok(*r),
                Err(r) => Err(AnyRecvError::WrongType(r)),
            })
    }
}

/// Error type for receievers. If an [mpsc] error occurs, it will be wrapped
/// by an appropriate wrapper variant. If receiver is supplied an incorrect type, 
/// a [AnyRecvError::WrongType(Box<dyn Any>)] will be returned containing the result 
/// that did not successfully downcast.
#[derive(Debug)]
pub enum AnyRecvError
{
    RecvError(mpsc::RecvError),
    RecvTimeoutError(mpsc::RecvTimeoutError),
    TryRecvError(mpsc::TryRecvError),
    WrongType(Box<dyn Any>)
}


#[cfg(test)]
mod tests 
{
    use crate::*;

    #[test]
    pub fn any_channel_test()
    {
        let (atx, arx) = channel();
        atx.send(67 as i32).unwrap();
        println!("{:?}", arx.recv::<u32>());
    }
    
    #[test]
    pub fn any_channel_test_1()
    {
        let (atx, arx) = channel();
        atx.send(88 as i32).unwrap();
        println!("{:?}", arx.recv::<i32>());
    }
    
    #[test]
    pub fn any_channel_test_2()
    {
        let to_send: f32 = 55.7;
        let (atx, arx) = channel();
        atx.send(to_send).unwrap();
        println!("{:?}", arx.recv::<f32>());
    }
}
