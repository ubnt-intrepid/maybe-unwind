use crate::unwind::{maybe_unwind, Unwind};
use futures_core::{
    future::Future,
    task::{self, Poll},
};
use std::{
    panic::{AssertUnwindSafe, UnwindSafe},
    pin::Pin,
};

/// A future for the [`maybe_unwind`] method.
///
/// [`maybe_unwind`]: ./trait.FutureMaybeUnwindExt.html#method.maybe_unwind
#[derive(Debug)]
#[cfg_attr(docs, doc(cfg(feature = "futures")))]
#[must_use = "futures do nothing unless you `.await` or poll them"]
pub struct MaybeUnwind<F> {
    inner: F,
}

impl<F> Future for MaybeUnwind<F>
where
    F: Future + UnwindSafe,
{
    type Output = Result<F::Output, Unwind>;

    fn poll(self: Pin<&mut Self>, cx: &mut task::Context<'_>) -> Poll<Self::Output> {
        let inner = unsafe { self.map_unchecked_mut(|me| &mut me.inner) };
        maybe_unwind(AssertUnwindSafe(|| inner.poll(cx)))?.map(Ok)
    }
}

/// An extension trait for `Future`s that provides an adaptor for capturing
/// the unwinding panic information.
#[cfg_attr(docs, doc(cfg(feature = "futures")))]
pub trait FutureMaybeUnwindExt: Future + Sized {
    /// Catches unwinding panics while polling the future.
    ///
    /// This is a variant of [`catch_unwind`] that also captures
    /// the panic information.
    ///
    /// # Example
    ///
    /// ```
    /// use maybe_unwind::FutureMaybeUnwindExt as _;
    ///
    /// std::panic::set_hook(Box::new(|info| {
    ///     maybe_unwind::capture_panic_info(info);
    /// }));
    ///
    /// # futures_executor::block_on(async {
    /// let res = do_something_async().maybe_unwind().await;
    /// # drop(res);
    /// # });
    /// # async fn do_something_async() {}
    /// ```
    ///
    /// [`catch_unwind`]: https://docs.rs/futures/0.3/futures/future/trait.FutureExt.html#method.catch_unwind
    fn maybe_unwind(self) -> MaybeUnwind<Self>
    where
        Self: UnwindSafe,
    {
        MaybeUnwind { inner: self }
    }
}

impl<F: Future> FutureMaybeUnwindExt for F {}
