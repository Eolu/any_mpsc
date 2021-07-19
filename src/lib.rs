#[cfg(feature = "buf_recv")]
mod buffered_receiver;
#[cfg(feature = "buf_recv")]
pub use buffered_receiver::*;

use core::any::*;
use std::{error::Error, fmt::Display, sync::mpsc::{self, Sender, Receiver}};

/// An [mpsc::channel] that supports dynamic typing.
#[inline]
pub fn channel() -> (AnySender, AnyReceiver)
{
    let (tx, rx) = mpsc::channel();
    (AnySender(tx), AnyReceiver(rx))
}

/// Wraps an [mpsc::Sender] to support dynamic typing.
#[derive(Debug)]
pub struct AnySender(pub Sender<Box<dyn Any>>);
unsafe impl Send for AnySender {}

impl AnySender
{
    /// Wraps [mpsc::Sender::send].
    #[inline]
    pub fn send<T: Any>(&self, t: T) -> Result<(), mpsc::SendError<Box<dyn Any>>>
    {
        self.0.send(Box::new(t))
    }
}

/// Wraps an [mpsc::Receiver] to support dynamic typing.
#[derive(Debug)]
pub struct AnyReceiver(pub Receiver<Box<dyn Any>>);
unsafe impl Send for AnyReceiver {}

impl AnyReceiver
{
    /// Wraps [mpsc::Receiver::recv]. See [crate::AnyRecvError] for details on the 
    /// return value.
    #[inline]
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
    #[inline]
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
    #[inline]
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
/// that did not successfully downcast. If buffered receiver is supplied an 
/// incorrect type, a [BufRecvError::WrongType(TypeId)] will be returned and the 
/// result will be stored in a buffer. If [BufferedReceiver::recv_buf] is called
/// with an empty buffer, EmptyBuffer will be returned
#[derive(Debug)]
pub enum AnyRecvError
{
    RecvError(mpsc::RecvError),
    RecvTimeoutError(mpsc::RecvTimeoutError),
    TryRecvError(mpsc::TryRecvError),
    WrongType(Box<dyn Any>),
    #[cfg(feature = "buf_recv")]
    BufRecvError(TypeId),
    #[cfg(feature = "buf_recv")]
    EmptyBuffer
}

impl Display for AnyRecvError
{
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result 
    {
        match self
        {
            AnyRecvError::RecvError(err) => err.fmt(f), 
            AnyRecvError::RecvTimeoutError(err) => err.fmt(f),
            AnyRecvError::TryRecvError(err) => err.fmt(f),
            AnyRecvError::WrongType(_) => write!(f, "Received wrong type"),
            AnyRecvError::BufRecvError(type_id) => write!(f, "Received wrong type: {:?}", type_id),
            AnyRecvError::EmptyBuffer => write!(f, "Buffer is empty"),
        }
    }
}

impl Error for AnyRecvError 
{
    #[inline]
    fn source(&self) -> Option<&(dyn Error + 'static)>
    {
        match self
        {
            AnyRecvError::RecvError(err) => Some(err),
            AnyRecvError::RecvTimeoutError(err) => Some(err),
            AnyRecvError::TryRecvError(err) => Some(err),
            _ => None
        }
    }
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
        let to_send = 55.7f32;
        let (atx, arx) = channel();
        atx.send(to_send).unwrap();
        println!("{:?}", arx.recv::<f32>());
    }

    #[test]
    #[cfg(feature = "buf_recv")]
    pub fn readme_test()
    {
        fn receive_handler<T: std::fmt::Debug + 'static>(rx: &mut BufferedReceiver)
        {
            match rx.recv::<T>()
            {
                Ok(result) => println!("{:?}", result),
                Err(AnyRecvError::BufRecvError(type_id)) => println!("Type with id {:?} added to buffer", type_id),
                Err(e) => eprintln!("{:?}", e)
            }
        }
        
        let (tx, mut rx) = crate::buffered_channel();
        
        tx.send(55.7f32).unwrap();
        tx.send(String::from("example")).unwrap();
        
        receive_handler::<f32>(&mut rx);
        receive_handler::<f32>(&mut rx);
        receive_handler::<String>(&mut rx);
    }

    #[test]
    #[cfg(feature = "buf_recv")]
    pub fn recv_until_test()
    {
        use std::thread;
        fn receive_handler<T: std::fmt::Debug + 'static>(rx: &mut BufferedReceiver)
        {
            match rx.recv_until::<T>()
            {
                Ok(result) => println!("{:?}", result),
                Err(e) => eprintln!("{:?}", e)
            }
        }
        
        let (tx, mut rx) = crate::buffered_channel();

        thread::spawn(move || 
        {
            receive_handler::<String>(&mut rx);
            receive_handler::<f32>(&mut rx);
        });
        
        tx.send(55.7f32).unwrap();
        tx.send(String::from("example")).unwrap();
    
    }
}
