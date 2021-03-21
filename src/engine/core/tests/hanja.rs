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

#[test]
fn empty() {
    test_input(&[
        (Key::normal(HangulHanja), "", ""),
    ]);
    test_input(&[
        (Key::normal(R), "ㄱ", ""),
        (Key::normal(HangulHanja), "ㄱ", ""),
    ]);
}

// issue #381
#[test]
fn ra_arrow() {
    test_input(&[
        (Key::normal(F), "ㄹ", ""),
        (Key::normal(K), "라", ""),
        (
            Key::normal(HangulHanja),
            "/囉(exclamatory final particle, nag)摞瘰砢儸臝蓏倮覶鑼騾驘拏",
            "",
        ),
        (
            Key::normal(Right),
            "囉/摞(to pile up)瘰砢儸臝蓏倮覶鑼騾驘拏",
            "",
        ),
        (Key::normal(Enter), "", "摞"),
    ])
}
