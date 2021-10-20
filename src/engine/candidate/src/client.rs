use crate::shared::{InitRequest, Request, Response, CANDIDATE_PROCESS_NAME};
use serde::Serialize;
use serde_json::{from_slice, to_writer};
use std::fmt;
use std::io::{self, BufRead, BufReader, Write};
use std::process::{Child, ChildStdin, ChildStdout, Stdio};
use std::time::{Duration, Instant};
use timeout_readwrite::{TimeoutReadExt, TimeoutReader, TimeoutWriteExt, TimeoutWriter};

pub struct Client {
    child: Child,
    buf: Vec<u8>,
    stdout: BufReader<TimeoutReader<ChildStdout>>,
    stdin: TimeoutWriter<ChildStdin>,
}

impl Client {
    pub fn new(candidate_list: &[(&str, &str)]) -> io::Result<Self> {
        let req = InitRequest {
            candidate_list: candidate_list
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
        };
        let mut child = std::process::Command::new(CANDIDATE_PROCESS_NAME)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()?;
        let mut client = Self {
            buf: Vec::with_capacity(4096),
            stdout: BufReader::new(
                child
                    .stdout
                    .take()
                    .unwrap()
                    .with_timeout(Duration::from_secs(1)),
            ),
            stdin: child
                .stdin
                .take()
                .unwrap()
                .with_timeout(Duration::from_secs(1)),
            child,
        };

        client.send_msg(&req)?;

        Ok(client)
    }

    fn send_msg(&mut self, req: &impl Serialize) -> io::Result<()> {
        self.buf.clear();
        to_writer(&mut self.buf, req)?;
        self.buf.push(b'\n');
        self.stdin.write(&self.buf)?;

        Ok(())
    }

    fn recv_msg(&mut self) -> io::Result<Response> {
        self.buf.clear();
        self.stdout.read_until(b'\n', &mut self.buf)?;
        from_slice(&self.buf).map_err(Into::into)
    }

    pub fn try_recv_msg(&mut self) -> io::Result<Option<Response>> {
        match self.recv_msg() {
            Ok(res) => Ok(Some(res)),
            Err(err) if err.kind() == io::ErrorKind::TimedOut => Ok(None),
            Err(err) => Err(err),
        }
    }

    pub fn close(&mut self) -> io::Result<Option<String>> {
        if let Ok(res) = self.try_recv_msg() {
            self.child.kill().ok();
            return Ok(match res {
                None | Some(Response::Quit) => None,
                Some(Response::Selected(s)) => Some(s),
            });
        }

        self.send_msg(&Request::Close)?;

        let exit = Instant::now() + Duration::from_secs(1);

        while Instant::now() < exit {
            if let Some(_exit_status) = self.child.try_wait()? {
                break;
            }

            std::thread::sleep(Duration::from_millis(100));
        }

        self.child.kill()?;

        Ok(None)
    }
}

impl fmt::Debug for Client {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "CandidateClient")
    }
}
