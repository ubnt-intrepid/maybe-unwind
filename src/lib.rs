use futures::{
    future::Future,
    task::{self, Poll},
};
use pin_project_lite::pin_project;
use scoped_tls::scoped_thread_local;
use std::{
    any::Any,
    cell::RefCell,
    panic::{self, AssertUnwindSafe, PanicInfo, UnwindSafe},
    pin::Pin,
};

scoped_thread_local!(static ERROR_MSG: RefCell<Option<String>>);

#[inline]
pub fn set_hook() -> Box<dyn Fn(&PanicInfo) + Send + Sync + 'static> {
    let old_hook = panic::take_hook();
    panic::set_hook(Box::new(maybe_unwind_panic_hook));
    old_hook
}

fn maybe_unwind_panic_hook(info: &PanicInfo) {
    if !ERROR_MSG.is_set() {
        eprintln!("warning: This panic occured outside the capture range of MaybeUnwind.",);
        eprintln!("warning: It is desirable to reset the panic handler to the default by using `std::panic::take_hook`, if this panic is intentional.");
        eprintln!("warning: The original panic message is:");
        eprintln!("{}", info);
        return;
    }

    ERROR_MSG.with(|error_msg| match error_msg.try_borrow_mut() {
        Ok(mut error_msg) => {
            error_msg.replace(format!("{}", info));
        }
        Err(_) => eprintln!("bug"),
    })
}

#[derive(Debug)]
#[non_exhaustive]
pub struct UnwindContext {
    pub cause: Box<dyn Any + Send + 'static>,
    pub error_msg: Option<String>,
}

pin_project! {
    #[derive(Debug)]
    #[must_use = "futures do nothing unless you `.await` or poll them"]
    pub struct MaybeUnwind<T> {
        #[pin]
        inner: T,
    }
}

impl<F> Future for MaybeUnwind<F>
where
    F: Future + UnwindSafe,
{
    type Output = Result<F::Output, UnwindContext>;

    fn poll(self: Pin<&mut Self>, cx: &mut task::Context<'_>) -> Poll<Self::Output> {
        let me = self.project();
        let mut error_msg = RefCell::new(None);
        let poll = ERROR_MSG.set(&error_msg, || {
            panic::catch_unwind(AssertUnwindSafe(|| me.inner.poll(cx)))
        });
        match poll {
            Ok(poll) => poll.map(Ok),
            Err(cause) => {
                let error_msg = error_msg.get_mut().take();
                Poll::Ready(Err(UnwindContext { cause, error_msg }))
            }
        }
    }
}

pub trait FutureExt: Future {
    fn maybe_unwind(self) -> MaybeUnwind<Self>
    where
        Self: Sized + UnwindSafe,
    {
        MaybeUnwind { inner: self }
    }
}

impl<F: Future + ?Sized> FutureExt for F {}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::executor::block_on;

    #[allow(unreachable_code)]
    #[test]
    fn smoke_test() {
        let old_hook = set_hook();

        assert!(block_on(async { "foo" }.maybe_unwind()).is_ok());

        let unwind = block_on(
            async {
                panic!("bar");
                "foo"
            }
            .maybe_unwind(),
        )
        .unwrap_err();

        assert!(unwind.error_msg.map_or(false, |msg| msg.contains("bar")));

        panic::set_hook(old_hook);
        panic!("explicit panic");
    }
}
