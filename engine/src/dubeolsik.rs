pub const DUBEOLSIK_LAYOUT: &str = include_str!("../data/dubeolsik.yaml");

#[cfg(test)]
mod tests {
    use crate::{InputEngine, InputResult, Layout};

    use xkbcommon::xkb;

    use pretty_assertions::assert_eq;

    #[track_caller]
    fn test_input(inputs: &[(xkb::Keysym, InputResult)]) {
        let mut engine = InputEngine::new(Layout::dubeolsik());

        engine.enable_hangul = true;

        for (sym, expect_result) in inputs.iter().copied() {
            assert_eq!(expect_result, engine.press_key_sym(sym));
        }
    }

    #[test]
    fn esc() {
        test_input(&[
            (xkb::KEY_r, InputResult::Preedit('ㄱ')),
            (xkb::KEY_Escape, InputResult::CommitBypass('ㄱ')),
            (xkb::KEY_r, InputResult::Commit('r')),
        ]);
    }

    #[test]
    fn issue_28() {
        test_input(&[
            (xkb::KEY_k, InputResult::Preedit('ㅏ')),
            (xkb::KEY_r, InputResult::Preedit('가')),
        ])
    }

    #[test]
    fn next_jaum() {
        test_input(&[
            (xkb::KEY_d, InputResult::Preedit('ㅇ')),
            (xkb::KEY_k, InputResult::Preedit('아')),
            (xkb::KEY_d, InputResult::Preedit('앙')),
            (xkb::KEY_e, InputResult::CommitPreedit('앙', 'ㄷ')),
        ])
    }

    #[test]
    fn not_com_moum_when_continue() {
        test_input(&[
            (xkb::KEY_d, InputResult::Preedit('ㅇ')),
            (xkb::KEY_h, InputResult::Preedit('오')),
            (xkb::KEY_d, InputResult::Preedit('옹')),
            (xkb::KEY_k, InputResult::CommitPreedit('오', '아')),
        ]);
    }

    #[test]
    fn com_moum() {
        test_input(&[
            (xkb::KEY_d, InputResult::Preedit('ㅇ')),
            (xkb::KEY_h, InputResult::Preedit('오')),
            (xkb::KEY_l, InputResult::Preedit('외')),
            (xkb::KEY_d, InputResult::Preedit('욍')),
            (xkb::KEY_d, InputResult::CommitPreedit('욍', 'ㅇ')),
            (xkb::KEY_k, InputResult::Preedit('아')),
            (xkb::KEY_s, InputResult::Preedit('안')),
            (xkb::KEY_g, InputResult::Preedit('않')),
            (xkb::KEY_e, InputResult::CommitPreedit('않', 'ㄷ')),
        ]);
    }

    #[test]
    fn number() {
        test_input(&[
            (xkb::KEY_d, InputResult::Preedit('ㅇ')),
            (xkb::KEY_h, InputResult::Preedit('오')),
            (xkb::KEY_l, InputResult::Preedit('외')),
            (xkb::KEY_d, InputResult::Preedit('욍')),
            (xkb::KEY_d, InputResult::CommitPreedit('욍', 'ㅇ')),
            (xkb::KEY_k, InputResult::Preedit('아')),
            (xkb::KEY_s, InputResult::Preedit('안')),
            (xkb::KEY_g, InputResult::Preedit('않')),
            (xkb::KEY_e, InputResult::CommitPreedit('않', 'ㄷ')),
            (xkb::KEY_1, InputResult::CommitCommit('ㄷ', '1')),
        ]);
    }

    #[test]
    fn exclamation_mark() {
        test_input(&[
            (xkb::KEY_R, InputResult::Preedit('ㄲ')),
            (xkb::KEY_exclam, InputResult::CommitCommit('ㄲ', '!')),
        ]);
    }

    #[test]
    fn backspace() {
        test_input(&[
            (xkb::KEY_r, InputResult::Preedit('ㄱ')),
            (xkb::KEY_k, InputResult::Preedit('가')),
            (xkb::KEY_d, InputResult::Preedit('강')),
            (xkb::KEY_BackSpace, InputResult::Preedit('가')),
            (xkb::KEY_q, InputResult::Preedit('갑')),
            (xkb::KEY_t, InputResult::Preedit('값')),
            (xkb::KEY_BackSpace, InputResult::Preedit('갑')),
            (xkb::KEY_BackSpace, InputResult::Preedit('가')),
            (xkb::KEY_BackSpace, InputResult::Preedit('ㄱ')),
            (xkb::KEY_BackSpace, InputResult::ClearPreedit),
            (xkb::KEY_r, InputResult::Preedit('ㄱ')),
        ])
    }

    #[test]
    fn compose_jong() {
        test_input(&[
            (xkb::KEY_d, InputResult::Preedit('ㅇ')),
            (xkb::KEY_j, InputResult::Preedit('어')),
            (xkb::KEY_q, InputResult::Preedit('업')),
            (xkb::KEY_t, InputResult::Preedit('없')),
        ])
    }

    #[test]
    fn backspace_moum_compose() {
        test_input(&[
            (xkb::KEY_d, InputResult::Preedit('ㅇ')),
            (xkb::KEY_h, InputResult::Preedit('오')),
            (xkb::KEY_k, InputResult::Preedit('와')),
            (xkb::KEY_BackSpace, InputResult::Preedit('오')),
            (xkb::KEY_BackSpace, InputResult::Preedit('ㅇ')),
        ])
    }
}
