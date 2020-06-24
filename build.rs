use std::env;

pub fn main() {
    if let Ok(profile) = env::var("PROFILE") {
        if profile == "debug" {
            println!("cargo:rustc-cfg=debug");
        }
    }
}