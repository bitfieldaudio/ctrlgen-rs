use futures_lite::pin;
use std::convert::Infallible;
use std::task::Poll;
use tokio::sync::oneshot;

#[derive(Debug)]
enum Inner<T> {
    Pending(oneshot::Receiver<T>),
    Ready(T),
    Empty,
}

/// A promise represents a value that may not yet have been received.
///
/// It is used to poll the result of an async operation from a non-async context, for example
/// an immiediate-mode GUI thread, or (as was the original use case) an audio processing thread. In general,
/// it is useful anywhere where one may want to start an async operation from a non-async context,
/// and then check if the result is available on subsequent passes.
///
/// This struct uses [tokio::sync::oneshot] to receive the item.
/// As opposed to the [tokio::sync::oneshot::Receiver], the promise retains
/// the item after it has been received.
///
/// ## Example
/// ```rust,ignore
/// async fn long_async_task() -> i32 {
///     tokio::time::sleep(Duration::from_secs(5));
///     return 42;
/// }
///
/// let promise = Promise::spawn(long_async_task());
/// // In GUI loop:
/// if let Some(x) = promise.get() {
///   ui.label(format!("Result: {x}"))
/// }
/// ```
///
/// ## Differences to poll-promise
/// This is similar in idea and implementaiton to [poll-promise](https://lib.rs/crates/poll-promise),
/// with a few minor differences:
///
///  - This uses [tokio::sync::oneshot] instead of [std::sync::mpsc], which should be better optimized
///    for this usecase. However, this means it is only available with tokio, and furthermore it is only
///    implemented for use on the tokio runtime.
///
///  - This promise provides an `empty` state along with `pending` and `ready`, which can be useful to
///    represent promises that will never be resolved, for example if the sender is closed. This also
///    makes it possible to deserialize promises, as unresolved promises can be mapped to an empty promise.
///    It also allows implementing `take(&mut self) -> Option<T>`, which takes the value out of the promise,
///    and leaves an empty promise in its place.
///
///  - This promise implements [std::future::Future], and can thus be `.await`ed. This makes functions
///    that return a promise useful in async contexts as well.
///
///  - This promise implements [std::ops::FromResidual] for [Result] and [Option], allowing you to use the
///    `?` operator in functions that return [Result] or [Option].
#[must_use = "Promises should not be discarded"]
#[derive(Debug)]
pub struct Promise<T> {
    inner: std::cell::UnsafeCell<Inner<T>>,
}

/// The type used to send the result to a promise
pub type Sender<T> = oneshot::Sender<T>;

impl<T> Promise<T> {
    /// Construct a promise from the channel it will receive the value on
    pub fn new(rx: oneshot::Receiver<T>) -> Self {
        Self {
            inner: std::cell::UnsafeCell::new(Inner::Pending(rx)),
        }
    }

    /// Construct a promise containing a value
    pub fn ready(val: T) -> Self {
        Self {
            inner: std::cell::UnsafeCell::new(Inner::Ready(val)),
        }
    }

    /// An empty promise will always resolve to no value.
    pub fn empty() -> Self {
        Self {
            inner: std::cell::UnsafeCell::new(Inner::Empty),
        }
    }

    /// Create a promise and its corresponding sender
    pub fn channel() -> (Sender<T>, Self) {
        let (tx, rx) = oneshot::channel();
        (tx, Self::new(rx))
    }

    /// Spawn a future on the tokio runtime, returning a promise to its result
    pub fn spawn<Fut>(fut: Fut) -> Self
    where
        Fut: std::future::Future<Output = T> + Send + 'static,
        T: Send + 'static,
    {
        let (tx, res) = Promise::channel();
        tokio::spawn(async { tx.send(fut.await) });
        res
    }

    /// Check if a value is ready in the promise. If this is true,
    pub fn is_ready(&self) -> bool {
        self.check_rx();
        matches!(unsafe { &*self.inner.get() }, Inner::Ready(_))
    }

    /// Check if the promise is empty, i.e. will never resolve
    pub fn is_empty(&self) -> bool {
        self.check_rx();
        matches!(unsafe { &*self.inner.get() }, Inner::Empty)
    }

    pub fn get(&self) -> Option<&T> {
        self.check_rx();
        match unsafe { &*self.inner.get() } {
            Inner::Ready(x) => Some(x),
            Inner::Pending(_) => None,
            Inner::Empty => None,
        }
    }

    /// Take the value out of the promise, leaving an empty promise in its place
    pub fn take(&mut self) -> Option<T> {
        self.check_rx();
        if self.is_ready() {
            if let Inner::Ready(x) =
                std::mem::replace(unsafe { &mut *self.inner.get() }, Inner::Empty)
            {
                return Some(x);
            }
        }
        None
    }

    fn check_rx(&self) {
        if let Inner::Pending(rx) = unsafe { &mut *self.inner.get() } {
            match rx.try_recv() {
                Ok(x) => unsafe { *self.inner.get() = Inner::Ready(x) },
                Err(oneshot::error::TryRecvError::Empty) => (),
                Err(oneshot::error::TryRecvError::Closed) => unsafe {
                    *self.inner.get() = Inner::Empty
                },
            }
        }
    }

    /// Map a promise through a function.
    ///
    /// Only use this when a promise type is needed as result. Otherwise, use [FutureExt::map] directly
    /// as it is more performant.
    pub fn map_promise<U, F>(self, f: F) -> Promise<U>
    where
        T: Send + 'static,
        F: FnOnce(T) -> U + Send + 'static,
        U: Send + 'static,
    {
        let (tx, res) = Promise::channel();
        tokio::spawn(async {
            if let Some(x) = self.await {
                let _ = tx.send(f(x));
            }
        });
        res
    }

    /// Only use this when a promise type is needed as result. Otherwise, use [FutureExt::then] directly
    /// as it is more performant.
    pub fn then_promise<Fut: std::future::Future, F>(self, f: F) -> Promise<Fut::Output>
    where
        T: Send + 'static,
        F: FnOnce(T) -> Fut + Send + 'static,
        Fut: Send,
        Fut::Output: Send + 'static,
    {
        let (tx, res) = Promise::channel();
        tokio::spawn(async {
            if let Some(x) = self.await {
                let _ = tx.send(f(x).await);
            }
        });
        res
    }

    /// Block the current thread, waiting for the promise to be resolved
    pub fn block_on(self) -> Option<T> {
        futures_lite::future::block_on(self)
    }
}

// Make Promise<Result> work with the ? operator
impl<T, E, F: From<E>> std::ops::FromResidual<Result<Infallible, E>> for Promise<Result<T, F>> {
    fn from_residual(residual: Result<Infallible, E>) -> Self {
        Self::ready(Err(F::from(unsafe { residual.unwrap_err_unchecked() })))
    }
}

// Make Promise<Option<T>> work with the ? operator
impl<T> std::ops::FromResidual<Option<Infallible>> for Promise<Option<T>> {
    fn from_residual(_: Option<Infallible>) -> Self {
        Self::ready(None)
    }
}

// Make a promise awaitable
impl<T> std::future::Future for Promise<T> {
    type Output = Option<T>;

    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        match unsafe { &mut *self.inner.get() } {
            Inner::Ready(x) => Poll::Ready(Some(unsafe { std::ptr::read(x as *mut T) })),
            Inner::Empty => Poll::Ready(None),
            Inner::Pending(rx) => {
                pin!(rx);
                rx.poll(cx).map(|x| x.ok())
            }
        }
    }
}

#[cfg(feature = "serde")]
impl<T> serde::Serialize for Promise<T>
where
    T: serde::Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.get().serialize(serializer)
    }
}

#[cfg(feature = "serde")]
impl<'de, T> serde::Deserialize<'de> for Promise<T>
where
    T: serde::Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        match Option::<T>::deserialize(deserializer)? {
            Some(x) => Ok(Promise::ready(x)),
            None => Ok(Promise::empty()),
        }
    }
}
