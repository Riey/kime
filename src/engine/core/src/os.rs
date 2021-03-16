use crate::{IconColor, InputCategory};
use kime_engine_backend::InputEngineBackend;
use std::io;

pub trait OsContext {
    fn read_global_hangul_state(&mut self) -> io::Result<InputCategory>;
    fn update_layout_state(&mut self, category: InputCategory, color: IconColor) -> io::Result<()>;
    fn hanja(
        &mut self,
        engine: &mut impl InputEngineBackend,
        commit_buf: &mut String,
    ) -> io::Result<()>;
    fn emoji(
        &mut self,
        engine: &mut impl InputEngineBackend,
        commit_buf: &mut String,
    ) -> io::Result<()>;
}

#[cfg(unix)]
mod unix {
    use crate::{IconColor, InputCategory};
    use kime_engine_backend::InputEngineBackend;
    use std::process::{Command, Stdio};
    use std::{
        io::{self, BufWriter, Read, Write},
        os::unix::net::UnixStream,
        path::PathBuf,
        time::Duration,
    };

    pub struct OsContext {
        sock_path: PathBuf,
        buf: Vec<u8>,
        buf_str: String,
    }

    fn get_state_dir() -> PathBuf {
        let run_path = kime_run_dir::get_run_dir();
        run_path.join("kime-indicator.sock")
    }

    impl Default for OsContext {
        fn default() -> Self {
            Self {
                buf: Vec::with_capacity(64),
                buf_str: String::with_capacity(16),
                sock_path: get_state_dir(),
            }
        }
    }

    impl super::OsContext for OsContext {
        fn read_global_hangul_state(&mut self) -> io::Result<InputCategory> {
            let mut buf = [0; 2];
            let mut client = UnixStream::connect(&self.sock_path)?;
            client.set_read_timeout(Some(Duration::from_secs(2))).ok();
            client.set_write_timeout(Some(Duration::from_secs(2))).ok();
            client.read_exact(&mut buf)?;
            match buf[0] {
                b'2' => Ok(InputCategory::Math),
                b'1' => Ok(InputCategory::Hangul),
                _ => Ok(InputCategory::Latin),
            }
        }

        fn update_layout_state(
            &mut self,
            category: InputCategory,
            color: IconColor,
        ) -> io::Result<()> {
            let category = match category {
                InputCategory::Math => 2,
                InputCategory::Hangul => 1,
                InputCategory::Latin => 0,
            };
            let color = match color {
                IconColor::Black => 0,
                IconColor::White => 1,
            };

            let mut client = UnixStream::connect(&self.sock_path)?;
            client.set_read_timeout(Some(Duration::from_secs(2))).ok();
            client.set_write_timeout(Some(Duration::from_secs(2))).ok();
            client.write_all(&[category, color])
        }

        fn hanja(
            &mut self,
            engine: &mut impl InputEngineBackend,
            commit_buf: &mut String,
        ) -> io::Result<()> {
            self.buf_str.clear();
            engine.preedit_str(&mut self.buf_str);
            let hangul = self.buf_str.as_str();
            let mut buf = [0; 8];

            for ch in hangul.chars() {
                let hanjas = kime_engine_dict::lookup(ch);

                if hanjas.is_empty() {
                    commit_buf.push(ch);
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
                self.buf.clear();
                let len = stdout.read_to_end(&mut self.buf)?;
                let h = std::str::from_utf8(&self.buf[..len])
                    .ok()
                    .and_then(|l| hanjas.get(l.trim_end_matches('\n').parse::<usize>().ok()?))
                    .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Not valid index"))?
                    .0;

                rofi.wait()?;

                commit_buf.push(h);
            }

            engine.reset();

            Ok(())
        }

        fn emoji(
            &mut self,
            engine: &mut impl InputEngineBackend,
            commit_buf: &mut String,
        ) -> io::Result<()> {
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

            engine.reset();
            commit_buf.push_str(emoji.trim_end_matches('\n'));
            self.buf.clear();
            Ok(())
        }
    }
}

mod fallback {
    use crate::{IconColor, InputCategory};
    use kime_engine_backend::InputEngineBackend;
    use std::io;

    #[derive(Default)]
    pub struct OsContext;

    impl super::OsContext for OsContext {
        fn read_global_hangul_state(&mut self) -> io::Result<InputCategory> {
            Err(io::Error::new(io::ErrorKind::Other, "Unsupported platform"))
        }

        fn update_layout_state(
            &mut self,
            _category: InputCategory,
            _color: IconColor,
        ) -> io::Result<()> {
            Err(io::Error::new(io::ErrorKind::Other, "Unsupported platform"))
        }

        fn hanja(
            &mut self,
            _state: &mut impl InputEngineBackend,
            _commit_buf: &mut String,
        ) -> io::Result<()> {
            Err(io::Error::new(io::ErrorKind::Other, "Unsupported platform"))
        }

        fn emoji(
            &mut self,
            _state: &mut impl InputEngineBackend,
            _commit_buf: &mut String,
        ) -> io::Result<()> {
            Err(io::Error::new(io::ErrorKind::Other, "Unsupported platform"))
        }
    }
}

#[cfg(unix)]
use unix as imp;

#[cfg(not(unix))]
use fallback as imp;

pub use imp::OsContext as DefaultOsContext;
