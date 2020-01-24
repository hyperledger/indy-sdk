use std::env;
use std::fs;
use std::path::Path;

fn main() {
    let target = env::var("TARGET").unwrap();
    println!("target={}", target);

    let sodium_static = env::var("CARGO_FEATURE_SODIUM_STATIC").ok();
    println!("sodium_static={:?}", sodium_static);

    if sodium_static.is_some() {
        println!("cargo:rustc-link-lib=static=sodium");
    }

    if target.find("-windows-").is_some() {
        // do not build c-code on windows, use binaries
        let output_dir = env::var("OUT_DIR").unwrap();
        let prebuilt_dir = env::var("INDY_PREBUILT_DEPS_DIR").unwrap();

        let dst = Path::new(&output_dir[..]).join("..\\..\\..");
        let prebuilt_lib = Path::new(&prebuilt_dir[..]).join("lib");

        println!("cargo:rustc-link-search=native={}", prebuilt_dir);
        println!("cargo:rustc-flags=-L {}\\lib", prebuilt_dir);
        println!("cargo:include={}\\include", prebuilt_dir);

        let files = vec!["libeay32md.dll", "libsodium.dll", "libzmq.dll", "ssleay32md.dll"];
        for f in files.iter() {
            if let Ok(_) = fs::copy(&prebuilt_lib.join(f), &dst.join(f)) {
                println!("copy {} -> {}", &prebuilt_lib.join(f).display(), &dst.join(f).display());
            }
        }
    } else if target.find("linux-android").is_some() {
        //statically link files
        println!("cargo:rustc-link-lib=static=crypto");
        println!("cargo:rustc-link-lib=static=ssl");
        println!("cargo:rustc-link-lib=static=sodium");
        println!("cargo:rustc-link-lib=static=zmq");
    }
}
