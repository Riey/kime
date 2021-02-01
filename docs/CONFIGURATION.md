# Options

[English](CONFIGURATION.md), [한국어](CONFIGURATION.ko.md)

Default config file is located at `/etc/kime/config.yaml`. Check
[default_config.yaml](../res/default_config.yaml) to see the default configuration
file. You edit `/etc/kime/config.yaml` or create a new config file at
`~/.config/kime/config.yaml`.

You can also change the location of config file using [`$XDG_CONFIG_DIR` or
`$XDG_CONFIG_HOME`][xdg] environment variable. kime will try to read
`$XDG_CONFIG_DIR/kime/config.yaml` and `$XDG_CONFIG_HOME/kime/config.yaml` too.

[xdg]: https://specifications.freedesktop.org/basedir-spec/basedir-spec-latest.html#introduction

## layout

Hangul layout name. "dubeolsik", "sebeolsik-390", and "sebeolsik-391" are
available as default. Custom layout can be added by creating layout YAML files
at `$XDG_CONFIG_HOME/kime/layouts/` directory. See [dubeolsik.yaml] for the
structure of keyboard layout file.

[dubeolsik.yaml]: ../src/engine/core/data/dubeolsik.yaml

| default |`dubeolsik`|
|---------|-----------|

## esc_turn_off

Turn off hangul mode when esc is pressed especially for VIM users

| default |`true`|
|---------|------|

## hangul_keys

Keycodes for switch hangul mode

| default |`[Hangul, Muhenkan, AltR, Super-Space]`|
|---------|---------------------------------------|

## xim_preedit_font

Preedit window font name and size for XIM

| default |`[D2Coding, 15.0]`|
|---------|------------------|

## compose

Adjust compose, decompose jamo

### compose_choseong_ssang

```txt
ㄱ + ㄱ = ㄲ
ㅅ + ㅅ = ㅆ
ㄷ + ㄷ = ㄸ
ㅂ + ㅂ = ㅃ
ㅈ + ㅈ = ㅉ
```

| default |`true`|
|---------|------|

### decompose_choseong_ssang

Same as above but work on backspace(e.g. ㄲ -> ㄱ)

| default |`false`|
|---------|-------|

### compose_jungseong_ssang

```txt
ㅑ + ㅣ = ㅒ
ㅕ + ㅣ = ㅖ
```

| default |`false`|
|---------|-------|

### decompose_jungseong_ssang

| default |`false`|
|---------|-------|

### compose_jongseong_ssang

```txt
ㄱ + ㄱ = ㄲ
ㅅ + ㅅ = ㅆ
```

| default |`false`|
|---------|-------|

### decompose_jongseong_ssang

| default |`false`|
|---------|-------|
