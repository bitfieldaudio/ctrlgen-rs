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
struct Service<T: From<i32>> {
    counter: T,
}

#[ctrlgen::ctrlgen(pub ServiceMsg,
    returnval = TokioRetval,
    proxy = ServiceProxy
)]
impl Service {
    pub fn increment_by(&mut self, arg: i32) -> i32 {
        self.counter = arg;
        self.counter
    }
}
```

This will generate the following code:

```rust
pub enum ServiceMsg
where TokioRetval: ::ctrlgen::Returnval,
{
    IncrementBy {
        arg: i32,
        ret: <TokioRetval as ::ctrlgen::Returnval>::Sender<i32>,
    },
}

impl ::ctrlgen::CallMut<Service> for ServiceMsg
where TokioRetval: ::ctrlgen::Returnval,
{
    type Output = core::result::Result<(), <TokioRetval as ::ctrlgen::Returnval>::SendError>;
    fn call_mut(self, this: &mut Service) -> Self::Output {
        match self {
            Self::IncrementBy { arg, ret } => {
                <TokioRetval as ::ctrlgen::Returnval>::send(ret, this.increment_by(arg))
            }
        }
    }
}

pub struct ServiceProxy<Sender: ::ctrlgen::MessageSender<ServiceMsg>> {
    sender: Sender,
}

impl<Sender: ::ctrlgen::MessageSender<ServiceMsg>> ServiceProxy<Sender>
where TokioRetval: ::ctrlgen::Returnval,
{
    pub fn new(sender: Sender) -> Self {
        Self { sender }
    }
    pub fn increment_by(&self, arg: i32) -> <TokioRetval as ::ctrlgen::Returnval>::RecvResult<i32> {
        let ret = <TokioRetval as ::ctrlgen::Returnval>::create();
        let msg = ServiceMsg::IncrementBy { arg, ret: ret.0 };
        self.sender.send(msg);
        <TokioRetval as ::ctrlgen::Returnval>::recv(ret.1)
    }
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
