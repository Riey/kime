mod dict {
    include!(concat!(env!("OUT_DIR"), "/dict.rs"));
}

pub use dict::UnicodeAnnotation;

#[cfg(test)]
mod tests {
    #[test]
    fn simple() {
        assert_eq!(crate::lookup('가').unwrap()[0].0, '家');
    }

    #[test]
    fn symbol_alpha() {
        assert_eq!(crate::lookup_math_symbol("alpha"), Some("α"));
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

pub fn lookup_math_symbol(keyword: &str) -> Option<&'static str> {
    crate::dict::MATH_SYMBOL_ENTRIES
        .binary_search_by_key(&keyword, |(k, _)| *k)
        .ok()
        .map(|idx| crate::dict::MATH_SYMBOL_ENTRIES[idx].1)
}

pub fn search_unicode_annotations(keyword: &str) -> impl Iterator<Item = UnicodeAnnotation> + '_ {
    crate::dict::UNICODE_ANNOTATIONS
        .iter()
        .copied()
        .filter(move |annotation| annotation.tts.contains(keyword))
}
