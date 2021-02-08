# 설정

[English](CONFIGURATION.md), [한국어](CONFIGURATION.ko.md)

`/etc/kime/config.yaml`에 기본값으로 설정된 파일이 있습니다. [default_config.yaml](../res/default_config.yaml)에서 기본 설정 파일을 온라인으로 볼 수도 있습니다. `/etc/kime/config.yaml`에서 전역 설정을 수정하거나 `~/.config/kime/config.yaml`에 새 설정 파일을 만들어 보세요.

[`$XDG_CONFIG_DIR`이나 `$XDG_CONFIG_HOME`][xdg] 환경 변수를 이용해 설정 파일의 위치를 바꿀 수도 있습니다. kime는 `$XDG_CONFIG_DIR/kime/config.yaml`과 `$XDG_CONFIG_HOME/kime/config.yaml`에 있는 설정 파일도 읽으려고 시도할 것입니다.

[xdg]: https://specifications.freedesktop.org/basedir-spec/basedir-spec-latest.html#introduction

## layout

키보드 자판을 설정합니다. `dubeolsik`(두벌식), `sebeolsik-390`(세벌식 390), `sebeolsik-391`(세벌식 391)이 기본으로 내장되어 있습니다. `$XDG_CONFIG_HOME/kime/layouts/`에 위 목록에 없는 키보드 자판을 YAML 파일로 직접 만들 수도 있습니다. [dubeolsik.yaml]을 참고해 보세요.

[dubeolsik.yaml]: ../src/engine/core/data/dubeolsik.yaml

| 기본값 |`dubeolsik`|
|--------|-----------|

## global_hangul_state

한영상태를 전역에서 설정합니다.

| 기본값 |`false`|
|--------|-------|

## hotkeys

엔진의 단축키를 설정합니다 형식은 `키: 행동` 입니다

### 행동

#### ToggleHangul

한영상태를 바꿉니다

#### ToEnglish

영문모드로 바꿉니다

#### ToHangul

한글모드로 바꿉니다

### 기본값

```txt
  Esc: ToEnglish
  Muhenkan: ToggleHangul
  Hangul: ToggleHangul
  AltR: ToggleHangul
  Super-Space: ToggleHangul
```

## xim_preedit_font

XIM에서 쓸 편집창 글꼴과 크기입니다.

| 기본값 |`[D2Coding, 15.0]`|
|--------|------------------|

## compose

자모 합성/분해 방식을 조정합니다.

### compose_choseong_ssang

같은 자음을 두 번 누를 때 쌍자음을 합성합니다.

```txt
ㄱ + ㄱ = ㄲ
ㅅ + ㅅ = ㅆ
ㄷ + ㄷ = ㄸ
ㅂ + ㅂ = ㅃ
ㅈ + ㅈ = ㅉ
```

| 기본값 |`true`|
|--------|------|

### decompose_choseong_ssang

쌍자음에 백스페이스를 누를 때 쌍자음을 분해시킵니다. (e.g. ㄲ -> ㄱ)

| 기본값 |`false`|
|--------|-------|

### compose_jungseong_ssang

```txt
ㅑ + ㅣ = ㅒ
ㅕ + ㅣ = ㅖ
```

| 기본값 |`false`|
|--------|-------|

### decompose_jungseong_ssang

| 기본값 |`false`|
|--------|-------|

### compose_jongseong_ssang

```txt
ㄱ + ㄱ = ㄲ
ㅅ + ㅅ = ㅆ
```

| 기본값 |`false`|
|--------|-------|

### decompose_jongseong_ssang

| 기본값 |`false`|
|--------|-------|
