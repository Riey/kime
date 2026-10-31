//! No-install immodule staging (verified layouts) + `/proc/<pid>/maps` checks.
//!
//! System-wide kime immodules exist on dev machines and would silently shadow
//! the freshly built ones; every immodule test must call [`maps_check`] with
//! the staged module path to prove the LOCAL .so loaded.

use std::path::PathBuf;

use crate::envs::{self, clean_cmd};
use crate::paths::{self, Frontend, ScratchDir};
use crate::Result;

/// A staged immodule: env vars to add to the probe process, and the staged
/// module path (use as the [`maps_check`] needle).
pub struct Staged {
    /// Environment the probe needs (`GTK_IM_MODULE`, `GTK_IM_MODULE_FILE` /
    /// `GTK_PATH` / `QT_PLUGIN_PATH`, ...).
    pub env: Vec<(String, String)>,
    /// Absolute path of the staged .so inside the scratch dir.
    pub module_path: PathBuf,
}

/// GTK3: copy the module, generate a private immodule cache with
/// `gtk-query-immodules-3.0`, and point `GTK_IM_MODULE_FILE` at it
/// (fully overrides the system cache — verified).
pub fn stage_gtk3(scratch: &ScratchDir) -> Result<Staged> {
    let dir = scratch.subdir("gtk3mod");
    let module = dir.join("libim-kime.so");
    copy(&paths::immodule(Frontend::Gtk3), &module)?;
    let cache = dir.join("immodules.cache");
    // The query tool dlopens the module, which links against libkime_engine.so.
    let output = clean_cmd("gtk-query-immodules-3.0")
        .env("LD_LIBRARY_PATH", envs::ld_library_path())
        .arg(&module)
        .output()
        .map_err(|e| format!("failed to run gtk-query-immodules-3.0: {e}"))?;
    if !output.status.success() {
        return Err(format!(
            "gtk-query-immodules-3.0 {} failed ({}): {}",
            module.display(),
            output.status,
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    std::fs::write(&cache, &output.stdout)
        .map_err(|e| format!("cannot write {}: {e}", cache.display()))?;
    Ok(Staged {
        env: vec![
            ("GTK_IM_MODULE".into(), "kime".into()),
            ("GTK_IM_MODULE_FILE".into(), cache.display().to_string()),
        ],
        module_path: module,
    })
}

/// GTK4: `GTK_PATH=<dir>` with the module at `<dir>/4.0.0/immodules/` —
/// empirically verified layout (NOT `<dir>/gtk-4.0/4.0.0/immodules/`).
pub fn stage_gtk4(scratch: &ScratchDir) -> Result<Staged> {
    let root = scratch.subdir("gtk4stage");
    let moddir = root.join("4.0.0/immodules");
    std::fs::create_dir_all(&moddir)
        .map_err(|e| format!("cannot create {}: {e}", moddir.display()))?;
    let module = moddir.join("libkime-gtk4.so");
    copy(&paths::immodule(Frontend::Gtk4), &module)?;
    Ok(Staged {
        env: vec![
            ("GTK_IM_MODULE".into(), "kime".into()),
            ("GTK_PATH".into(), root.display().to_string()),
        ],
        module_path: module,
    })
}

/// Qt5/Qt6: `QT_PLUGIN_PATH=<dir>` with the plugin at
/// `<dir>/platforminputcontexts/libkimeplatforminputcontextplugin.so`.
pub fn stage_qt(scratch: &ScratchDir, qt_major: u32) -> Result<Staged> {
    let frontend = match qt_major {
        5 => Frontend::Qt5,
        6 => Frontend::Qt6,
        n => return Err(format!("stage_qt: unsupported Qt major {n}")),
    };
    let root = scratch.subdir(&format!("qt{qt_major}stage"));
    let moddir = root.join("platforminputcontexts");
    std::fs::create_dir_all(&moddir)
        .map_err(|e| format!("cannot create {}: {e}", moddir.display()))?;
    let module = moddir.join("libkimeplatforminputcontextplugin.so");
    copy(&paths::immodule(frontend), &module)?;
    Ok(Staged {
        env: vec![
            ("QT_IM_MODULE".into(), "kime".into()),
            ("QT_PLUGIN_PATH".into(), root.display().to_string()),
        ],
        module_path: module,
    })
}

/// Assert that `/proc/<pid>/maps` contains `path_substring` — i.e. the process
/// mapped the staged local .so, not a system copy. The error lists every
/// kime-related mapping to make shadowing obvious.
pub fn maps_check(pid: i32, path_substring: &str) -> Result<()> {
    let maps_path = format!("/proc/{pid}/maps");
    let maps =
        std::fs::read_to_string(&maps_path).map_err(|e| format!("cannot read {maps_path}: {e}"))?;
    if maps.lines().any(|l| l.contains(path_substring)) {
        return Ok(());
    }
    let kime_maps: Vec<&str> = maps.lines().filter(|l| l.contains("kime")).collect();
    Err(format!(
        "pid {pid} did not map {path_substring};\nkime-related mappings:\n{}",
        if kime_maps.is_empty() {
            "  (none)".to_string()
        } else {
            kime_maps.join("\n")
        }
    ))
}

/// [`maps_check`] against a [`Staged`] module.
pub fn maps_check_staged(pid: i32, staged: &Staged) -> Result<()> {
    maps_check(
        pid,
        staged
            .module_path
            .to_str()
            .ok_or("staged module path is not UTF-8")?,
    )
}

fn copy(from: &std::path::Path, to: &std::path::Path) -> Result<()> {
    std::fs::copy(from, to)
        .map(|_| ())
        .map_err(|e| format!("cannot copy {} -> {}: {e}", from.display(), to.display()))
}
