use daemonize::Daemonize;
use kime_config::{Module, RawConfig};
use std::sync::atomic::{AtomicBool, Ordering::SeqCst};
use std::{
    env, io,
    process::{Command, Stdio},
};
use std::{fs::File, path::Path};

const fn process_name(module: Module) -> &'static str {
    match module {
        Module::Xim => "kime-xim",
        Module::Wayland => "kime-wayland",
        Module::Indicator => "kime-indicator",
    }
}

fn kill_daemon(pid: &Path) -> io::Result<()> {
    let pid = std::fs::read_to_string(pid)?;

    let ret = Command::new("kill")
        .arg(pid)
        .spawn()?
        .wait_with_output()?
        .status;

    if ret.success() {
        Ok(())
    } else {
        log::error!("kill return: {}", ret);
        Err(io::Error::new(io::ErrorKind::Other, "kill command failed"))
    }
}

fn main() -> Result<(), ()> {
    let mut args = kime_version::cli_boilerplate!(
        Ok(()),
        "-k or --kill: kill daemon then exit",
        "-D or --no-daemon: don't start as daemon",
    );

    let run_dir = kime_run_dir::get_run_dir();
    let pid = run_dir.join("kime.pid");

    if args.contains(["-k", "--kill"]) {
        return kill_daemon(&pid).map_err(|err| {
            log::error!("Can't kill daemon: {}", err);
        });
    }

    if !args.contains(["-D", "--no-daemon"]) {
        let stderr = run_dir.join("kime.err");
        let stderr_file = match File::create(stderr) {
            Ok(file) => file,
            Err(err) => {
                log::error!("Can't create stderr file: {}", err);
                return Err(());
            }
        };
        match Daemonize::new()
            .working_directory("/tmp")
            .stderr(stderr_file)
            .pid_file(&pid)
            .start()
        {
            Ok(_) => {}
            Err(err) => {
                log::error!("Can't daemonize kime: {}", err);
                return Err(());
            }
        }
    }

    let dir = xdg::BaseDirectories::with_prefix("kime").map_err(|err| {
        log::error!("Can't get xdg dirs: {}", err);
        ()
    })?;
    let config = match dir.find_config_file("config.yaml") {
        Some(path) => {
            let config: RawConfig =
                serde_yaml::from_reader(File::open(path).expect("Can't open config file"))
                    .expect("Can't read config file");
            config.daemon
        }
        None => {
            log::warn!("Can't find config file use default config");
            Default::default()
        }
    };

    static RUN: AtomicBool = AtomicBool::new(true);

    ctrlc::set_handler(|| {
        log::info!("Receive exit signal");
        RUN.store(false, SeqCst);
    })
    .expect("Set ctrlc handler");

    log::info!("Initialized");

    let mut processes = config
        .modules
        .into_iter()
        .filter_map(|module| {
            let name = process_name(module);
            match Command::new(name)
                .stdin(Stdio::null())
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit())
                .spawn()
            {
                Ok(p) => Some((name, p, false)),
                Err(err) => {
                    log::error!("Can't spawn {}: {}", name, err);
                    None
                }
            }
        })
        .collect::<Vec<_>>();

    while RUN.load(SeqCst) {
        // Remove finished process
        for (name, process, exited) in processes.iter_mut() {
            match process.try_wait().expect("Wait process") {
                Some(status) => {
                    log::info!("Process {} has exit with {}", name, status);
                    *exited = true;
                }
                None => {}
            }
        }

        processes.retain(|(_, _, exited)| !*exited);

        if processes.is_empty() {
            log::info!("All process has exited");
            return Ok(());
        }

        std::thread::sleep(std::time::Duration::from_secs(1));
    }

    for (name, mut process, _) in processes {
        log::info!("KILL {}", name);
        process.kill().ok();
    }

    Ok(())
}
