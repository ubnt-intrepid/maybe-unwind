use super::{maybe_unwind, Unwind};
use futures_core::{
    future::Future,
    task::{self, Poll},
};
use pin_project::pin_project;
use std::{
    panic::{AssertUnwindSafe, UnwindSafe},
    pin::Pin,
};

/// A future for the [`maybe_unwind`] method.
///
/// [`maybe_unwind`]: ./trait.FutureMaybeUnwindExt.html#method.maybe_unwind
#[pin_project]
#[derive(Debug)]
#[must_use = "futures do nothing unless you `.await` or poll them"]
pub struct MaybeUnwind<T> {
    #[pin]
    inner: T,
}

impl<F> Future for MaybeUnwind<F>
where
    F: Future + UnwindSafe,
{
    type Output = Result<F::Output, Unwind>;

    fn poll(self: Pin<&mut Self>, cx: &mut task::Context<'_>) -> Poll<Self::Output> {
        let me = self.project();
        maybe_unwind(AssertUnwindSafe(|| me.inner.poll(cx)))?.map(Ok)
    }
}

/// An extension trait for `Future`s that provides an adaptor for capturing
/// the unwinding panic information.
pub trait FutureMaybeUnwindExt: Future + Sized {
    /// Catches unwinding panics while polling the future.
    ///
    /// This is a variant of [`catch_unwind`] that also captures
    /// the panic information.
    ///
    /// # Example
    ///
    /// ```
    /// # use maybe_unwind::maybe_unwind;
    /// use maybe_unwind::FutureMaybeUnwindExt as _;
    ///
    /// maybe_unwind::set_hook();
    ///
    /// # futures_executor::block_on(async {
    /// let res = do_something_async().maybe_unwind().await;
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
