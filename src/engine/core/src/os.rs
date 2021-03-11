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
    use std::io::{self, BufWriter, Read, Write};
    use std::os::unix::net::UnixStream;
    use std::process::{Command, Stdio};

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
            let hangul = state.preedit_str();
            let mut hanja = String::with_capacity(hangul.len());
            let mut buf = [0; 8];

            for ch in hangul.chars() {
                let hanjas = kime_engine_dict::lookup(ch);

                if hanjas.is_empty() {
                    hanja.push(ch);
                    continue;
                }

                let mut rofi = Command::new("rofi")
                    .arg("-dmenu")
                    .arg("-i")
                    .arg("-format")
                    .arg("i")
                    .stdin(Stdio::piped())
                    .stdout(Stdio::piped())
                    .spawn()?;

                let mut stdin = BufWriter::new(rofi.stdin.take().unwrap());

                for (hanja, definition) in hanjas.iter().copied() {
                    stdin.write_all(hanja.encode_utf8(&mut buf[..]).as_bytes())?;
                    stdin.write_all(b": ")?;
                    stdin.write_all(definition.as_bytes())?;
                    stdin.write_all(b"\n")?;
                }

                stdin.flush()?;

                let mut stdout = rofi.stdout.take().unwrap();
                let len = stdout.read_to_end(&mut self.buf)?;
                let h = std::str::from_utf8(&self.buf[..len])
                    .ok()
                    .and_then(|l| hanjas.get(l.trim_end_matches('\n').parse::<usize>().ok()?))
                    .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Not valid index"))?
                    .0;

                rofi.wait()?;

                hanja.push(h);
            }

            state.pass_replace(&hanja);
            self.buf.clear();

            Ok(true)
        }

        fn emoji(&mut self, state: &mut HangulState) -> io::Result<bool> {
            let mut rofimoji = Command::new("rofimoji")
                .arg("--action")
                .arg("print")
                .stdout(Stdio::piped())
                .spawn()?;

            let mut stdout = rofimoji.stdout.take().unwrap();
            let len = stdout.read_to_end(&mut self.buf)?;
            let emoji = std::str::from_utf8(&self.buf[..len])
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

            rofimoji.wait()?;

            state.pass(emoji.trim_end_matches('\n'));
            self.buf.clear();
            Ok(true)
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
