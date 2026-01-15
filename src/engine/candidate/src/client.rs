use nix::poll::{self, PollFd, PollFlags, PollTimeout};
use std::fmt;
use std::io::{self, BufWriter, Write};
use std::os::unix::io::{AsFd, BorrowedFd};
use std::process::{Child, Stdio};

pub const CANDIDATE_PROCESS_NAME: &str = "kime-candidate-window";

pub struct Client {
    child: Child,
}

impl Client {
    pub fn new(candidate_list: &[(&str, &str)]) -> io::Result<Self> {
        let mut child = std::process::Command::new(CANDIDATE_PROCESS_NAME)
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

        Ok(Self { child })
    }

    pub fn is_ready(&self) -> bool {
        let stdout: BorrowedFd = self.child.stdout.as_ref().unwrap().as_fd();
        let fds = &mut [PollFd::new(stdout, PollFlags::POLLIN)];
        poll::poll(fds, PollTimeout::from(200u16)) == Ok(1)
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
