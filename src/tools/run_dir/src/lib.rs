use std::env;
use std::path::PathBuf;

pub fn get_run_dir() -> PathBuf {
    let path = get_run_dir_impl();

    if !path.exists() {
        std::fs::create_dir(&path).ok();
    }

    path
}

pub fn get_run_dir_impl() -> PathBuf {
    if let Ok(dir) = env::var("XDG_RUNTIME_DIR") {
        dir.into()
    } else if let Ok(uid) = env::var("UID") {
        PathBuf::from(format!("/tmp/kime-{}", uid))
    } else {
        PathBuf::from("/tmp")
    }
}
