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
}

impl TaskCommand {
    pub fn run(self) {
        match self {
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

                if build_xim {
                    Command::new("cargo")
                        .args(&["build", "--bin=kime-xim", mode.cargo_profile()])
                        .spawn()
                        .expect("Spawn cargo")
                        .wait()
                        .expect("Run cargo");

                    std::fs::copy(
                        src_path.join(mode.cargo_target_dir()).join("kime-xim"),
                        &out_path.join("kime-xim"),
                    )
                    .expect("Copy xim file");
                }

                let mut cmake_command = Command::new("cmake");

                cmake_command
                    .current_dir(&cmake_path)
                    .arg(src_path)
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
            }
        }
    }
}

fn main() {
    let args = TaskCommand::from_args();

    args.run();
}
