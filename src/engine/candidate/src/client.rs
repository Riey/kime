use nix::poll;
use std::fmt;
use std::io::{self, BufWriter, Write};
use std::os::unix::io::{AsRawFd, RawFd};
use std::process::{Child, Stdio};

pub const CANDIDATE_PROCESS_NAME: &str = "kime-candidate-window";

pub struct Client {
    child: Child,
    stdout_fd: RawFd,
}

impl Client {
    pub fn new(candidate_list: &[(&str, &str)]) -> io::Result<Self> {
        Self::with_exe_path(CANDIDATE_PROCESS_NAME, candidate_list)
    }

    pub fn with_exe_path(path: &str, candidate_list: &[(&str, &str)]) -> io::Result<Self> {
        let mut child = std::process::Command::new(path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()?;

        let mut stdin = BufWriter::new(child.stdin.take().unwrap());

        for (key, value) in candidate_list {
            stdin.write_all(key.as_bytes())?;
            stdin.write_all(b"\n")?;
            stdin.write_all(value.as_bytes())?;
            stdin.write_all(b"\n")?;
        }

        stdin.flush()?;

        drop(stdin);

        let stdout_fd = child.stdout.as_ref().unwrap().as_raw_fd();

        Ok(Self { stdout_fd, child })
    }

    pub fn is_ready(&self) -> bool {
        let fds = &mut [poll::PollFd::new(self.stdout_fd, poll::PollFlags::POLLIN)];
        poll::poll(fds, 200) == Ok(1)
    }

    pub fn close(mut self) -> io::Result<Option<String>> {
        if self.is_ready() {
            Ok(String::from_utf8(self.child.wait_with_output()?.stdout).ok())
        } else {
            self.child.kill()?;
            Ok(None)
        }
    }
}

impl fmt::Debug for Client {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "CandidateClient")
    }
}
