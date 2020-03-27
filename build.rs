use version_check as rustc;

fn main() {
    match rustc::Channel::read() {
        Some(channel) if channel.is_nightly() => {
            println!("cargo:rustc-cfg=nightly");
        }
        _ => (),
    }
}
