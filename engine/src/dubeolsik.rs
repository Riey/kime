pub const DUBEOLSIK_LAYOUT: &str = include_str!("../data/dubeolsik.yaml");

#[cfg(test)]
mod tests {
    use super::super::{
        state::CharacterState,
        InputResult,
        KeyCode::{self, *},
        Layout,
    };

    use pretty_assertions::assert_eq;

    #[track_caller]
    fn test_input(inputs: &[(KeyCode, InputResult)]) {
        let layout = Layout::dubeolsik();
        let mut enable_hangul = true;
        let mut state = CharacterState::default();

        for (code, expect_result) in inputs.iter().copied() {
            assert_eq!(
                expect_result,
                layout.map_key(&mut state, &mut enable_hangul, code, false)
            );
        }
    }

    #[test]
    fn com_moum() {
        test_input(&[
            (D, InputResult::Preedit('ㅇ')),
            (H, InputResult::Preedit('오')),
            (L, InputResult::Preedit('외')),
            (D, InputResult::Preedit('욍')),
            (D, InputResult::CommitPreedit('욍', 'ㅇ')),
            (K, InputResult::Preedit('아')),
            (S, InputResult::Preedit('안')),
            (G, InputResult::Preedit('않')),
            (E, InputResult::CommitPreedit('않', 'ㄷ')),
        ]);
    }

    #[test]
    fn backspace() {
        test_input(&[
            (R, InputResult::Preedit('ㄱ')),
            (K, InputResult::Preedit('가')),
            (D, InputResult::Preedit('강')),
            (Bs, InputResult::Preedit('가')),
            (Q, InputResult::Preedit('갑')),
            (T, InputResult::Preedit('값')),
            (Bs, InputResult::Preedit('갑')),
            (Bs, InputResult::Preedit('가')),
            (Bs, InputResult::Preedit('ㄱ')),
            (Bs, InputResult::ClearPreedit),
            (R, InputResult::Preedit('ㄱ')),
        ])
    }

    #[test]
    fn compose_jong() {
        test_input(&[
            (D, InputResult::Preedit('ㅇ')),
            (J, InputResult::Preedit('어')),
            (Q, InputResult::Preedit('업')),
            (T, InputResult::Preedit('없')),
        ])
    }

    #[test]
    fn backspace_moum_compose() {
        test_input(&[
            (D, InputResult::Preedit('ㅇ')),
            (H, InputResult::Preedit('오')),
            (K, InputResult::Preedit('와')),
            (Bs, InputResult::Preedit('오')),
            (Bs, InputResult::Preedit('ㅇ')),
        ])
    }
}
