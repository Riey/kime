use crate::HangulState;
use std::io;

pub trait OsContext {
    fn read_global_hangul_state(&mut self) -> io::Result<bool>;
    fn update_layout_state(&mut self, state: bool) -> io::Result<()>;
    fn hanja(&mut self, state: &mut HangulState) -> io::Result<bool>;
    fn emoji(&mut self, state: &mut HangulState) -> io::Result<bool>;
}

#[cfg(unix)]
mod unix {
    use crate::HangulState;
    use std::io::{self, Read, Write};
    use std::net::Shutdown;
    use std::os::unix::net::UnixStream;
    use std::process;

    pub struct OsContext {
        buf: Vec<u8>,
    }

    impl Default for OsContext {
        fn default() -> Self {
            Self {
                buf: Vec::with_capacity(64),
            }
        }
    }

    impl super::OsContext for OsContext {
        fn read_global_hangul_state(&mut self) -> io::Result<bool> {
            let mut stream = UnixStream::connect("/tmp/kime_window.sock")?;
            stream.write_all(b"l")?;
            let len = stream.read_to_end(&mut self.buf)?;
            let data = &self.buf[..len];
            let ret = data == b"han";
            self.buf.clear();
            Ok(ret)
        }

        fn update_layout_state(&mut self, state: bool) -> io::Result<()> {
            let mut stream = UnixStream::connect("/tmp/kime_window.sock")?;
            stream.write_all(if state { b"ihan" } else { b"ieng" })?;

            Ok(())
        }

        fn hanja(&mut self, state: &mut HangulState) -> io::Result<bool> {
            let mut stream = UnixStream::connect("/tmp/kime_window.sock")?;
            let hangul = state.preedit_str();
            stream.write_all(format!("h{}", hangul).as_bytes())?;
            stream.flush()?;
            stream.shutdown(Shutdown::Write)?;
            let len = stream.read_to_end(&mut self.buf)?;

            if len == 0 {
                Ok(false)
            } else {
                let hanja = std::str::from_utf8(&self.buf[..len])
                    .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
                state.pass_replace(hanja);
                self.buf.clear();

                Ok(true)
            }
        }

        fn emoji(&mut self, state: &mut HangulState) -> io::Result<bool> {
            let mut rofimoji = process::Command::new("rofimoji")
                .arg("--action")
                .arg("print")
                .stdout(process::Stdio::piped())
                .spawn()?;

            let len = rofimoji.stdout.as_mut().unwrap().read_to_end(&mut self.buf)?;

            let exit = rofimoji.wait()?;

            if !exit.success() || len == 0 {
                Ok(false)
            } else {
                let emoji = std::str::from_utf8(&self.buf[..len])
                    .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
                state.pass(emoji.trim_end_matches('\n'));
                self.buf.clear();

                Ok(true)
            }
        }
    }
}

mod fallback {
    use std::io;

    #[derive(Default)]
    pub struct OsContext;

    impl super::OsContext for OsContext {
        fn read_global_hangul_state(&mut self) -> io::Result<bool> {
            Err(io::Error::new(io::ErrorKind::Other, "Unsupported platform"))
        }

        fn update_layout_state(&mut self, _state: bool) -> io::Result<()> {
            Err(io::Error::new(io::ErrorKind::Other, "Unsupported platform"))
        }

        fn hanja(&mut self, _state: &mut crate::HangulState) -> io::Result<bool> {
            Err(io::Error::new(io::ErrorKind::Other, "Unsupported platform"))
        }

        fn emoji(&mut self, _state: &mut crate::HangulState) -> io::Result<bool> {
            Err(io::Error::new(io::ErrorKind::Other, "Unsupported platform"))
        }
    }
}

#[cfg(unix)]
use unix as imp;

#[cfg(not(unix))]
use fallback as imp;

pub use imp::OsContext as DefaultOsContext;
