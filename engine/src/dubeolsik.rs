pub const DUBEOLSIK_LAYOUT: &str = include_str!("../data/dubeolsik.yaml");

#[cfg(test)]
mod tests {
    use super::super::{state::CharacterState, InputResult, Layout};

    use xkbcommon::xkb::*;

    use pretty_assertions::assert_eq;

    #[track_caller]
    fn test_input(inputs: &[(Keysym, InputResult)]) {
        let layout = Layout::dubeolsik();
        let mut state = CharacterState::default();

        for (code, expect_result) in inputs.iter().copied() {
            assert_eq!(expect_result, layout.map_key(&mut state, code));
        }
    }

    #[test]
    fn com_moum() {
        test_input(&[
            (KEY_d, InputResult::Preedit('ㅇ')),
            (KEY_h, InputResult::Preedit('오')),
            (KEY_l, InputResult::Preedit('외')),
            (KEY_d, InputResult::Preedit('욍')),
            (KEY_d, InputResult::CommitPreedit('욍', 'ㅇ')),
            (KEY_k, InputResult::Preedit('아')),
            (KEY_s, InputResult::Preedit('안')),
            (KEY_g, InputResult::Preedit('않')),
            (KEY_e, InputResult::CommitPreedit('않', 'ㄷ')),
        ]);
    }

    #[test]
    fn number() {
        test_input(&[
            (KEY_d, InputResult::Preedit('ㅇ')),
            (KEY_h, InputResult::Preedit('오')),
            (KEY_l, InputResult::Preedit('외')),
            (KEY_d, InputResult::Preedit('욍')),
            (KEY_d, InputResult::CommitPreedit('욍', 'ㅇ')),
            (KEY_k, InputResult::Preedit('아')),
            (KEY_s, InputResult::Preedit('안')),
            (KEY_g, InputResult::Preedit('않')),
            (KEY_e, InputResult::CommitPreedit('않', 'ㄷ')),
            (KEY_1, InputResult::CommitCommit('ㄷ', '1')),
        ]);
    }

    #[test]
    fn exclamation_mark() {
        test_input(&[
            (KEY_R, InputResult::Preedit('ㄲ')),
            (KEY_exclam, InputResult::CommitCommit('ㄲ', '!')),
        ]);
    }

    #[test]
    fn backspace() {
        test_input(&[
            (KEY_r, InputResult::Preedit('ㄱ')),
            (KEY_k, InputResult::Preedit('가')),
            (KEY_d, InputResult::Preedit('강')),
            (KEY_BackSpace, InputResult::Preedit('가')),
            (KEY_q, InputResult::Preedit('갑')),
            (KEY_t, InputResult::Preedit('값')),
            (KEY_BackSpace, InputResult::Preedit('갑')),
            (KEY_BackSpace, InputResult::Preedit('가')),
            (KEY_BackSpace, InputResult::Preedit('ㄱ')),
            (KEY_BackSpace, InputResult::ClearPreedit),
            (KEY_r, InputResult::Preedit('ㄱ')),
        ])
    }

    #[test]
    fn compose_jong() {
        test_input(&[
            (KEY_d, InputResult::Preedit('ㅇ')),
            (KEY_j, InputResult::Preedit('어')),
            (KEY_q, InputResult::Preedit('업')),
            (KEY_t, InputResult::Preedit('없')),
        ])
    }

    #[test]
    fn backspace_moum_compose() {
        test_input(&[
            (KEY_d, InputResult::Preedit('ㅇ')),
            (KEY_h, InputResult::Preedit('오')),
            (KEY_k, InputResult::Preedit('와')),
            (KEY_BackSpace, InputResult::Preedit('오')),
            (KEY_BackSpace, InputResult::Preedit('ㅇ')),
        ])
    }
}
