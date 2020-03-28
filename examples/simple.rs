use maybe_unwind::{capture_panic_info, maybe_unwind};
use std::panic;

fn main() {
    let old_hook = panic::take_hook();
    panic::set_hook(Box::new(move |info| {
        let captured = capture_panic_info(info);
        if !captured {
            old_hook(info);
        }
    }));

    let res = maybe_unwind(|| {
        panic!("oops");
    });

    if let Err(unwind) = res {
        eprintln!("{:#}", unwind);
    }
}
