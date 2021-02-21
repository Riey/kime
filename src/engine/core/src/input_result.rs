bitflags::bitflags! {
    #[repr(transparent)]
    pub struct InputResult: u32 {
        const CONSUMED = 0b1;
        const LANGUAGE_CHANGED = 0b10;
        const HAS_PREEDIT = 0b100;
        const NEED_RESET = 0b1000;
        const NEED_FLUSH = 0b10000;
    }
}

impl Default for InputResult {
    fn default() -> Self {
        Self::empty()
    }
}
