# config.yaml

[English](CONFIGURATION.md), [한국어](CONFIGURATION.ko.md)

`/usr/share/doc/kime/default_config.yaml`에 기본 설정 파일 샘플이 있습니다.
[default_config.yaml](../res/default_config.yaml)에서 기본 설정 파일을 온라인으로 볼 수도 있습니다.
이 파일을 `/etc/xdg/kime/config.yaml`로 복사하여 전역 설정으로 사용하세요.
`~/.config/kime/config.yaml`에 사용자마다 각각 적용되는 설정 파일을 만들 수도 있습니다.

[`$XDG_CONFIG_DIR`이나 `$XDG_CONFIG_HOME`][xdg] 환경 변수를 이용해 설정 파일의 위치를 바꿀 수도 있습니다. kime는 `$XDG_CONFIG_DIR/kime/config.yaml`과 `$XDG_CONFIG_HOME/kime/config.yaml`에 있는 설정 파일도 읽으려고 시도할 것입니다.

[xdg]: https://specifications.freedesktop.org/basedir-spec/basedir-spec-latest.html#introduction

# log

kime 프로그램들의 로그 레벨을 설정합니다

`OFF`, `ERROR`, `WARN`, `INFO`, `DEBUG`, `TRACE` 중에서 선택해주세요.

## global_level

전역 로깅 레벨입니다

# daemon

`kime` 데몬의 설정입니다

## modules

데몬의 모듈 목록입니다 기본값은 *전부*입니다

* Xim
* Wayland
* Indicator

# indicator

`kime-indicator`의 설정입니다

### icon_color

indicator에서 사용할 아이콘의 색을 정합니다

#### 가능한 값

* Black
* White

| 기본값 |`Black`|
|--------|-------|

# engine

`kime-engine`의 설정입니다

## translation_layer

키코드 번역 레이어를 추가합니다 특수한 키보드를 사용할때 유용합니다.

| 기본값 |`null`|
|--------|-------|

## default_category

입력기가 시작될때의 기본 언어를 설정합니다. `Latin`(로마자), `Hangul`(한글) 중에서 설정해주세요

| 기본값 |`Latin`|
|--------|-------|

## global_category_state

언어상태를 전역에서 설정합니다.

| 기본값 |`false`|
|--------|-------|


## hotkeys

엔진의 단축키를 설정합니다 형식은 `키: 내용` 입니다

### 키 형식

키는 0개 이상의 수식키 접두사 뒤에 키 이름을 이어서 적습니다.

| 접두사   | 수식키                        |
|----------|-------------------------------|
| `Super-` | Super (윈도우/로고 키)        |
| `M-`     | Alt (Emacs의 Meta에서 온 `M`) |
| `C-`     | Ctrl                          |
| `S-`     | Shift                         |

접두사는 순서에 상관없이 조합할 수 있으며, kime가 출력할 때는 `Super-`,
`M-`, `C-`, `S-` 순서로 적습니다.

예시:

* `S-Space` — Shift+Space
* `M-C-Backslash` — Alt+Ctrl+`\`
* `Super-Space` — Super+Space

사용할 수 있는 키 이름:

* 숫자: `1`-`9`, `0`
* 숫자패드 숫자(NumLock 켜짐): `N1`-`N9`, `N0`
* 로마자: `A`-`Z`
* `Minus`, `Equal`, `Backslash`, `Grave`, `OpenBracket`, `CloseBracket`,
  `Space`, `Comma`, `Period`, `SemiColon`, `Quote`, `Slash`
* `Esc`, `Shift`, `Backspace`, `Enter`, `Tab`, `ControlL`, `ControlR`,
  `Delete`, `Insert`, `Home`, `End`, `PageUp`, `PageDown`, `Muhenkan`,
  `Henkan`, `AltL`, `AltR`, `SuperL`, `SuperR`, `Hangul`, `HangulHanja`
* 방향키: `Left`, `Right`, `Up`, `Down`
* 기능키: `F1`-`F12`

`Shift`는 좌우 구분이 없는 단일 키 이름이고 Ctrl, Alt, Super는
`ControlL`/`ControlR`, `AltL`/`AltR`, `SuperL`/`SuperR`로 좌우가
구분됩니다. 수식키 자체도 단축키로 지정할 수 있습니다(기본 설정의
`AltR`, `ControlR`처럼).

### global_hotkeys

전역 단축키입니다

### category_hotkeys

언어별 단축키입니다 전역 단축키를 덮어씁니다

### mode_hotkeys

모드별 단축키입니다 전역과 언어별 단축키를 덮어씁니다

### 내용

#### behavior

##### !Toggle [InputCategory, InputCategory]

왼쪽과 오른쪽의 상태를 바꿉니다

##### !Switch InputCategory

해당 언어로 바꿉니다

##### !Mode InputMode

해당 모드를 활성화합니다 각 모드의 동작은 [입력 모드](#입력-모드)를
참고하세요

##### Commit

현재 조합상태를 종료하고 커밋합니다

##### Ignore

아무 동작도 하지 않습니다

#### result

##### Bypass

키를 계속 처리합니다

##### Consume

키 처리를 종료합니다

##### ConsumeIfProcessed

단축키가 실행됐을 경우에는 Consume처럼, 아닐때는 Bypass처럼 동작합니다.

## xim_preedit_font

XIM에서 쓸 편집창 글꼴과 크기입니다.

| 기본값 |`[Noto Sans CJK KR, 15.0]`|
|--------|--------------------------|

## candidate_font

[Hanja 모드](#hanja)에서 쓰는 `kime-candidate-window` 팝업의 글꼴
이름입니다.

| 기본값 |`Noto Sans CJK KR`|
|--------|------------------|

## latin

로마자 입력기를 설정합니다.

### preferred_direct

될 수 있으면 키 이밴트를 외부에서 처리합니다.

### layout

로마자 자판을 설정합니다.

| 기본값 |`Qwerty`|
|--------|-------|

### 가능한 자판들

* `Qwerty`
* `Dvorak`
* `Colemak`

## hangul

한글 입력기를 설정합니다.

### layout

한글 자판을 설정합니다.

| 기본값 |`dubeolsik`|
|--------|-------|

### 내장된 자판들

* `direct`
* `qwerty`
* `colmak`
* `dubeolsik`(두벌식)
* `sebeolsik-3-90`(세벌식 390)
* `sebeolsik-3-91`(세벌식 최종)
* `sebeolsik-3sin-1995`(신세벌식 1995)
* `sebeolsik-3sin-p2`(신세벌식 p2 *옛한글은 미구현*)

`$XDG_CONFIG_HOME/kime/layouts/`에 위 목록에 없는 키보드 자판을 YAML 파일로 직접 만들 수도 있습니다. [dubeolsik.yaml]을 참고해 보세요.

[dubeolsik.yaml]: ../src/engine/backends/hangul/data/dubeolsik.yaml

### preedit_johab

편집상태에 조합형을 어느정도로 사용할지 설정합니다.

| default |`Needed`|
|---------|-------|

### word_commit

커밋을 단어 단위로 합니다.

| 기본값 |`false`|
|--------|-------|

### addons

한글 자판의 추가 기능을 설정 합니다

형식은 `자판이름: [Addon]` 입니다 `all`은 모든 자판에 적용됩니다.

#### 기본값

```yaml
all:
  - ComposeChoseongSsang
dubeolsik:
  - TreatJongseongAsChoseong
sebeolsik-3sin-p2:
  - ComposeJongseongSsang
```

#### Addons

##### TreatJongseongAsChoseong

종성을 초성처럼 취급합니다.

```txt
간 + ㅏ = 가나
값 + ㅏ = 갑사
```

##### TreatJongseongAsChoseongCompose

이전 종성과 현재 초성을 조합합니다.

참고로 이건 다른 애드온들에 따라 달라집니다 이 예제는 `ComposeChoseongSsang`이 켜져있어야 작동합니다

```txt
읅 + ㄱ = 을ㄲ
앇 + ㅅ = 악ㅆ
```

##### FlexibleComposeOrder

초성, 중성, 종성의 순서를 바꿔도 조합이 되도록 합니다 오타 교정에 도움이 될 수 있습니다.

```txt
ㅏ + ㄱ = 가
ㅚ + ㄱ = 괴
ㅏ + $ㅁ + ㅁ = 맘
```

##### ComposeChoseongSsang

같은 자음을 두 번 누를 때 쌍자음을 합성합니다.

```txt
ㄱ + ㄱ = ㄲ
ㅅ + ㅅ = ㅆ
ㄷ + ㄷ = ㄸ
ㅂ + ㅂ = ㅃ
ㅈ + ㅈ = ㅉ
```

##### DecomposeChoseongSsang

쌍자음에 백스페이스를 누를 때 쌍자음을 분해시킵니다. (e.g. ㄲ -> ㄱ)

##### ComposeJungseongSsang

```txt
ㅑ + ㅣ = ㅒ
ㅕ + ㅣ = ㅖ
```

##### DecomposeJungseongSsang

##### ComposeJongseongSsang

```txt
ㄱ + ㄱ = ㄲ
ㅅ + ㅅ = ㅆ
```

##### DecomposeJongseongSsang

# 입력 모드

kime에는 `Latin`/`Hangul` 입력 언어 외에 세 가지 입력 *모드*가
있습니다: `Math`, `Hanja`, `Emoji`. 모드는 `!Mode`
단축키([hotkeys](#hotkeys) 참고)로 켜지며 현재 언어 위에서 동작합니다.

기본 단축키:

| 모드    | 기본 단축키                     | 사용 범위          |
|---------|---------------------------------|--------------------|
| `Math`  | `M-C-Backslash` (Alt+Ctrl+`\`)  | 전역               |
| `Emoji` | `M-C-E` (Alt+Ctrl+E)            | 전역               |
| `Hanja` | `F9`, `HangulHanja`, `ControlR` | `Hangul` 언어에서만 |

모든 모드에서 `Enter`와 `Tab`이 기본으로 `Commit`에 연결되어 있어 현재
입력을 커밋합니다. 전역 단축키는 `mode_hotkeys`에서 덮어쓰지 않는 한
모드 안에서도 계속 동작하므로, 예를 들어 기본 `Esc` 단축키(`!Switch
Latin`)로 모드에서 빠져나올 수 있습니다.

## Math

수학 기호를 (대부분) LaTeX 이름으로 입력하는 모드입니다. 이 모드는
지속되는 모드로, 기호를 커밋한 뒤에도 언어를 바꾸기 전까지(예:
`Esc`나 한영 전환키) 유지됩니다.

* `\`로 기호 이름 입력을 시작합니다. 이름을 입력한 뒤
  `Enter`/`Tab`으로 커밋합니다: `\pi` → π, `\Pi` → Π (이름은
  대소문자를 구분합니다).
* 기호 이름을 입력 중이 아닐 때 누른 키는 일반 로마자로 커밋됩니다.
* `\\`는 백슬래시 `\` 문자를 커밋합니다.
* `Backspace`는 이름의 마지막 글자를 지우며, 이름이 비어 있으면 기호
  입력을 종료합니다.
* 이름 앞에 `\<스타일>.<이름>` 형식으로 스타일을 붙일 수 있습니다.
  스타일은 `sf`(산세리프), `bf`(굵게), `it`(기울임), `tt`(고정폭),
  `bb`(겹선), `scr`(스크립트), `cal`(필기체), `frak`(프락투어)이며
  순서에 상관없이 이어 붙일 수 있습니다: `\bfit.alpha` → 𝜶.
* 없는 이름은 아무것도 커밋하지 않습니다. 해석할 수 없는 스타일은
  조용히 무시되고 스타일 없는 기호가 커밋됩니다.
* LaTeX에 없는 이름도 일부 있습니다. 예: `\squotl` → 「. 전체 목록은
  [symbol_map.json]을 보세요.

[symbol_map.json]: ../src/engine/dict/data/symbol_map.json

## Emoji

이모지를 이름으로 검색해 입력하는 모드입니다.

* 이모지의 영어 이름 일부를 입력하세요 — 영어 [유니코드 CLDR][cldr]
  이름에 대한 부분 문자열 검색이므로 한글 이름으로는 검색되지
  않습니다.
* 편집창에 검색어와 함께 일치하는 후보가 최대 5개 표시됩니다.
* `Space`도 검색어의 일부입니다(예: `red apple`).
* `Enter`/`Tab`은 첫 번째 후보를 커밋하고 모드를 종료합니다.
* `Backspace`는 검색어의 마지막 글자를 지우며, 검색어가 비어 있으면
  모드를 종료합니다.

[cldr]: https://cldr.unicode.org

## Hanja

조합 중인 한글을 한자로 변환하는 모드로, `Hangul` 언어에서 편집 중인
글자가 있을 때만 동작합니다(예: 한을 입력한 뒤 `F9`).

* 후보와 뜻을 보여주는 `kime-candidate-window` 팝업이 열립니다.
  `kime-candidate-window` 바이너리가 `PATH`에 설치되어 있어야 합니다.
* 마우스로 후보를 클릭하면 커밋되고, `Esc`를 누르면 변환 없이
  닫힙니다.

팝업의 키보드 조작은 아직 미비하며 개편 중입니다.
