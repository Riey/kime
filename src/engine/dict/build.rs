#[path = "src/math_symbol_key.rs"]
mod math_symbol_key;

use itertools::Itertools;
use math_symbol_key::*;
use serde::{Deserialize, Deserializer};
use std::{
    collections::BTreeMap,
    env,
    io::{BufWriter, Write},
    mem,
    path::PathBuf,
};
use unic::emoji::char::is_emoji;

#[derive(Default, Debug, Clone, Copy)]
struct HanjaEntry {
    hanja: &'static str,
    description: &'static str,
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
                    "sf" => Style::SF,
                    "bf" => Style::BF,
                    "it" => Style::IT,
                    "tt" => Style::TT,
                    "bb" => Style::BB,
                    "scr" => Style::SCR,
                    "cal" => Style::CAL,
                    "frak" => Style::FRAK,
                    _ => return Err(Error::custom("no matching style name")),
                })
            })
            .fold(Ok(Style::NONE), |sty1, sty2| Ok(sty1? | sty2?));
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

type Dict = BTreeMap<&'static str, Vec<HanjaEntry>>;

fn load_hanja_dict() -> Dict {
    let hanja_data = include_str!("data/hanja.txt");
    let hanja_freq = include_str!("data/freq-hanja.txt");

    let mut freq_dict: BTreeMap<char, u32> = BTreeMap::new();

    for line in hanja_freq.lines() {
        match line.split(':').next_tuple() {
            Some((hanja, freq)) => {
                if let Some(hanja) = hanja.chars().next() {
                    if let Ok(freq) = freq.parse() {
                        freq_dict.insert(hanja, freq);
                    }
                }
            }
            None => continue,
        }
    }

    let mut dict = Dict::new();

    for line in hanja_data.lines() {
        if line.starts_with('#') {
            continue;
        }

        match line.split(':').next_tuple() {
            Some((hangul, hanja, description)) => {
                // skip unused hanja
                if description.is_empty() {
                    continue;
                }

                dict.entry(hangul)
                    .or_default()
                    .push(HanjaEntry { hanja, description });
            }
            None => continue,
        }
    }

    for (_, entries) in dict.iter_mut() {
        entries.sort_by_key(|e| {
            std::cmp::Reverse(
                e.hanja
                    .chars()
                    .map(|c| freq_dict.get(&c).map_or(0, |n| *n))
                    .sum::<u32>(),
            )
        })
    }

    dict
}

fn load_unicode_annotations() -> quick_xml::Result<Vec<UnicodeEntry>> {
    use quick_xml::{events::Event, Reader};

    let mut out = Vec::with_capacity(512);
    let mut current_entry = UnicodeEntry::default();

    let mut reader = Reader::from_str(include_str!("data/en.xml"));

    loop {
        match reader.read_event()? {
            Event::Start(start) if start.name().0 == b"annotation" => {
                let cp = start.attributes().next().unwrap()?;
                debug_assert_eq!(cp.key.0, b"cp");
                let cp = cp.decode_and_unescape_value(&reader)?;
                if current_entry.cp != cp {
                    if !current_entry.cp.is_empty() {
                        out.push(mem::take(&mut current_entry));
                    }

                    current_entry.cp = cp.into_owned();
                    current_entry.description =
                        reader.read_text(start.to_end().name())?.into_owned();
                } else {
                    current_entry.tts = reader.read_text(start.to_end().name())?.into_owned();
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
        "pub static HANJA_ENTRIES: &[(&str, &[(&str, &str)])] = &[",
    )
    .unwrap();

    for (k, values) in load_hanja_dict() {
        write!(out, "(\"{}\", &[", k).unwrap();
        for value in values {
            write!(out, "(\"{}\", \"{}\"),", value.hanja, value.description).unwrap();
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

    writeln!(
        out,
        "pub static MATH_SYMBOL_ENTRIES: &[(SymbolKey, &str)] = &{:?};",
        symbol_map
    )
    .unwrap();

    writeln!(out, "#[derive(Clone, Copy, Debug)] pub struct UnicodeAnnotation {{ pub codepoint: &'static str, pub tts: &'static str, }}").unwrap();
    writeln!(
        out,
        "pub static UNICODE_ANNOTATIONS: &[UnicodeAnnotation] = &["
    )
    .unwrap();
    for entry in load_unicode_annotations().unwrap() {
        if !entry.cp.chars().any(|c| is_emoji(c)) {
            continue;
        }

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
