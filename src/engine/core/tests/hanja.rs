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
            "/倮(bare, naked, uncovered)儸囉摞瘰砢臝蓏覶鑼騾驘拏",
            "",
        ),
        (Key::normal(Enter), "", "倮"),
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
            "/倮(bare, naked, uncovered)儸囉摞瘰砢臝蓏覶鑼騾驘拏",
            "",
        ),
        (
            Key::normal(Right),
            "倮/儸(bandit, daredevil)囉摞瘰砢臝蓏覶鑼騾驘拏",
            "",
        ),
        (Key::normal(Enter), "", "摞"),
    ])
}
