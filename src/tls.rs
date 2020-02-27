use crate::unwind::Captured;
use std::{cell::Cell, mem, ptr::NonNull};

pub(crate) struct Context<'a> {
    pub(crate) captured: &'a mut Option<Captured>,
}

impl Context<'_> {
    pub(crate) fn is_set() -> bool {
        TLS_CTX.with(|tls| tls.get().is_some())
    }

    pub(crate) fn scope<F, R>(&mut self, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        unsafe {
            let ctx_ptr = mem::transmute::<&mut Self, &mut Context<'static>>(self);
            let old_ctx = TLS_CTX.with(|tls| tls.replace(Some(NonNull::from(ctx_ptr))));
            let _guard = Guard(old_ctx);
            f()
        }
    }

    pub(crate) fn try_with<F, R>(f: F) -> Result<R, AccessError>
    where
        F: FnOnce(&mut Context<'_>) -> R,
    {
        let ctx_ptr = TLS_CTX.with(|tls| tls.take());
        let _guard = Guard(ctx_ptr);
        match ctx_ptr {
            Some(mut ctx_ptr) => unsafe { Ok(f(ctx_ptr.as_mut())) },
            None => Err(AccessError(())),
        }
    }
}

thread_local! {
    static TLS_CTX: Cell<Option<NonNull<Context<'static>>>> = Cell::new(None);
}

struct Guard(Option<NonNull<Context<'static>>>);

impl Drop for Guard {
    fn drop(&mut self) {
        TLS_CTX.with(|tls| {
            tls.set(self.0.take());
        });
    }
}

pub(crate) struct AccessError(());
