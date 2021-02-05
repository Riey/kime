use anyhow::{Context, Result};
use is_executable::is_executable;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::{Command, ExitStatus};
use std::{collections::HashMap, path::Path};
use structopt::StructOpt;
use strum::IntoEnumIterator;

trait ExitStatusExt: Sized {
    fn assert_success(self);
}

impl ExitStatusExt for ExitStatus {
    fn assert_success(self) {
        assert!(self.success(), "Command run failed");
    }
}

trait CommandExt {
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
#[structopt(about = "Tool for build, test, deploy kime")]
enum TaskCommand {
    #[structopt(about = "Build kime with given frontends")]
    Build {
        #[structopt(
            long,
            parse(try_from_str),
            default_value = "Release",
            help = "Build mode: [Debug, Release]"
        )]
        mode: BuildMode,
        #[structopt(
            parse(try_from_str),
            help = "Select frontend availiable list: [XIM, WAYLAND, QT5, QT6, GTK2, GTK3, GTK4]"
        )]
        frontends: Vec<Frontend>,
    },
    #[structopt(about = "Install kime files into given path")]
    Install {
        #[structopt(parse(from_os_str), help = "Path to install files")]
        target_path: PathBuf,
    },
    #[structopt(about = "Make deb file into given path")]
    ReleaseDeb {
        #[structopt(parse(from_os_str), help = "Path to write deb file")]
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
                    .wait()?
                    .assert_success();
            }
            TaskCommand::Install { target_path } => {
                let out_path = build_path.join("out");

                install(
                    out_path.join("kime-indicator"),
                    target_path.join("usr/bin/kime-indicator"),
                )?;
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
                    out_path.join("default_config.yaml"),
                    target_path.join("etc/kime/config.yaml"),
                )?;
                install(
                    out_path.join("kime-eng-64x64.png"),
                    target_path.join("usr/share/kime/kime-eng-64x64.png"),
                )?;
                install(
                    out_path.join("kime-han-64x64.png"),
                    target_path.join("usr/share/kime/kime-han-64x64.png"),
                )?;
            }
            TaskCommand::Build { frontends, mode } => {
                let cargo_args = env::var("KIME_CARGO_ARGS")
                    .map(|args| args.split(" ").map(ToString::to_string).collect::<Vec<_>>())
                    .unwrap_or_default();
                let cmake_args = env::var("KIME_CMAKE_ARGS")
                    .map(|args| args.split(" ").map(ToString::to_string).collect::<Vec<_>>())
                    .unwrap_or_default();
                let ninja_args = env::var("KIME_NINJA_ARGS")
                    .map(|args| args.split(" ").map(ToString::to_string).collect::<Vec<_>>())
                    .unwrap_or_default();
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

                let mut cargo_projects = vec![
                    ("kime-indicator", "kime-indicator"),
                    ("kime-engine-capi", "libkime_engine.so"),
                ];

                let mut cargo = Command::new("cargo");

                cargo.arg("build").mode(mode);

                for arg in cargo_args {
                    cargo.arg(arg);
                }

                if frontends[&Frontend::Xim] {
                    cargo_projects.push(("kime-xim", "kime-xim"));
                }

                if frontends[&Frontend::Wayland] {
                    cargo_projects.push(("kime-wayland", "kime-wayland"));
                }

                for (package, _binary) in cargo_projects.iter().copied() {
                    cargo.arg("-p").arg(package);
                }

                cargo.spawn()?.wait()?.assert_success();

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

                for arg in cmake_args {
                    cmake_command.arg(arg);
                }

                cmake_command.spawn()?.wait()?.assert_success();

                let mut ninja = Command::new("ninja");

                ninja.current_dir(&cmake_path);

                for arg in ninja_args {
                    ninja.arg(arg);
                }

                ninja.spawn()?.wait()?.assert_success();

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
                    strip_all(&out_path)?;
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

        Command::new("strip")
            .arg("-s")
            .arg(path)
            .spawn()?
            .wait()?
            .assert_success();
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
            .wait()?
            .assert_success();
    }

    Ok(())
}

fn main() {
    let args = TaskCommand::from_args();

    args.run().expect("Run command");
}
