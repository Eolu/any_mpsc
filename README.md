# any-mpsc
A wrapper for an `mpsc::channel` that allows arbitrary types to be passed through. Comes in 2 different flavors.

### any_mpsc::channel
The `any_mpsc::channel` function may be used to create a basic `(AnySender, AnyReceiver)`. `AnySender` takes any value and sends it through the underlying channel with the `send` method (internally as a `Box<dyn Any>`). `AnyReceiver` contains generic versions of `recv`, `recv_timeout`, and `try_recv`. If the generic parameter supplied doesn't correspond with the type the `AnySender` pushed through, a `AnyRecvError::WrongType(Box<dyn Any>)` will be returned containing the value.

### any_mpsc::buffered_channel
Probably more useful, the `buf_recv` default feature enables the `any_mpsc::buffered_channel` function. This will return a `(AnySender, BufferedReceiver)`. The `BufferedReceiver` works differently from the `AnyReceiver` in that if an unmatching generic type is supplied, it will instead return a `AnyRecvError::BufRecvError(TypeId)`. The actual value will be stored in its internal buffer, and the next time `recv`, `recv_timeout`, or `try_recv` is called with a generic parameter matching its type, that buffered value will be returned and removed from the buffer. Additional methods for interaction with the channel and buffer exist, see the table at the bottom of this section.

Example:

```rust

fn receive_handler<T: Debug + 'static>(rx: &mut BufferedReceiver)
{
    match rx.recv::<T>()
    {
        Ok(result) => println!("{:?}", result),
        Err(AnyRecvError::BufRecvError(type_id)) => println!("Type with id {:?} added to buffer", type_id),
        Err(e) => eprintln!("{}", e)
    }
}

let (tx, mut rx) = any_mpsc::buffered_channel();

tx.send(55.7f32)?;
tx.send(String::from("example"))?;

receive_handler::<f32>(&mut rx);
receive_handler::<f32>(&mut rx);
receive_handler::<String>(&mut rx);

```
The output is this:
```
55.7
Type with id TypeId { t: 7854412396444247302 } added to buffer
"example"
```
|Method|Description|
|-|-|
|`recv_until`|Will continuously recv until the supplied generic type is found. Any unmatching types received are placed in the buffer.|
|`recv`|Attempts to pop from internal buffer. If buffer is empty, calls mpsc recv|
|`recv_timeout`|Attempts to pop from internal buffer. If buffer is empty, calls mpsc recv_timeout|
|`try_recv`|Attempts to pop from internal buffer. If buffer is empty, calls mpsc try_recv|
|`recv_live`|Calls mpsc recv regardless of whether or not the buffer is empty. Unmatching result types will still be placed in the buffer.|
|`recv_timeout_live`|Calls mpsc recv_timeout regardless of whether or not the buffer is empty. Unmatching result types will still be placed in the buffer.|
|`try_recv_live`|Calls mpsc try_recv regardless of whether or not the buffer is empty. Unmatching result types will still be placed in the buffer.|
|`recv_nobuf`|Equivalent to `AnyReceiver::recv` (bypasses the buffer entirely)|
|`recv_timeout_nobuf`|Equivalent to `AnyReceiver::recv_timeout` (bypasses the buffer entirely)|
|`try_recv_nobuf`|Equivalent to `AnyReceiver::try_recv` (bypasses the buffer entirely)|
|`recv_buf`|Attempts to pop from the internal buffer. Never attempts to access the internal channel at all.|

