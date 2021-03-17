#[macro_use]
mod shared;

define_layout_test!("dubeolsik");

#[test]
fn ra() {
    test_input(&[
        (Key::normal(F), "ㄹ", ""),
        (Key::normal(K), "라", ""),
        (
            Key::normal(HangulHanja),
            "/囉(exclamatory final particle, nag)摞瘰砢儸臝蓏倮覶鑼騾驘拏",
            "",
        ),
        (Key::normal(Enter), "", "囉"),
    ])
}
