extern crate dirs_next;

use std::env;
use std::path::PathBuf;

#[cfg(test)]
pub fn test_pool_ip() -> String {
    env::var("TEST_POOL_IP").unwrap_or("127.0.0.1".to_string())
}

pub fn pool_path(pool_name: &str) -> PathBuf {
    let mut path = indy_home_path();
    path.push("pool");
    path.push(pool_name);
    path
}

#[cfg(not(any(target_os = "android", target_os = "ios")))]
pub fn indy_home_path() -> PathBuf {
    let mut path = dirs_next::home_dir().unwrap();
    path.push(".indy_client");
    path
}

#[cfg(target_os = "ios")]
pub fn indy_home_path() -> PathBuf {
    let mut path = dirs_next::home_dir().unwrap();
    path.push("Documents/.indy_client");
    path
}

#[cfg(target_os = "android")]
pub fn indy_home_path() -> PathBuf {
    android_create_indy_client_dir();
    android_indy_client_dir_path()
}

#[cfg(target_os = "android")]
pub mod android {
    use std::fs;

    pub fn indy_client_dir_path() -> PathBuf {
        let external_storage = env::var("EXTERNAL_STORAGE");
        let android_dir = match external_storage {
            Ok (val) => val + "/.indy_client",
            Err(err) => {
                panic!("Failed to find external storage path {:?}", err)
            }
        };

        PathBuf::from(android_dir)
    }

    pub fn create_indy_client_dir() {
        //Creates directory only if it is not present.
        fs::create_dir_all(indy_client_dir_path().as_path()).unwrap();
    }
}
