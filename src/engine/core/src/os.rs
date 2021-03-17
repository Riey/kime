use crate::{IconColor, InputCategory};
use kime_engine_backend::InputEngineBackend;
use std::io;

pub trait OsContext {
    fn read_global_hangul_state(&mut self) -> io::Result<InputCategory>;
    fn update_layout_state(&mut self, category: InputCategory, color: IconColor) -> io::Result<()>;
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
        io::{self, Read, Write},
        os::unix::net::UnixStream,
        path::PathBuf,
        time::Duration,
    };

    pub struct OsContext {
        sock_path: PathBuf,
        buf: Vec<u8>,
    }

    fn get_state_dir() -> PathBuf {
        let run_path = kime_run_dir::get_run_dir();
        run_path.join("kime-indicator.sock")
    }

    impl Default for OsContext {
        fn default() -> Self {
            Self {
                buf: Vec::with_capacity(64),
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
