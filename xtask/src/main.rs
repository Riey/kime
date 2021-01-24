use std::collections::HashSet;
use std::path::PathBuf;
use std::process::Command;
use structopt::StructOpt;

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

    fn cargo_profile(self) -> &'static str {
        match self {
            BuildMode::Debug => "",
            BuildMode::Release => "--release",
        }
    }

    fn cargo_target_dir(self) -> &'static str {
        match self {
            BuildMode::Debug => "target/debug",
            BuildMode::Release => "target/release",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, strum_macros::EnumString, strum_macros::Display)]
enum Frontend {
    #[strum(to_string = "XIM")]
    Xim,
    #[strum(to_string = "QT5")]
    Qt5,
    // Gtk2,
    #[strum(to_string = "GTK3")]
    Gtk3,
    // Gtk4,
}

#[derive(StructOpt)]
enum TaskCommand {
    Test {},
    Build {
        #[structopt(long, parse(try_from_str), default_value = "Release")]
        mode: BuildMode,
        #[structopt(long, parse(from_os_str))]
        build_path: Option<PathBuf>,
        #[structopt(long, parse(from_os_str))]
        src_path: Option<PathBuf>,
        #[structopt(parse(try_from_str))]
        frontends: Vec<Frontend>,
    },
    Install {
        #[structopt(long, parse(from_os_str))]
        out_path: Option<PathBuf>,
        #[structopt(parse(from_os_str))]
        target_path: PathBuf,
    },
    ReleaseDeb {
        #[structopt(long, parse(from_os_str))]
        out_path: Option<PathBuf>,
        #[structopt(parse(from_os_str))]
        target_path: Option<PathBuf>,
    },
}

fn build_core(mode: BuildMode) {
    Command::new("cargo")
        .args(&["build", "--lib=kime_engine_capi", mode.cargo_profile()])
        .spawn()
        .expect("Spawn cargo")
        .wait()
        .expect("Run cargo");
}

fn install(exe: bool, src: PathBuf, target: PathBuf) {
    if src.exists() {
        println!("Install {} into {}", src.display(), target.display());

        Command::new("install")
            .arg(if exe { "-Dsm755" } else { "-Dm644" })
            .arg(src)
            .arg("-T")
            .arg(target)
            .spawn()
            .expect("Spawn install")
            .wait()
            .expect("Run install");
    }
}

impl TaskCommand {
    pub fn run(self) {
        match self {
            TaskCommand::ReleaseDeb {
                out_path,
                target_path,
            } => {
                let target_path = target_path
                    .unwrap_or_else(|| std::env::current_dir().expect("Get current dir"));
                let deb_dir = tempfile::tempdir().expect("Create tempdir");
                let control_path = deb_dir.as_ref().join("DEBIAN/control");
                std::fs::create_dir_all(control_path.parent().unwrap()).expect("Create DEBIAN dir");

                std::fs::write(
                    control_path,
                    include_str!("../control.in").replace("%VER%", env!("CARGO_PKG_VERSION")),
                )
                .expect("Write control");

                // Install into tempdir
                TaskCommand::Install {
                    out_path,
                    target_path: deb_dir.path().into(),
                }
                .run();

                Command::new("dpkg-deb")
                    .arg("--build")
                    .arg(deb_dir.as_ref())
                    .arg(target_path.join(format!("kime_{}_amd64.deb", env!("CARGO_PKG_VERSION"))))
                    .spawn()
                    .expect("Spawn dpkg-deb")
                    .wait()
                    .expect("Run dpkg-deb");
            }
            TaskCommand::Install {
                out_path,
                target_path,
            } => {
                let out_path = out_path.unwrap_or_else(|| {
                    std::env::current_dir()
                        .expect("Load current dir")
                        .join("build")
                        .join("out")
                });

                install(
                    true,
                    out_path.join("kime-xim"),
                    target_path.join("usr/bin/kime-xim"),
                );
                install(
                    true,
                    out_path.join("libkime-gtk3.so"),
                    target_path.join("usr/lib/gtk-3.0/3.0.0/immodules/im-kime.so"),
                );
                install(true, out_path.join("libkime-qt5.so"), target_path.join("usr/lib/qt/plugins/platforminputcontexts/libkimeplatforminputcontextplugin.so"));
                install(
                    true,
                    out_path.join("libkime_engine.so"),
                    target_path.join("usr/lib/libkime_engine.so"),
                );
                install(
                    false,
                    out_path.join("kime_engine.h"),
                    target_path.join("usr/include/kime_engine.h"),
                );
                install(
                    false,
                    out_path.join("config.yaml"),
                    target_path.join("etc/kime/config.yaml"),
                );
            }
            TaskCommand::Test {} => {
                Command::new("cargo")
                    .args(&["test", "-p=kime-engine-core"])
                    .spawn()
                    .expect("Spawn cargo")
                    .wait()
                    .expect("Run test");
            }
            TaskCommand::Build {
                frontends,
                mode,
                build_path,
                src_path,
            } => {
                let mut build_xim = false;
                let mut cmake_flags = HashSet::new();
                let src_path =
                    src_path.unwrap_or_else(|| std::env::current_dir().expect("Load current dir"));
                let build_path = build_path.unwrap_or_else(|| src_path.join("build"));
                let out_path = build_path.join("out");
                let cmake_path = build_path.join("cmake");
                let cmake_out_path = cmake_path.join("lib");

                std::fs::create_dir_all(&out_path).expect("create out_path");
                std::fs::create_dir_all(&cmake_out_path).expect("create cmake_out_path");

                for frontend in frontends.iter() {
                    match frontend {
                        Frontend::Xim => build_xim = true,
                        other => drop(cmake_flags.insert(other)),
                    }
                }

                // build engine
                build_core(mode);

                std::fs::copy(
                    src_path
                        .join(mode.cargo_target_dir())
                        .join("libkime_engine.so"),
                    out_path.join("libkime_engine.so"),
                )
                .expect("Copy engine file");

                if build_xim {
                    Command::new("cargo")
                        .args(&["build", "--bin=kime-xim", mode.cargo_profile()])
                        .env(
                            "RUSTFLAGS",
                            format!("-L{}", src_path.join(mode.cargo_target_dir()).display()),
                        )
                        .spawn()
                        .expect("Spawn cargo")
                        .wait()
                        .expect("Run cargo");

                    std::fs::copy(
                        src_path.join(mode.cargo_target_dir()).join("kime-xim"),
                        out_path.join("kime-xim"),
                    )
                    .expect("Copy xim file");
                }

                let mut cmake_command = Command::new("cmake");

                cmake_command
                    .current_dir(&cmake_path)
                    .arg(&src_path)
                    .arg("-GNinja")
                    .arg(mode.cmake_build_type());

                for flags in cmake_flags.iter() {
                    cmake_command.arg(format!("-DENABLE_{}=ON", flags));
                }

                cmake_command
                    .spawn()
                    .expect("Spawn cmake")
                    .wait()
                    .expect("Run cmake");

                Command::new("ninja")
                    .current_dir(&cmake_path)
                    .spawn()
                    .expect("Spawn ninja")
                    .wait()
                    .expect("Run ninja");

                for file in cmake_out_path.read_dir().expect("Read cmake out") {
                    let file = file.expect("Read entry");

                    std::fs::copy(file.path(), &out_path.join(file.file_name()))
                        .expect("Copy file");
                }

                std::fs::copy(
                    src_path.join("engine").join("cffi").join("kime_engine.h"),
                    out_path.join("kime_engine.h"),
                )
                .expect("Copy engine header file");

                serde_yaml::to_writer(
                    std::fs::File::create(out_path.join("config.yaml"))
                        .expect("Create config file"),
                    &kime_engine_core::RawConfig::default(),
                )
                .expect("Write config file");
            }
        }
    }
}

fn main() {
    let args = TaskCommand::from_args();

    args.run();
}
