# ctrlgen-rs

A fork of [vi/trait-enumizer](https://github.com/vi/trait-enumizer) that attempts to be easier to use,
while sacrificing a bit of flexibility.

Documentation to come. See docs of trait-enumizer, and examples here for now.

## Example

```rust
#[derive(Default)]
struct Service {
    counter: i32,
    flag: bool,
}

#[ctrlgen::ctrlgen(pub ServiceMsg,
    returnval = LocalRetval,
    proxy = ServiceProxy
)]
impl Service {
    pub fn increment_by(&mut self, arg: i32) -> i32 {
        self.counter += arg;
        self.counter
    }

    pub fn set_flag(&mut self, flag: bool) {
        self.flag = flag;
    }
}

fn test() {
    let service = RefCell::new(Service {
        counter: 0,
        flag: false,
    });
    
    let msg = ServiceMsg::SetFlag { flag: true };
    msg.call_mut(&mut *service.borrow_mut());

    // With proxy:
    let proxy = ServiceProxy::new(|msg: ServiceMsg| {
        msg.call_mut(&mut *service.borrow_mut());
    });

    let ret = proxy.increment_by(2);
    assert_eq!(*ret.borrow(), Some(2));
    assert_eq!(service.borrow().counter, 2);
}
```

## Returnval

By setting the `returnval = <Trait>` parameter, you configure the channel over which return values are sent.
`<Trait>` must implement `ctrlgen::Returnval`. 

Example Implementation:

```rust
pub struct FailedToSendRetVal;
pub struct TokioRetval;
impl Returnval for TokioRetval {
    type Sender<T> = promise::Sender<T>;
    type Receiver<T> = promise::Promise<T>;
    type SendError = FailedToSendRetVal;

    type RecvResult<T> = promise::Promise<T>;

    fn create<T>() -> (Self::Sender<T>, Self::Receiver<T>) {
        promise::Promise::channel()
    }

    fn recv<T>(rx: Self::Receiver<T>) -> Self::RecvResult<T> {
        rx
    }

    fn send<T>(tx: Self::Sender<T>, msg: T) -> core::result::Result<(), Self::SendError> {
        tx.send(msg).map_err(|_| FailedToSendRetVal)
    }
}
```
