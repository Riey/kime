mod dict {
    include!(concat!(env!("OUT_DIR"), "/dict.rs"));
}

#[cfg(test)]
mod tests {
    #[test]
    fn simple() {
        dbg!(&crate::dict::DICT[&'가']);
        assert_eq!(crate::lookup('가')[0].0, '家');
    }
}

pub fn lookup(hangul: char) -> &'static [(char, &'static str)] {
    crate::dict::DICT.get(&hangul).copied().unwrap_or(&[])
}
