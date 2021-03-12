use crate::{HangulState, IconColor, InputCategory};
use std::io;

pub trait OsContext {
    fn read_global_hangul_state(&mut self) -> io::Result<InputCategory>;
    fn update_layout_state(&mut self, category: InputCategory, color: IconColor) -> io::Result<()>;
    fn hanja(&mut self, state: &mut HangulState) -> io::Result<bool>;
    fn emoji(&mut self, state: &mut HangulState) -> io::Result<bool>;
}

#[cfg(unix)]
mod unix {
    use crate::{HangulState, IconColor, InputCategory};
    use std::fs::File;
    use std::process::{Command, Stdio};
    use std::{
        io::{self, BufWriter, Read, Write},
        path::PathBuf,
    };

    pub struct OsContext {
        state_path: PathBuf,
        buf: Vec<u8>,
    }

    fn get_state_dir() -> PathBuf {
        let run_path =
            PathBuf::from(std::env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| "/tmp".into()));
        run_path.join("kime-indicator.state")
    }

    impl Default for OsContext {
        fn default() -> Self {
            Self {
                buf: Vec::with_capacity(64),
                state_path: get_state_dir(),
            }
        }
    }

    impl super::OsContext for OsContext {
        fn read_global_hangul_state(&mut self) -> io::Result<InputCategory> {
            let mut file = File::open(&self.state_path)?;
            let mut buf = [0; 1];
            file.read_exact(&mut buf)?;
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
            let color = match color {
                IconColor::White => "--white",
                IconColor::Black => "--black",
            };
            let category = match category {
                InputCategory::Latin => "--latin",
                InputCategory::Hangul => "--hangul",
            };

            Command::new("kime-indicator")
                .arg(color)
                .arg(category)
                .spawn()?
                .wait()?;

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
    use crate::{InputCategory, IconColor};

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
