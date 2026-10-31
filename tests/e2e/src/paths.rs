//! Artifact discovery and per-test scratch directories.
//!
//! Env overrides:
//! - `KIME_E2E_BUILD_DIR`  — meson build dir (default `<repo>/build`)
//! - `KIME_E2E_TARGET_DIR` — cargo output dir (default `<repo>/target/debug`)
//! - `KIME_E2E_KEEP_LOGS=1` — keep scratch dirs even on success

use std::path::{Path, PathBuf};

/// Repository root (parent of `tests/e2e`).
pub fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("tests/e2e has a repo root two levels up")
        .to_path_buf()
}

/// Meson build directory containing the C/C++ frontend immodules.
pub fn build_dir() -> PathBuf {
    match std::env::var_os("KIME_E2E_BUILD_DIR") {
        Some(d) => PathBuf::from(d),
        None => repo_root().join("build"),
    }
}

/// Cargo output directory containing kime binaries and `libkime_engine.so`.
pub fn target_dir() -> PathBuf {
    match std::env::var_os("KIME_E2E_TARGET_DIR") {
        Some(d) => PathBuf::from(d),
        None => build_dir().join("target/debug"),
    }
}

fn require(path: PathBuf, what: &str) -> PathBuf {
    if !path.exists() {
        panic!(
            "{what} not found at {}.\n\
             Build the artifacts first:\n  meson setup build --buildtype=debug -Dcargo_profile=debug\n  ninja -C build\n\
             or point KIME_E2E_BUILD_DIR / KIME_E2E_TARGET_DIR at an existing build.",
            path.display()
        );
    }
    path
}

/// Path of a kime binary in the target dir (`kime-xim`, `kime-wayland`, ...),
/// panicking with a build hint if missing.
pub fn bin(name: &str) -> PathBuf {
    require(target_dir().join(name), name)
}

/// Frontend immodules produced by the meson build.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Frontend {
    Gtk3,
    Gtk4,
    Qt5,
    Qt6,
}

fn immodule_rel(frontend: Frontend) -> &'static str {
    match frontend {
        Frontend::Gtk3 => "src/frontends/gtk3/libim-kime.so",
        Frontend::Gtk4 => "src/frontends/gtk4/libkime-gtk4.so",
        Frontend::Qt5 => "src/frontends/qt5/libkimeplatforminputcontextplugin.so",
        Frontend::Qt6 => "src/frontends/qt6/libkimeplatforminputcontextplugin.so",
    }
}

/// Path of a frontend immodule .so in the meson build dir.
pub fn immodule(frontend: Frontend) -> PathBuf {
    let rel = immodule_rel(frontend);
    require(build_dir().join(rel), rel)
}

/// [`immodule`], but `None` when the build did not produce it — for frontends
/// a build may legitimately leave out. Only Qt5 qualifies: nixpkgs refuses
/// Qt5 and Qt6 in one environment, so the CI devshell builds the Qt6 plugin
/// only. Every other frontend must be present, and [`immodule`] says so.
pub fn immodule_opt(frontend: Frontend) -> Option<PathBuf> {
    let path = build_dir().join(immodule_rel(frontend));
    path.exists().then_some(path)
}

/// `tests/e2e/clients` (C/C++/python sources vendored with the suite).
pub fn clients_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("clients")
}

/// Where cc.rs puts compiled clients and wayland-scanner glue.
pub fn clients_out_dir() -> PathBuf {
    repo_root().join("target/e2e-clients")
}

/// Per-test scratch directory under `<repo>/target/e2e/`.
///
/// Removed on clean drop; kept (with its path printed) when the test panicked
/// or `KIME_E2E_KEEP_LOGS=1` is set.
pub struct ScratchDir {
    path: PathBuf,
}

impl ScratchDir {
    pub fn new(test_name: &str) -> ScratchDir {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .subsec_nanos();
        let path = repo_root().join(format!(
            "target/e2e/{test_name}-{}-{nanos}",
            std::process::id()
        ));
        std::fs::create_dir_all(&path)
            .unwrap_or_else(|e| panic!("cannot create scratch dir {}: {e}", path.display()));
        eprintln!("[e2e] scratch dir: {}", path.display());
        ScratchDir { path }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Path of a (not necessarily existing) file inside the scratch dir.
    pub fn file(&self, name: &str) -> PathBuf {
        self.path.join(name)
    }

    /// Create a subdirectory inside the scratch dir.
    pub fn subdir(&self, name: &str) -> PathBuf {
        let d = self.path.join(name);
        std::fs::create_dir_all(&d)
            .unwrap_or_else(|e| panic!("cannot create scratch subdir {}: {e}", d.display()));
        d
    }
}

impl Drop for ScratchDir {
    fn drop(&mut self) {
        let keep = std::thread::panicking()
            || std::env::var("KIME_E2E_KEEP_LOGS").map_or(false, |v| v == "1");
        if keep {
            eprintln!("[e2e] keeping logs in {}", self.path.display());
        } else {
            let _ = std::fs::remove_dir_all(&self.path);
        }
    }
}
