#[macro_use]
mod shared;

define_layout_test!("dubeolsik");

#[test]
fn ga() {
    test_input(&[
        (Key::normal(R), "ㄱ", ""),
        (Key::normal(K), "가", ""),
        (
            Key::normal(HangulHanja),
            "0/4 [1] 可(옳을 가)[2] 家(집 가)[3] 加(더할 가)[4] 歌(노래 가)[5] 價(값 가)[6] 街(거리 가)[7] 假(거짓 가)[8] 佳(아름다울 가)[9] 暇(겨를 가)[10] 架(시렁 가)",
            "",
        ),
        (Key::normal(One), "", "可"),
    ])
}

#[test]
fn empty() {
    test_input(&[(Key::normal(HangulHanja), "", "")]);
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
            "0/1 [1] 羅(새그물 라)[2] 裸(벌거벗을 라, 벌거숭이 라)[3] 懶(게으를 라)[4] 螺(소라 라)[5] 拏(붙잡을 라)[6] 邏(순행할 라, 돌 라)[7] 癩(약물 중독 라)[8] 蘿(무 라, 소나무겨우살이 라)[9] 喇(나팔)[10] 騾(노새 라)",
            "",
        ),
        (
            Key::normal(Right),
            "1/1 [1] 囉(소리 읽힐 라)[2] 鑼(징 라)[3] 瘰(연주창 라)[4] 臝(벌거벗을 라)[5] 倮(알몸 라)[6] 曪(날 흐릴 라)[7] 驘(옹 솥 라)",
            "",
        ),
        (Key::normal(One), "", "囉"),
    ])
}
