#![deny(deprecated)]

use maybe_unwind::maybe_unwind;
use std::{panic::PanicInfo, sync::Once};

fn ensure_set_hook() {
    fn test_hook(info: &PanicInfo) {
        maybe_unwind::capture_panic_info(info);
    }

    static SET_HOOK: Once = Once::new();
    SET_HOOK.call_once(|| {
        std::panic::set_hook(Box::new(test_hook));
    });
}

#[test]
fn never_unwind() {
    ensure_set_hook();
    assert!(maybe_unwind(|| "foo").is_ok());
}

#[allow(unreachable_code)]
#[test]
fn has_unwind() {
    ensure_set_hook();
    let unwind = maybe_unwind(|| {
        panic!("bar");
        "foo"
    })
    .unwrap_err();
    assert_eq!(unwind.payload_str(), "bar");
    assert!(unwind
        .location()
        .map_or(false, |loc| loc.file().contains(file!())));
}

#[test]
#[should_panic(expected = "explicit panic")]
fn without_wrapper() {
    ensure_set_hook();
    panic!("explicit panic");
}

#[allow(unreachable_code)]
#[test]
fn nested1() {
    ensure_set_hook();
    let res = maybe_unwind(|| {
        maybe_unwind(|| {
            panic!("bar");
            "baz"
        })
    });
    let res = res.unwrap();
    let unwind = res.unwrap_err();
    assert_eq!(unwind.payload_str(), "bar");
}

#[allow(unreachable_code)]
#[test]
fn nested2() {
    ensure_set_hook();
    let res = maybe_unwind(|| {
        let _ = maybe_unwind(|| {
            panic!("bar");
            "baz"
        });
        panic!("foo");
    });
    let unwind = res.unwrap_err();
    assert_eq!(unwind.payload_str(), "foo");
}

#[cfg(feature = "futures")]
mod futures {
    use super::ensure_set_hook;
    use futures_executor::block_on;
    use maybe_unwind::FutureMaybeUnwindExt as _;

    #[test]
    fn never_unwind() {
        ensure_set_hook();
        block_on(async {
            assert!(async { "foo" }.maybe_unwind().await.is_ok());
        })
    }

    #[allow(unreachable_code)]
    #[test]
    fn has_unwind() {
        ensure_set_hook();
        block_on(async {
            let unwind = async {
                panic!("bar");
                "foo"
            }
            .maybe_unwind()
            .await
            .unwrap_err();
            assert_eq!(unwind.payload_str(), "bar");
            assert!(unwind
                .location()
                .map_or(false, |loc| loc.file().contains(file!())));
        })
    }

    #[allow(unreachable_code)]
    #[test]
    fn nested1() {
        ensure_set_hook();
        block_on(async {
            let res = async {
                async {
                    panic!("bar");
                    "baz"
                }
                .maybe_unwind()
                .await
            }
            .maybe_unwind()
            .await;
            let res = res.unwrap();
            let unwind = res.unwrap_err();
            assert_eq!(unwind.payload_str(), "bar");
        })
    }

    #[allow(unreachable_code)]
    #[test]
    fn nested2() {
        ensure_set_hook();
        block_on(async {
            let res = async {
                let _ = async {
                    panic!("bar");
                    "baz"
                }
                .maybe_unwind()
                .await;
                panic!("foo");
            }
            .maybe_unwind()
            .await;
            let unwind = res.unwrap_err();
            assert_eq!(unwind.payload_str(), "foo");
        })
    }
}

#[cfg(feature = "nightly")]
mod backtrace {
    use super::*;

    #[test]
    fn smoke() {
        ensure_set_hook();
        let unwind = maybe_unwind(|| {
            panic!("oops");
        })
        .unwrap_err();
        let _backtrace = unwind.backtrace().unwrap();
    }
}
