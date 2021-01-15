# Options

## layout

Hangul layout name

### default

`dubeolsik`

## esc_turn_off

Turn off hangul mode when esc is pressed especially for VIM users

### default

`true`

## hangul_keys

Keycodes for switch hangul mode

### default

`[Hangul, Henkan, AltR]`

## xim_preedit_font

Preedit window font for XIM

### default

`D2Coding`

## gtk_commit_english

Commit english rather then bypass keyevent need for some programs like gedit

### default

`true`

## compose

Adjust compose, decompose jamo

* compose_choseong_ssang

```
ㄱ + ㄱ = ㄲ
ㅅ + ㅅ = ㅆ
ㄷ + ㄷ = ㄸ
ㅂ + ㅂ = ㅃ
ㅈ + ㅈ = ㅉ
```

### default

`true`

* decompose_choseong_ssang

Same as above but work on backspace(e.g. ㄲ -> ㄱ)

### default

`false`

* compose_jungseong_ssang

```
ㅑ + ㅣ = ㅒ
ㅕ + ㅣ = ㅖ
```

### default

`false`


* decompose_jungseong_ssang

### default

`false`

* compose_jongseong_ssang

```
ㄱ + ㄱ = ㄲ
ㅅ + ㅅ = ㅆ
```

### default

`false`

* decompose_jongseong_ssang

### default

`false`
