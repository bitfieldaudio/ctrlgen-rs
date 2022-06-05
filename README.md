# ctrlgen-rs

A fork of [vi/trait-enumizer](https://github.com/vi/trait-enumizer) that attempts to be easier to use,
while sacrificing a bit of flexibility.

Documentation to come. See docs of trait-enumizer, and examples here for now.

## Differences to trait-enumizer:
 - ctrlgen only supports inherent impls, it will not enumize traits.
 - ctrlgen prefers generating impls of traits over inherent functions, to
   make it more transparent to the user what is done. For example, the call function
   is implemented by implementing the `CallMut` trait on the enum.
 - ctrlgen tries to minimize and simplify the argument syntax, at the cost of some configurability.
   For example, the call trait will always be implemented.
 - ctrlgen requires the nightly rust channel for (amongst other things) GATs
 - ctrlgen supports generics in the struct definition (but not in the method signatures)
 - Proxies are implemented slightly differently, and are generally simpler. However, they currently
   don't support a lot of the options trait-enumizer has.
 - Proxies with async senders are currently not implemented
 - Async return value senders are currently not implemented.
 - no_std support is untested/unimplemented, but will be easy to do.

Most of the efforts here could probably be merged into trait-enumizer, but i was in a hurry,
so i implemented the features i needed instead.

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
