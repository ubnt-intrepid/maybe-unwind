use maybe_unwind::maybe_unwind;
use std::sync::Once;

fn ensure_set_hook() {
    static SET_HOOK: Once = Once::new();
    SET_HOOK.call_once(maybe_unwind::set_hook);
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
    assert!(unwind.file().map_or(false, |file| file.contains(file!())));
    assert!(unwind.line().is_some());
    assert!(unwind.column().is_some());
}

#[test]
#[should_panic(expected = "explicit panic")]
fn without_wrapper() {
    ensure_set_hook();
    panic!("explicit panic");
}

#[cfg(feature = "futures")]
mod futures {
    use super::*;
    use futures_executor::block_on;
    use maybe_unwind::FutureMaybeUnwindExt as _;

    #[allow(unreachable_code)]
    #[test]
    #[should_panic(expected = "explicit panic")]
    fn smoke() {
        ensure_set_hook();

        assert!(block_on(async { "foo" }.maybe_unwind()).is_ok());

        let unwind = block_on(
            async {
                panic!("bar");
                "foo"
            }
            .maybe_unwind(),
        )
        .unwrap_err();

        assert!(unwind.payload_str() == "bar");

        panic!("explicit panic");
    }
}
