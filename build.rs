use std::{
    env, fs,
    path::Path,
    process::{Command, ExitStatus},
};

fn main() {
    if probe_backtrace().map_or(false, |st| st.success()) {
        println!("cargo:rustc-cfg=backtrace");
    }
}

// copied from anyhow/build.rs
fn probe_backtrace() -> Option<ExitStatus> {
    let rustc = env::var_os("RUSTC")?;
    let out_dir = env::var_os("OUT_DIR")?;

    let probefile = Path::new(&out_dir).join("probe_backtrace.rs");
    fs::write(
        &probefile,
        r#"
            #![feature(backtrace)]
            #![allow(dead_code)]
            use std::{
                backtrace::{Backtrace, BacktraceStatus},
                error,
                fmt,
            };
            #[derive(Debug)]
            struct E;
            impl fmt::Display for E {
                fn fmt(&self, _formatter: &mut fmt::Formatter) -> fmt::Result {
                    unimplemented!()
                }
            }
            impl error::Error for E {
                fn backtrace(&self) -> Option<&Backtrace> {
                    let backtrace = Backtrace::capture();
                    match backtrace.status() {
                        BacktraceStatus::Captured | BacktraceStatus::Disabled | _ => {}
                    }
                    unimplemented!()
                }
            }
        "#,
    )
    .ok()?;

    Command::new(rustc)
        .arg("--edition=2018")
        .arg("--crate-name=maybe_unwind_probe_backtrace")
        .arg("--crate-type=lib")
        .arg("--emit=metadata")
        .arg("--out-dir")
        .arg(out_dir)
        .arg(probefile)
        .status()
        .ok()
}
