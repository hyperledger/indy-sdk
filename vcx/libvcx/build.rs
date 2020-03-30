use std::env;
use std::fs;
use std::io::prelude::*;
use std::fs::File;
use std::path::Path;

extern crate toml;

extern crate serde_derive;

// error in rust compiler.  Bugfix requested in Sept. 2017
// these are used, but the compiler is not seeing it for
// some reason
#[allow(unused_imports)]
#[macro_use]
extern crate serde_json;
// error in rust compiler.  Bugfix has been submitted in Sept. 2017
#[allow(unused_imports)]
#[macro_use]
extern crate serde;

// used in formatting the Cargo.toml file
#[derive(Deserialize, Debug)]
struct Tomlfile {
    contents: Contents,
}

// used in formatting the Cargo.toml file
#[derive(Deserialize, Debug)]
struct Metadata {
    deb: Deb,
}

// used in formatting the Cargo.toml file
#[derive(Deserialize, Debug)]
struct Deb {
    revision: Option<String>,
}


// used in formatting the Cargo.toml file
#[derive(Deserialize, Debug)]
struct Package {
    name: String,
    version: Option<String>,
    metadata: Metadata,
}

// used in formatting the Cargo.toml file
#[derive(Deserialize, Debug)]
struct Contents {
    package: Package,
    dependencies: Option<toml::Value>,
}

fn main() {
    let target = env::var("TARGET").unwrap();
    println!("target={}", target);

    if let Ok(_mode) = env::var("LIBINDY_STATIC") {
        let libindy_lib_path = env::var("LIBINDY_DIR").unwrap();
        println!("cargo:rustc-link-search=native={}",libindy_lib_path);
        println!("cargo:rustc-link-lib=static=indy");
    }else if target.contains("aarch64-linux-android") || target.contains("armv7-linux-androideabi") ||
        target.contains("arm-linux-androideabi") || target.contains("i686-linux-android") ||
        target.contains("x86_64-linux-android") || target.contains("aarch64-apple-ios") ||
        target.contains("armv7-apple-ios") || target.contains("armv7s-apple-ios") ||
        target.contains("i386-apple-ios") || target.contains("x86_64-apple-ios") {

        let libindy_lib_path = match env::var("LIBINDY_DIR"){
            Ok(val) => val,
            Err(..) => panic!("Missing required environment variable LIBINDY_DIR")
        };

        let openssl = match env::var("OPENSSL_LIB_DIR") {
            Ok(val) => val,
            Err(..) => match env::var("OPENSSL_DIR") {
                Ok(dir) => Path::new(&dir[..]).join("/lib").to_string_lossy().into_owned(),
                Err(..) => panic!("Missing required environment variables OPENSSL_DIR or OPENSSL_LIB_DIR")
            }
        };

        println!("cargo:rustc-link-search=native={}",libindy_lib_path);
        println!("cargo:rustc-link-lib=static=indy");
        println!("cargo:rustc-link-search=native={}", openssl);
        println!("cargo:rustc-link-lib=static=crypto");
        println!("cargo:rustc-link-lib=static=ssl");

    }else if target.contains("darwin"){
        //OSX specific logic
        println!("cargo:rustc-link-lib=sodium");
        println!("cargo:rustc-link-lib=zmq");
        println!("cargo:rustc-link-lib=indy");
        //OSX does not allow 3rd party libs to be installed in /usr/lib. Instead install it in /usr/local/lib
        println!("cargo:rustc-link-search=native=/usr/local/lib");
    }else if target.contains("-linux-"){
        //Linux specific logic
        println!("cargo:rustc-link-lib=indy");
        println!("cargo:rustc-link-search=native=/usr/lib");
    }else if target.contains("-windows-") {
        println!("cargo:rustc-link-lib=indy.dll");

        let profile = env::var("PROFILE").unwrap();
        println!("profile={}", profile);

        let output_dir = env::var("OUT_DIR").unwrap();
        println!("output_dir={}", output_dir);
        let output_dir = Path::new(output_dir.as_str());

        let indy_dir = env::var("INDY_DIR").unwrap_or(format!("..\\..\\libindy\\target\\{}", profile));
        println!("indy_dir={}", indy_dir);
        let indy_dir = Path::new(indy_dir.as_str());

        let dst = output_dir.join("..\\..\\..\\..");
        println!("cargo:rustc-flags=-L {}", indy_dir.as_os_str().to_str().unwrap());

        let files = vec!["indy.dll", "libeay32md.dll", "libsodium.dll", "libzmq.dll", "ssleay32md.dll"];
        for f in files.iter() {
            if let Ok(_) = fs::copy(&indy_dir.join(f), &dst.join(f)) {
                println!("copy {} -> {}", &indy_dir.join(f).display(), &dst.join(f).display());
            }
        }
    }

    match env::var("CARGO_FEATURE_CI") {
        Ok(_) => {
            println!("injecting version information");
            // Leaving as unwrap, this is in the build script.
            let revision = get_revision().unwrap();
            write_variables(&revision);
        },
        Err(_) => {println!("NOT injecting version information"); },
    };
}


// Writes to the file 'src/utils/version_constants.rs' for use
// in outputing the version dynamically.
fn write_variables(revision:&str) {
    let out_dir = "src/utils/";
    let dest_path = Path::new(&out_dir).join("version_constants.rs");
    let mut f = File::create(&dest_path).unwrap();
    let s = format!("pub const VERSION: &'static str = env!(\"CARGO_PKG_VERSION\");\npub const REVISION: &'static str = \"{}\";\n", revision);
    if let Err(e) = f.write_all(s.as_bytes()) {
       panic!("Error creating version_constants.rs: {}", e);
    };

}

// Gets the revision number from the Cargo.toml file.
pub fn get_revision() -> Option<String> {
    let dir = match  env::var("CARGO_MANIFEST_DIR"){
        Ok(d) => d,
        Err(_) => panic!("Couldn't Manifest Directory"),
    };
    let filename = "Cargo.toml";
    let p = format!("{}/{}",dir,filename);
    let mut input = String::new();
    File::open(p).and_then(|mut f| {
        f.read_to_string(&mut input)}).unwrap();
    let tomlfile:Contents = toml::from_str(&input).unwrap();
    let revision:String = match tomlfile.package.metadata.deb.revision {
        Some(v) => v,
        None => String::from(""),
    };
    Some(format!("+{}", revision))
}

