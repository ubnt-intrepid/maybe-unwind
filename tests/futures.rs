#![cfg(feature = "futures")]

use futures_executor::block_on;
use maybe_unwind::FutureMaybeUnwindExt as _;
use std::sync::Once;

fn ensure_set_hook() {
    static SET_HOOK: Once = Once::new();
    SET_HOOK.call_once(|| maybe_unwind::set_hook());
}

#[allow(unreachable_code)]
#[test]
#[should_panic(expected = "explicit panic")]
fn smoke_test() {
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

    let captured = unwind.captured.expect("empty capture");
    assert!(captured.payload() == Some("bar"));

    panic!("explicit panic");
}
