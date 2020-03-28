use crate::unwind::Captured;
use std::{cell::Cell, ptr::NonNull};

pub(crate) struct Context<'a> {
    pub(crate) captured: &'a mut Option<Captured>,
}

impl Context<'_> {
    pub(crate) fn is_set() -> bool {
        TLS_CTX.with(|tls| tls.get().is_some())
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
    pub(crate) static TLS_CTX: Cell<Option<NonNull<Context<'static>>>> = Cell::new(None);
}

pub(crate) struct Guard(pub(crate) Option<NonNull<Context<'static>>>);

impl Drop for Guard {
    fn drop(&mut self) {
        TLS_CTX.with(|tls| {
            tls.set(self.0.take());
        });
    }
}

pub(crate) struct AccessError(());

macro_rules! with_set_ctx {
    ($ctx:expr, $body:block) => {{
        use crate::context::{Context, Guard, TLS_CTX};
        use std::{mem, ptr::NonNull};
        let ctx = $ctx;
        let old_ctx = unsafe {
            let ctx_ptr = mem::transmute::<_, &mut Context<'static>>(ctx);
            TLS_CTX.with(|tls| tls.replace(Some(NonNull::from(ctx_ptr))))
        };
        let _guard = Guard(old_ctx);
        $body
    }};
}
