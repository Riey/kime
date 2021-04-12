# config.yaml

[English](CONFIGURATION.md), [한국어](CONFIGURATION.ko.md)

Default config file is located at `/etc/xdg/kime/config.yaml`. Check
[default_config.yaml](../res/default_config.yaml) to see the default configuration
file. You edit `/etc/xdg/kime/config.yaml` or create a new config file at
`~/.config/kime/config.yaml`.

You can also change the location of config file using [`$XDG_CONFIG_DIR` or
`$XDG_CONFIG_HOME`][xdg] environment variable. kime will try to read
`$XDG_CONFIG_DIR/kime/config.yaml` and `$XDG_CONFIG_HOME/kime/config.yaml` too.

[xdg]: https://specifications.freedesktop.org/basedir-spec/basedir-spec-latest.html#introduction

# daemon

`kime` daemon setting

## modules

List of daemon modules default is *all*

* Xim
* Wayland
* Indicator

# indicator

`kime-indicator` setting

## icon_color

Set icon color for indicator

### Possible values

* Black
* White

| default |`Black`|
|---------|-------|

# engine

`kime-engine` setting

## default_category

Set default InputCategory when IME starts, please select between `Latin` and `Hangul`

| default |`Latin`|
|---------|-------|

## global_category_state

Set category state globally

| default |`false`|
|---------|-------|

## hotkeys

Set engine hotkey format is `Key: Content`

### global_hotkeys

Global hotkey

### category_hotkeys

Hotkey for specific category override global hotkey

### mode_hotkeys

Hotkey for specific mode override global, category hotkey

### content

#### behavior

##### Toggle: [InputCategory, InputCategory]

Toggle Left and Right category

##### Switch: InputCategory

Switch to specific category

##### Mode: InputMode

Enable specific mode

##### Commit

End current preedit state then commit

##### Ignore

Do nothing

#### result

##### Bypass

Bypass key to continue key process

##### Consume

Consume key to end key process

##### ConsumeIfProcessed

When hotkey processed it act like Consume otherwise it act like Bypass

## xim_preedit_font

Preedit window font name and size for XIM

| default |`[D2Coding, 15.0]`|
|---------|------------------|

## latin

Set latin setting

### preferred_direct

Handling key event by external as possible

### layout

Set latin layout

| default |`Qwerty`|
|---------|--------|

### embeded layouts

* `Qwerty`
* `Dvorak`
* `Colemak`

## hangul

Set hangul setting

### word_commit

Let commit by word

| default |`false`|
|---------|-------|

### layout

Set hangul layout

| default |`dubeolsik`|
|---------|-------|

#### Embeded layouts

* `direct`
* `qwerty`
* `colmak`
* `dubeolsik`(두벌식)
* `sebeolsik-390`(세벌식 390)
* `sebeolsik-391`(세벌식 최종)
* `sebeolsik-3sin-1995`(신세벌식 1995)
* `sebeolsik-3sin-p2`(신세벌식 p2 *옛한글은 미구현*)

Custom layout can be added by creating layout YAML files
at `$XDG_CONFIG_HOME/kime/layouts/` directory. See [dubeolsik.yaml] for the
structure of keyboard layout file.

[dubeolsik.yaml]: ../src/engine/core/data/dubeolsik.yaml

### layout_addons

Adjust layout addons

format is `layout_name: [Addon]`, `all` applys all layouts

#### default

```yaml
all:
  - ComposeChoseongSsang
dubeolsik:
  - TreatJongseongAsChoseongg
```

#### Addons

##### TreatJongseongAsChoseong

Treat jongseong as choseong

```txt
간 + ㅏ = 가나
값 + ㅏ = 갑사
```

##### TreatJongseongAsChoseongCompose

Compose previous jongseong and current choseong

Note that it depends on other addons this example is only work when `ComposeChoseongSsang` is on

```txt
읅 + ㄱ = 을ㄲ
앇 + ㅅ = 악ㅆ
```

##### FlexibleComposeOrder

Compose choseong and jungseong even order is reversed it could be help for fix typo error.

```txt
ㅏ + ㄱ = 가
ㅚ + ㄱ = 괴
```

##### ComposeChoseongSsang

When you press same choseong it will be ssangjaum

```txt
ㄱ + ㄱ = ㄲ
ㅅ + ㅅ = ㅆ
ㄷ + ㄷ = ㄸ
ㅂ + ㅂ = ㅃ
ㅈ + ㅈ = ㅉ
```

##### DecomposeChoseongSsang

Same as above but work on backspace(e.g. ㄲ -> ㄱ)

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

#### DecomposeJongseongSsang
