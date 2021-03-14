use itertools::Itertools;
use phf_codegen::Map;
use serde::Deserialize;
use std::{collections::BTreeMap, env, io::Write, path::PathBuf};

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

#[derive(Deserialize)]
struct KeySymPair {
    keyword: String,
    symbol: String,
}

fn main() {
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

    let mut out =
        std::fs::File::create(PathBuf::from(env::var("OUT_DIR").unwrap()).join("dict.rs")).unwrap();

    let mut map = Map::new();
    let mut entries = String::new();
    for (k, mut values) in dict {
        entries.push_str("&[");
        values.sort_unstable_by_key(|x| x.ty);
        for value in values {
            if value.hanja == '\0' {
                continue;
            }
            use std::fmt::Write;
            write!(entries, "('{}', \"{}\"),", value.hanja, value.definition).unwrap();
        }
        entries.push_str("]");
        map.entry(k, &entries);
        entries.clear();
    }

    writeln!(
        out,
        "pub static DICT: phf::Map<char, &[(char, &str)]> = {};",
        map.build()
    )
    .unwrap();

    let symbol_map = include_str!("data/symbol_map.json");
    let symbol_map: Vec<KeySymPair> = serde_json::from_str(symbol_map).unwrap();

    let mut map = Map::new();
    let mut entries = String::new();
    for pair in &symbol_map {
        let keyword = &pair.keyword;
        entries.push_str("\"");
        entries.push_str(&pair.symbol);
        entries.push_str("\"");
        map.entry(keyword.as_str(), &entries);
        entries.clear();
    }
    writeln!(
        out,
        "pub static SYMBOL_DICT: phf::Map<&'static str, &'static str> = {};",
        map.build()
    )
    .unwrap();
}
