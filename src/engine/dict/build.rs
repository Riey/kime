#[path = "src/math_symbol_key.rs"]
mod math_symbol_key;

use math_symbol_key::*;
use itertools::Itertools;
use serde::{Deserialize,Deserializer};
use std::{
    collections::BTreeMap,
    env,
    io::{BufWriter, Write},
    mem,
    path::PathBuf,
};

fn parse_unich(ch: &str) -> char {
    let ch = ch.strip_prefix("U+").unwrap();
    std::char::from_u32(u32::from_str_radix(ch, 16).unwrap()).unwrap()
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone, Copy)]
enum EntryType {
    E,
    N,
    X,
    Empty,
}

impl Default for EntryType {
    fn default() -> Self {
        Self::Empty
    }
}

#[derive(Default, Debug, Clone, Copy)]
struct HanjaEntry {
    hanja: char,
    definition: &'static str,
    ty: EntryType,
}

#[derive(Default, Debug, Clone)]
struct UnicodeEntry {
    cp: String,
    description: String,
    tts: String,
}

impl<'de> Deserialize<'de> for Style {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        use serde::de::Error;

        let styles: Vec<&str> = Deserialize::deserialize(deserializer)?;
        let style = styles
            .into_iter()
            .map(|s| {
                Ok(match s {
                    "sf" => STYLE_SF,
                    "bf" => STYLE_BF,
                    "it" => STYLE_IT,
                    "tt" => STYLE_TT,
                    "bb" => STYLE_BB,
                    "scr" => STYLE_SCR,
                    "cal" => STYLE_CAL,
                    "frak" => STYLE_FRAK,
                    _ => return Err(Error::custom("no matching style name")),
                })
            })
            .fold(Ok(STYLE_NONE), |sty1, sty2| Ok(sty1? | sty2?));
        style
    }
}

#[derive(Deserialize)]
struct StySymPair<'a> {
    style: Style,
    symbol: &'a str,
}

#[derive(Deserialize)]
struct KeySymPair<'a> {
    keyword: &'a str,
    symbols: Vec<StySymPair<'a>>,
}

fn load_hanja_dict() -> BTreeMap<char, Vec<HanjaEntry>> {
    let mut dict: BTreeMap<char, Vec<HanjaEntry>> = BTreeMap::new();
    let unihan = include_str!("data/Unihan_Readings.txt");
    let mut entry = HanjaEntry::default();
    for line in unihan.lines() {
        if line.starts_with('#') {
            continue;
        }

        let line = line.trim_end();

        if let Some((ch, field, data)) = line.split('\t').next_tuple::<(&str, &str, &str)>() {
            let ch = parse_unich(ch);

            match field {
                "kDefinition" => {
                    entry.hanja = ch;
                    entry.definition = data;
                }
                "kHangul" => {
                    if entry.definition.is_empty() || entry.hanja == '\0' {
                        continue;
                    }
                    for data in data.split(' ') {
                        let (hangul, ty) = data.split(':').next_tuple::<(&str, &str)>().unwrap();

                        entry.ty = match ty.as_bytes().last().unwrap() {
                            b'E' => EntryType::E,
                            b'N' => EntryType::N,
                            b'X' => EntryType::X,
                            _ => EntryType::Empty,
                        };

                        dict.entry(hangul.chars().next().unwrap())
                            .or_default()
                            .push(std::mem::take(&mut entry));
                    }
                }
                _ => {}
            }
        }
    }

    dict
}

fn load_unicode_annotations() -> quick_xml::Result<Vec<UnicodeEntry>> {
    use quick_xml::{events::Event, Reader};

    let mut out = Vec::with_capacity(512);
    let mut buf = Vec::with_capacity(512);
    let mut current_entry = UnicodeEntry::default();

    let mut reader = Reader::from_str(include_str!("data/en.xml"));

    loop {
        match reader.read_event(&mut buf)? {
            Event::Start(start) if start.name() == b"annotation" => {
                let cp = start.attributes().next().unwrap()?;
                debug_assert_eq!(cp.key, b"cp");
                let cp = cp.unescape_and_decode_value(&reader)?;
                if current_entry.cp != cp {
                    if !current_entry.cp.is_empty() {
                        out.push(mem::take(&mut current_entry));
                    }

                    current_entry.cp = cp;
                    current_entry.description = reader.read_text(b"annotation", &mut buf)?;
                } else {
                    current_entry.tts = reader.read_text(b"annotation", &mut buf)?;
                }
            }
            Event::Eof => break,
            _ => {}
        }
    }

    out.push(mem::take(&mut current_entry));
    Ok(out)
}

fn main() {
    let mut out = BufWriter::new(
        std::fs::File::create(PathBuf::from(env::var("OUT_DIR").unwrap()).join("dict.rs")).unwrap(),
    );

    writeln!(out, "use crate::math_symbol_key::*;").unwrap();
    writeln!(
        out,
        "pub static HANJA_ENTRIES: &[(char, &[(char, &str)])] = &[",
    )
    .unwrap();

    for (k, mut values) in load_hanja_dict() {
        write!(out, "('{}', &[", k).unwrap();
        values.sort_unstable_by_key(|x| x.ty);
        for value in values {
            if value.hanja == '\0' {
                continue;
            }
            write!(out, "('{}', \"{}\"),", value.hanja, value.definition).unwrap();
        }
        writeln!(out, "]),").unwrap();
    }

    writeln!(out, "];").unwrap();

    let symbol_map_data = include_str!("data/symbol_map.json");
    let symbol_map_data: Vec<KeySymPair> = serde_json::from_str(symbol_map_data).unwrap();
    let mut symbol_map: Vec<(SymbolKey, &str)> = Vec::new();
    for key_sym_pair in &symbol_map_data {
        let keyword = &key_sym_pair.keyword;
        for sty_sym_pair in &key_sym_pair.symbols {
            let style = sty_sym_pair.style;
            let symbol = sty_sym_pair.symbol;
            symbol_map.push((SymbolKey(keyword, style), symbol));
        }
    }
    symbol_map.sort_unstable_by_key(|pair| pair.0);

    writeln!(out, "pub static MATH_SYMBOL_ENTRIES: &[(SymbolKey, &str)] = &{:?};", symbol_map).unwrap();

    writeln!(out, "#[derive(Clone, Copy, Debug)] pub struct UnicodeAnnotation {{ pub codepoint: &'static str, pub tts: &'static str, }}").unwrap();
    writeln!(
        out,
        "pub static UNICODE_ANNOTATIONS: &[UnicodeAnnotation] = &["
    )
    .unwrap();
    for entry in load_unicode_annotations().unwrap() {
        writeln!(
            out,
            "UnicodeAnnotation {{ codepoint: \"{}\", tts: \"{}\" }},",
            entry.cp, entry.tts
        )
        .unwrap()
    }
    writeln!(out, "];").unwrap();

    out.flush().unwrap();
}
