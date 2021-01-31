use anyhow::{Context, Result};
use is_executable::is_executable;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::{collections::HashMap, path::Path};
use structopt::StructOpt;
use strum::IntoEnumIterator;

trait CommandExt: Sized {
    fn mode(&mut self, mode: BuildMode) -> &mut Self;
}

impl CommandExt for Command {
    fn mode(&mut self, mode: BuildMode) -> &mut Self {
        if mode.is_release() {
            self.arg("--release")
        } else {
            self
        }
    }
}

#[derive(Clone, Copy, strum_macros::EnumString)]
enum BuildMode {
    Debug,
    Release,
}

impl BuildMode {
    fn cmake_build_type(self) -> &'static str {
        match self {
            BuildMode::Debug => "-DCMAKE_BUILD_TYPE=Debug",
            BuildMode::Release => "-DCMAKE_BUILD_TYPE=Release",
        }
    }

    fn is_release(self) -> bool {
        match self {
            BuildMode::Debug => false,
            BuildMode::Release => true,
        }
    }

    fn cargo_target_dir(self) -> &'static str {
        match self {
            BuildMode::Debug => "target/debug",
            BuildMode::Release => "target/release",
        }
    }
}

#[derive(
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    strum_macros::EnumString,
    strum_macros::Display,
    strum_macros::EnumIter,
)]
enum Frontend {
    #[strum(to_string = "XIM")]
    Xim,
    #[strum(to_string = "WAYLAND")]
    Wayland,
    #[strum(to_string = "QT5")]
    Qt5,
    #[strum(to_string = "QT6")]
    Qt6,
    #[strum(to_string = "GTK2")]
    Gtk2,
    #[strum(to_string = "GTK3")]
    Gtk3,
    #[strum(to_string = "GTK4")]
    Gtk4,
}

impl Frontend {
    pub fn is_cmake(self) -> bool {
        match self {
            Frontend::Xim | Frontend::Wayland => false,
            _ => true,
        }
    }
}

#[derive(StructOpt)]
enum TaskCommand {
    Test,
    Build {
        #[structopt(long, parse(try_from_str), default_value = "Release")]
        mode: BuildMode,
        #[structopt(
            parse(try_from_str),
            about = "Select frontend availiable list: [XIM, WAYLAND, QT5, QT6, GTK2, GTK3, GTK4]"
        )]
        frontends: Vec<Frontend>,
    },
    Install {
        #[structopt(parse(from_os_str))]
        target_path: PathBuf,
    },
    ReleaseDeb {
        #[structopt(parse(from_os_str))]
        target_path: Option<PathBuf>,
    },
}

impl TaskCommand {
    pub fn run(self) -> Result<()> {
        let current_dir = env::current_dir()?;

        let project_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap();
        let src_path = project_root.join("src");
        let res_path = project_root.join("res");
        let build_path = project_root.join("build");

        env::set_current_dir(project_root)?;

        match self {
            TaskCommand::ReleaseDeb { target_path } => {
                let target_path = target_path.unwrap_or(current_dir);
                let deb_dir = tempfile::tempdir()?;
                let control_path = deb_dir.as_ref().join("DEBIAN/control");
                fs::create_dir_all(control_path.parent().unwrap())?;

                fs::write(
                    control_path,
                    fs::read_to_string(src_path.join("xtask").join("control.in"))?
                        .replace("%VER%", env!("CARGO_PKG_VERSION")),
                )
                .context("Write control")?;

                // Install into tempdir
                TaskCommand::Install {
                    target_path: deb_dir.path().into(),
                }
                .run()?;

                Command::new("dpkg-deb")
                    .arg("--build")
                    .arg(deb_dir.as_ref())
                    .arg(target_path.join(format!("kime_{}_amd64.deb", env!("CARGO_PKG_VERSION"))))
                    .spawn()?
                    .wait()?;
            }
            TaskCommand::Install { target_path } => {
                let out_path = build_path.join("out");

                install(out_path.join("kimed"), target_path.join("usr/bin/kimed"))?;
                install(
                    out_path.join("kime-xim"),
                    target_path.join("usr/bin/kime-xim"),
                )?;
                install(
                    out_path.join("kime-wayland"),
                    target_path.join("usr/bin/kime-wayland"),
                )?;
                install(
                    out_path.join("libkime-gtk2.so"),
                    target_path.join("usr/lib/gtk-2.0/2.10.0/immodules/im-kime.so"),
                )?;
                install(
                    out_path.join("libkime-gtk3.so"),
                    target_path.join("usr/lib/gtk-3.0/3.0.0/immodules/im-kime.so"),
                )?;
                install(
                    out_path.join("libkime-gtk4.so"),
                    target_path.join("usr/lib/gtk-4.0/4.0.0/immodules/libkime-gtk4.so"),
                )?;
                install(out_path.join("libkime-qt5.so"), target_path.join("usr/lib/qt/plugins/platforminputcontexts/libkimeplatforminputcontextplugin.so"))?;
                install(out_path.join("libkime-qt6.so"), target_path.join("usr/lib/qt6/plugins/platforminputcontexts/libkimeplatforminputcontextplugin.so"))?;
                install(
                    out_path.join("libkime_engine.so"),
                    target_path.join("usr/lib/libkime_engine.so"),
                )?;
                install(
                    out_path.join("kime_engine.h"),
                    target_path.join("usr/include/kime_engine.h"),
                )?;
                install(
                    out_path.join("config.yaml"),
                    target_path.join("etc/kime/config.yaml"),
                )?;
            }
            TaskCommand::Test => {
                Command::new("cargo")
                    .args(&["test", "-p=kime-engine-core"])
                    .spawn()?
                    .wait()?;
            }
            TaskCommand::Build { frontends, mode } => {
                let mut frontends = frontends
                    .into_iter()
                    .map(|f| (f, true))
                    .collect::<HashMap<_, _>>();

                for f in Frontend::iter() {
                    frontends.entry(f).or_insert(false);
                }

                let out_path = build_path.join("out");
                let cmake_path = build_path.join("cmake");
                let cmake_out_path = cmake_path.join("lib");

                fs::create_dir_all(&out_path)?;
                fs::create_dir_all(&cmake_out_path)?;

                build_core(mode)?;

                fs::copy(
                    project_root
                        .join(mode.cargo_target_dir())
                        .join("libkime_engine.so"),
                    out_path.join("libkime_engine.so"),
                )?;

                let mut cargo_projects = vec![("kimed", "kimed")];

                let mut cargo = Command::new("cargo");

                cargo.arg("build").mode(mode);

                if frontends[&Frontend::Xim] {
                    cargo_projects.push(("kime-xim", "kime-xim"));
                }

                if frontends[&Frontend::Wayland] {
                    cargo_projects.push(("kime-wayland", "kime-wayland"));
                }

                for (package, _binary) in cargo_projects.iter().copied() {
                    cargo.arg("-p").arg(package);
                }

                assert!(cargo.spawn()?.wait()?.success());

                for (_package, binary) in cargo_projects.iter().copied() {
                    fs::copy(
                        project_root.join(mode.cargo_target_dir()).join(binary),
                        out_path.join(binary),
                    )
                    .context("Copy binary file")?;
                }

                let mut cmake_command = Command::new("cmake");

                cmake_command
                    .current_dir(&cmake_path)
                    .arg(&src_path)
                    .arg("-GNinja")
                    .arg(mode.cmake_build_type());

                for (frontend, on) in frontends.iter() {
                    if !frontend.is_cmake() {
                        continue;
                    }
                    let flag = if *on { "ON" } else { "OFF" };

                    cmake_command.arg(format!("-DENABLE_{}={}", frontend, flag));
                }

                cmake_command.spawn()?.wait()?;

                Command::new("ninja")
                    .current_dir(&cmake_path)
                    .spawn()?
                    .wait()?;

                for file in cmake_out_path.read_dir()? {
                    let file = file?;

                    fs::copy(file.path(), &out_path.join(file.file_name())).context("Copy file")?;
                }

                fs::copy(
                    src_path.join("engine").join("cffi").join("kime_engine.h"),
                    out_path.join("kime_engine.h"),
                )
                .context("Copy engine header file")?;

                for res in res_path.read_dir()? {
                    let res = res?;
                    let path = res.path();
                    let ty = res.file_type()?;

                    if !ty.is_file() {
                        continue;
                    }

                    let file_name = path.file_name().unwrap();
                    fs::copy(&path, out_path.join(file_name))?;
                }

                if mode.is_release() {
                    strip_all(&out_path).ok();
                }
            }
        }

        Ok(())
    }
}

fn strip_all(dir: &Path) -> Result<()> {
    for path in dir.read_dir()? {
        let path = path?.path();

        if !is_executable(&path) {
            continue;
        }

        Command::new("strip").arg("-s").arg(path).spawn()?.wait()?;
    }

    Ok(())
}

fn install(src: PathBuf, target: PathBuf) -> Result<()> {
    if src.exists() {
        println!("Install {} into {}", src.display(), target.display());

        Command::new("install")
            .arg(if is_executable(&src) {
                "-Dsm755"
            } else {
                "-Dm644"
            })
            .arg(src)
            .arg("-T")
            .arg(target)
            .spawn()?
            .wait()?;
    }

    Ok(())
}

fn build_core(mode: BuildMode) -> Result<()> {
    Command::new("cargo")
        .args(&["build", "-p=kime-engine-capi"])
        .mode(mode)
        .spawn()?
        .wait()?;

    Ok(())
}

fn main() {
    let args = TaskCommand::from_args();

    args.run().expect("Run command");
}
