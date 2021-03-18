pub mod math_symbol_key;
mod dict {
    include!(concat!(env!("OUT_DIR"), "/dict.rs"));
}

pub use dict::UnicodeAnnotation;
use math_symbol_key::*;

#[cfg(test)]
mod tests {
    #[test]
    fn simple() {
        assert_eq!(crate::lookup('가').unwrap()[0].0, '家');
    }

    #[test]
    fn math_symbols() {
        use crate::lookup_math_symbol;
        use crate::math_symbol_key::*;

        assert_eq!(lookup_math_symbol("alpha", STYLE_NONE), Some("α"));
        assert_eq!(lookup_math_symbol("alpha", STYLE_BF), Some("𝛂"));
        assert_eq!(lookup_math_symbol("alpha", STYLE_IT), Some("𝛼"));
        assert_eq!(lookup_math_symbol("alpha", STYLE_BF | STYLE_IT), Some("𝜶"));

        assert_eq!(lookup_math_symbol("R", STYLE_SF | STYLE_BF | STYLE_IT), Some("𝙍"));
        assert_eq!(lookup_math_symbol("R", STYLE_TT), Some("𝚁"));
        assert_eq!(lookup_math_symbol("R", STYLE_BB), Some("ℝ"));
        assert_eq!(lookup_math_symbol("R", STYLE_SCR), Some("ℛ"));
        assert_eq!(lookup_math_symbol("R", STYLE_CAL), Some("𝓡"));
        assert_eq!(lookup_math_symbol("R", STYLE_FRAK), Some("ℜ"));
    }

    #[test]
    fn unicode() {
        assert_eq!(
            crate::search_unicode_annotations("thinkin")
                .next()
                .unwrap()
                .codepoint,
            "🤔"
        );
    }
}

pub fn lookup(hangul: char) -> Option<&'static [(char, &'static str)]> {
    crate::dict::HANJA_ENTRIES
        .binary_search_by_key(&hangul, |(k, _)| *k)
        .ok()
        .map(|idx| crate::dict::HANJA_ENTRIES[idx].1)
}

pub fn lookup_math_symbol(keyword: &str, style: Style) -> Option<&'static str> {
    let key = SymbolKey(keyword, style);
    crate::dict::MATH_SYMBOL_ENTRIES
        .binary_search_by_key(&key, |(k, _)| *k)
        .ok()
        .map(|idx| crate::dict::MATH_SYMBOL_ENTRIES[idx].1)
}

pub fn search_unicode_annotations(keyword: &str) -> impl Iterator<Item = UnicodeAnnotation> + '_ {
    crate::dict::UNICODE_ANNOTATIONS
        .iter()
        .copied()
        .filter(move |annotation| annotation.tts.contains(keyword))
}
