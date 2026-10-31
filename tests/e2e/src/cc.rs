//! On-demand compilation of the C/C++ test clients in `tests/e2e/clients/`.
//!
//! Outputs go to `<repo>/target/e2e-clients/`, memoized by source mtime, and
//! are renamed into place atomically so concurrent test binaries never see a
//! half-written executable. Wayland protocol glue is generated with
//! `wayland-scanner` from the vendored XMLs in `clients/protocols/`.

use std::path::{Path, PathBuf};
use std::process::Command;

use crate::paths;
use crate::Result;

/// Path of the `gtk_probe.py` script (no compilation needed).
pub fn gtk_probe_py() -> PathBuf {
    paths::clients_dir().join("gtk_probe.py")
}

/// Path of the `gtk_reset_probe.py` script (no compilation needed).
pub fn gtk_reset_probe_py() -> PathBuf {
    paths::clients_dir().join("gtk_reset_probe.py")
}

/// Build the XIM client (`xim_client <outfile>`).
pub fn xim_client() -> Result<PathBuf> {
    let src = paths::clients_dir().join("xim_client.c");
    build("xim_client", &[src.clone()], |cmd, out| {
        cmd.args(["-O1", "-o"]).arg(out).arg(&src).arg("-lX11");
    })
}

/// Build the raw XTEST key injector (`xtest_key <keycode> <press|release|tap>`).
pub fn xtest_key() -> Result<PathBuf> {
    let src = paths::clients_dir().join("xtest_key.c");
    build("xtest_key", &[src.clone()], |cmd, out| {
        cmd.args(["-O1", "-o"])
            .arg(out)
            .arg(&src)
            .args(["-lX11", "-lXtst"]);
    })
}

/// Build the virtual-keyboard injector (reads commands from stdin).
pub fn vkbd_inject() -> Result<PathBuf> {
    let src = paths::clients_dir().join("vkbd_inject.c");
    let glue = scanner_glue("virtual-keyboard-unstable-v1")?;
    let flags = pkg_config(&["wayland-client", "xkbcommon"])?;
    build(
        "vkbd_inject",
        &[src.clone(), glue.clone()],
        move |cmd, out| {
            cmd.args(["-O1", "-I"])
                .arg(paths::clients_out_dir())
                .arg("-o")
                .arg(out)
                .arg(&src)
                .arg(&glue)
                .args(&flags);
        },
    )
}

/// Build the keymapless virtual-keyboard holder (the #782 repro
/// precondition: a seat keyboard whose keymap is NULL).
pub fn keymapless_kbd() -> Result<PathBuf> {
    let src = paths::clients_dir().join("keymapless_kbd.c");
    let glue = scanner_glue("virtual-keyboard-unstable-v1")?;
    let flags = pkg_config(&["wayland-client"])?;
    build(
        "keymapless_kbd",
        &[src.clone(), glue.clone()],
        move |cmd, out| {
            cmd.args(["-O1", "-I"])
                .arg(paths::clients_out_dir())
                .arg("-o")
                .arg(out)
                .arg(&src)
                .arg(&glue)
                .args(&flags);
        },
    )
}

/// Build the plain wl_keyboard probe (`wlkbd_probe <outfile>`).
pub fn wlkbd_probe() -> Result<PathBuf> {
    let src = paths::clients_dir().join("wlkbd_probe.c");
    let glue = scanner_glue("xdg-shell")?;
    let flags = pkg_config(&["wayland-client"])?;
    build(
        "wlkbd_probe",
        &[src.clone(), glue.clone()],
        move |cmd, out| {
            cmd.args(["-O1", "-I"])
                .arg(paths::clients_out_dir())
                .arg("-o")
                .arg(out)
                .arg(&src)
                .arg(&glue)
                .args(&flags);
        },
    )
}

/// Build the Qt probe against Qt5 or Qt6 (`qt_probe5`/`qt_probe6 <outfile>`).
pub fn qt_probe(qt_major: u32) -> Result<PathBuf> {
    assert!(qt_major == 5 || qt_major == 6, "qt_major must be 5 or 6");
    let src = paths::clients_dir().join("qt_probe.cpp");
    let pkgs = [
        format!("Qt{qt_major}Widgets"),
        format!("Qt{qt_major}Gui"),
        format!("Qt{qt_major}Core"),
    ];
    let flags = pkg_config(&pkgs.iter().map(String::as_str).collect::<Vec<_>>())?;
    build_with(
        "g++",
        &format!("qt_probe{qt_major}"),
        &[src.clone()],
        move |cmd, out| {
            cmd.args(["-O1", "-fPIC", "-std=c++17", "-o"])
                .arg(out)
                .arg(&src)
                .args(&flags);
        },
    )
}

/// Run wayland-scanner (client-header + private-code) for a vendored protocol
/// XML; returns the generated `<base>-code.c` path. The header lands next to
/// it as `<base>-client.h` (add `-I clients_out_dir()`).
fn scanner_glue(base: &str) -> Result<PathBuf> {
    let xml = paths::clients_dir().join(format!("protocols/{base}.xml"));
    if !xml.exists() {
        return Err(format!("vendored protocol XML missing: {}", xml.display()));
    }
    let out_dir = paths::clients_out_dir();
    std::fs::create_dir_all(&out_dir)
        .map_err(|e| format!("cannot create {}: {e}", out_dir.display()))?;
    let header = out_dir.join(format!("{base}-client.h"));
    let code = out_dir.join(format!("{base}-code.c"));
    for (mode, out) in [("client-header", &header), ("private-code", &code)] {
        if is_fresh(out, &[xml.clone()]) {
            continue;
        }
        let tmp = tmp_path(out);
        let status = Command::new("wayland-scanner")
            .arg(mode)
            .arg(&xml)
            .arg(&tmp)
            .status()
            .map_err(|e| format!("failed to run wayland-scanner: {e}"))?;
        if !status.success() {
            return Err(format!(
                "wayland-scanner {mode} {} failed ({status})",
                xml.display()
            ));
        }
        std::fs::rename(&tmp, out)
            .map_err(|e| format!("cannot move {} into place: {e}", out.display()))?;
    }
    Ok(code)
}

fn pkg_config(pkgs: &[&str]) -> Result<Vec<String>> {
    let output = Command::new("pkg-config")
        .args(["--cflags", "--libs"])
        .args(pkgs)
        .output()
        .map_err(|e| format!("failed to run pkg-config: {e}"))?;
    if !output.status.success() {
        return Err(format!(
            "pkg-config --cflags --libs {pkgs:?} failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    Ok(String::from_utf8_lossy(&output.stdout)
        .split_whitespace()
        .map(str::to_string)
        .collect())
}

fn build(
    name: &str,
    srcs: &[PathBuf],
    configure: impl FnOnce(&mut Command, &Path),
) -> Result<PathBuf> {
    build_with("cc", name, srcs, configure)
}

fn build_with(
    compiler: &str,
    name: &str,
    srcs: &[PathBuf],
    configure: impl FnOnce(&mut Command, &Path),
) -> Result<PathBuf> {
    let out_dir = paths::clients_out_dir();
    std::fs::create_dir_all(&out_dir)
        .map_err(|e| format!("cannot create {}: {e}", out_dir.display()))?;
    let out = out_dir.join(name);
    if is_fresh(&out, srcs) {
        return Ok(out);
    }
    let tmp = tmp_path(&out);
    let mut cmd = Command::new(compiler);
    configure(&mut cmd, &tmp);
    let output = cmd
        .output()
        .map_err(|e| format!("failed to run {compiler} for {name}: {e}"))?;
    if !output.status.success() {
        return Err(format!(
            "compiling {name} failed ({}):\n{}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    std::fs::rename(&tmp, &out)
        .map_err(|e| format!("cannot move {} into place: {e}", out.display()))?;
    Ok(out)
}

/// True when `out` exists and is newer than every source.
fn is_fresh(out: &Path, srcs: &[PathBuf]) -> bool {
    let Ok(out_m) = std::fs::metadata(out).and_then(|m| m.modified()) else {
        return false;
    };
    srcs.iter().all(|s| {
        std::fs::metadata(s)
            .and_then(|m| m.modified())
            .map_or(false, |src_m| src_m < out_m)
    })
}

fn tmp_path(out: &Path) -> PathBuf {
    out.with_extension(format!("tmp{}", std::process::id()))
}
