extern crate dirs_next;

use std::path::PathBuf;
use std::env;

pub fn tmp_path() -> PathBuf {
    let mut path = env::temp_dir();
    path.push("indy_client");
    path
}

pub fn indy_home_path() -> PathBuf {
    // TODO: FIXME: Provide better handling for the unknown home path case!!!
    let mut path = dirs_next::home_dir().unwrap_or(PathBuf::from("/home/indy"));
    let mut indy_client_dir = ".indy_client";
    if cfg!(target_os = "ios") {
        indy_client_dir = "Documents/.indy_client";
    }
    path.push(indy_client_dir);

    if cfg!(target_os = "android") {
        path = android_indy_client_dir_path();
    }
    path
}

pub fn android_indy_client_dir_path() -> PathBuf {
    let external_storage = env::var("EXTERNAL_STORAGE");
    let android_dir: String;
    match external_storage {
        Ok(val) => android_dir = val + "/.indy_client",
        Err(err) => {
            panic!("Failed to find external storage path {:?}", err)
        }
    }

    PathBuf::from(android_dir)
}

#[cfg(test)]
pub fn test_pool_ip() -> String {
    env::var("TEST_POOL_IP").unwrap_or("127.0.0.1".to_string())
}

pub fn tmp_file_path(file_name: &str) -> PathBuf {
    let mut path = tmp_path();
    path.push(file_name);
    path
}
