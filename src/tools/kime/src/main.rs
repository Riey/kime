use kime_engine_core::{load_raw_config_from_config_dir, DaemonModule as Module};
use nix::fcntl::{Flock, FlockArg};
use nix::sys::signal::{kill, Signal};
use nix::unistd::{daemon, Pid};
use std::os::unix::io::AsRawFd;
use std::sync::atomic::{AtomicBool, Ordering::SeqCst};
use std::{fs::File, io::Write, path::Path};
use std::{
    io,
    process::{Command, Stdio},
};

const fn process_name(module: Module) -> &'static str {
    match module {
        Module::Xim => "kime-xim",
        Module::Wayland => "kime-wayland",
        Module::Indicator => "kime-indicator",
    }
}

fn kill_daemon(pid_path: &Path) -> io::Result<()> {
    let pid_str = std::fs::read_to_string(pid_path)?;
    let pid: i32 = match pid_str.trim().parse() {
        Ok(pid) => pid,
        Err(err) => {
            log::error!("kill return: {}", err);
            return Err(io::Error::new(io::ErrorKind::Other, "kill command failed"));
        }
    };

    match kill(Pid::from_raw(pid), Signal::SIGTERM) {
        Ok(_) => Ok(()),
        Err(err) => {
            log::error!("kill return: {}", err);
            Err(io::Error::new(io::ErrorKind::Other, "kill command failed"))
        }
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
        let stderr_path = run_dir.join("kime.err");
        let stderr_file = match File::create(&stderr_path) {
            Ok(file) => file,
            Err(err) => {
                log::error!("Can't create stderr file: {}", err);
                return Err(());
            }
        };

        // Daemonize: fork and detach from terminal (noclose=true to keep fds open)
        match daemon(true, true) {
            Ok(_) => {}
            Err(err) => {
                log::error!("Can't daemonize kime: {}", err);
                return Err(());
            }
        }

        // Change working directory to /tmp
        let _ = std::env::set_current_dir("/tmp");

        // Redirect stderr to file
        unsafe {
            nix::libc::dup2(stderr_file.as_raw_fd(), nix::libc::STDERR_FILENO);
        }
    }

    // Create PID file and lock it exclusively to prevent duplicate instances
    // Lock must be held until program exits (like daemonize library behavior)
    // Possible errors:
    //   - File::create fails: permission denied, disk full, etc.
    //   - Flock::lock returns EWOULDBLOCK: another kime instance is running
    //   - Flock::lock returns other error: unexpected lock failure
    let _pid_lock = match File::create(&pid).and_then(|file| {
        Flock::lock(file, FlockArg::LockExclusiveNonblock).map_err(|(_, e)| io::Error::from(e))
    }) {
        Ok(mut lock) => {
            writeln!(lock, "{}", std::process::id()).map_err(|err| {
                log::error!("Can't daemonize kime: {}", err);
            })?;
            Some(lock)
        }
        Err(err) if err.kind() == io::ErrorKind::WouldBlock => {
            log::error!("Another instance of kime daemon is already running.");
            return Err(());
        }
        Err(err) => {
            log::error!("Can't daemonize kime: {}", err);
            return Err(());
        }
    };

    let config = load_raw_config_from_config_dir().daemon;

    static RUN: AtomicBool = AtomicBool::new(true);

    ctrlc::set_handler(|| {
        log::info!("Receive exit signal");
        RUN.store(false, SeqCst);
    })
    .expect("Set ctrlc handler");

    log::info!("Initialized");

    let mut processes = config
        .modules
        .iter()
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
