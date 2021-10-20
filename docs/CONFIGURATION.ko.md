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

### global_hotkeys

전역 단축키입니다

### category_hotkeys

언어별 단축키입니다 전역 단축키를 덮어씁니다

### mode_hotkeys

모드별 단축키입니다 전역과 언어별 단축키를 덮어씁니다

### 내용

#### behavior

##### Toggle: [InputCategory, InputCategory]

왼쪽과 오른쪽의 상태를 바꿉니다

##### Switch: InputCategory

해당 언어로 바꿉니다

##### Mode: InputMode

해당 모드를 활성화합니다

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

| 기본값 |`[D2Coding, 15.0]`|
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
