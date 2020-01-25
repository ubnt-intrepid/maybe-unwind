use futures::{
    future::Future,
    task::{self, Poll},
};
use lazy_static::lazy_static;
use pin_project_lite::pin_project;
use scoped_tls::scoped_thread_local;
use std::{
    any::Any,
    cell::RefCell,
    panic::{self, AssertUnwindSafe, PanicInfo, UnwindSafe},
    pin::Pin,
    sync::RwLock,
};

type PanicHook = dyn Fn(&PanicInfo) + Send + Sync + 'static;

lazy_static! {
    static ref OLD_HOOK: RwLock<Option<Box<PanicHook>>> = RwLock::new(None);
}

scoped_thread_local! {
    static CAPTURED_PANIC_INFO: RefCell<Option<CapturedPanicInfo>>
}

#[inline]
pub fn set_hook() {
    match OLD_HOOK.write() {
        Ok(mut old_hook) => {
            old_hook.get_or_insert_with(panic::take_hook);
        }
        Err(err) => panic!("{}", err),
    }
    panic::set_hook(Box::new(maybe_unwind_panic_hook));
}

#[inline]
pub fn reset_hook() {
    if let Ok(mut old_lock) = OLD_HOOK.write() {
        if let Some(old_hook) = old_lock.take() {
            panic::set_hook(old_hook);
        }
    }
}

fn maybe_unwind_panic_hook(info: &PanicInfo) {
    if !CAPTURED_PANIC_INFO.is_set() {
        fallback_hook(info);
        return;
    }

    CAPTURED_PANIC_INFO.with(|captured| match captured.try_borrow_mut() {
        Ok(mut captured) => {
            captured.replace(CapturedPanicInfo {
                message: info.to_string(),
                file: info.location().map(|loc| loc.file().to_string()),
                line: info.location().map(|loc| loc.line()),
                column: info.location().map(|loc| loc.column()),
            });
        }
        Err(_) => eprintln!("bug"),
    })
}

fn fallback_hook(info: &PanicInfo) {
    let old_hook = OLD_HOOK.read().ok();
    let old_hook = old_hook.as_ref().and_then(|old_hook| old_hook.as_ref());
    if let Some(old_hook) = old_hook {
        (old_hook)(info);
    } else {
        eprintln!("warning: the original panic hook is not available (this is a bug).");
    }
}

#[derive(Debug)]
#[non_exhaustive]
pub struct CapturedPanicInfo {
    pub message: String,
    pub file: Option<String>,
    pub line: Option<u32>,
    pub column: Option<u32>,
}

#[derive(Debug)]
#[non_exhaustive]
pub struct UnwindContext {
    pub cause: Box<dyn Any + Send + 'static>,
    pub captured: Option<CapturedPanicInfo>,
}

pub fn maybe_unwind<F, R>(f: F) -> Result<R, UnwindContext>
where
    F: FnOnce() -> R + UnwindSafe,
{
    let mut captured = RefCell::new(None);
    CAPTURED_PANIC_INFO
        .set(&captured, || panic::catch_unwind(f))
        .map_err(|cause| UnwindContext {
            cause,
            captured: captured.get_mut().take(),
        })
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
        maybe_unwind(AssertUnwindSafe(|| me.inner.poll(cx)))?.map(Ok)
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
    #[should_panic(expected = "explicit panic")]
    fn smoke_test() {
        set_hook();

        assert!(block_on(async { "foo" }.maybe_unwind()).is_ok());

        let unwind = block_on(
            async {
                panic!("bar");
                "foo"
            }
            .maybe_unwind(),
        )
        .unwrap_err();

        assert!(unwind
            .captured
            .map_or(false, |captured| captured.message.contains("bar")));

        panic!("explicit panic");
    }
}
