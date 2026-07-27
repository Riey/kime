# config.yaml

[English](CONFIGURATION.md), [한국어](CONFIGURATION.ko.md)

Sample config file is located at `/usr/share/doc/kime/default_config.yaml`. Check
[default_config.yaml](../res/default_config.yaml) to see the default configuration
file. Copy this file to `/etc/xdg/kime/config.yaml` for global default configuration.
You may create per user file at `~/.config/kime/config.yaml`.

You can also change the location of config file using [`$XDG_CONFIG_DIR` or
`$XDG_CONFIG_HOME`][xdg] environment variable. kime will try to read
`$XDG_CONFIG_DIR/kime/config.yaml` and `$XDG_CONFIG_HOME/kime/config.yaml` too.

[xdg]: https://specifications.freedesktop.org/basedir-spec/basedir-spec-latest.html#introduction

# log

Set kime programs logging level

Please select one of `OFF`, `ERROR`, `WARN`, `INFO`, `DEBUG`, `TRACE`

## global_level

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

## translation_layer

Set keycode translation layer useful when you're using special keyboard.

| default |`null`|
|---------|-------|

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

### Key format

A key is written as zero or more modifier prefixes followed by a key name.

| prefix   | modifier                     |
|----------|------------------------------|
| `Super-` | Super (Windows/logo key)     |
| `M-`     | Alt (`M` as in Emacs' Meta)  |
| `C-`     | Ctrl                         |
| `S-`     | Shift                        |

Prefixes can be combined in any order; kime itself prints them in the
order `Super-`, `M-`, `C-`, `S-`.

Examples:

* `S-Space` — Shift+Space
* `M-C-Backslash` — Alt+Ctrl+`\`
* `Super-Space` — Super+Space

Available key names:

* digits: `1`-`9`, `0`
* numpad digits (NumLock on): `N1`-`N9`, `N0`
* letters: `A`-`Z`
* `Minus`, `Equal`, `Backslash`, `Grave`, `OpenBracket`, `CloseBracket`,
  `Space`, `Comma`, `Period`, `SemiColon`, `Quote`, `Slash`
* `Esc`, `Shift`, `Backspace`, `Enter`, `Tab`, `ControlL`, `ControlR`,
  `Delete`, `Insert`, `Home`, `End`, `PageUp`, `PageDown`, `Muhenkan`,
  `Henkan`, `AltL`, `AltR`, `SuperL`, `SuperR`, `Hangul`, `HangulHanja`
* arrow keys: `Left`, `Right`, `Up`, `Down`
* function keys: `F1`-`F12`

Note that `Shift` is a single key name without left/right variants while
Ctrl, Alt and Super come as `ControlL`/`ControlR`, `AltL`/`AltR`,
`SuperL`/`SuperR`. Modifier keys themselves can also be bound as hotkeys
(e.g. the default `AltR` and `ControlR` bindings).

### global_hotkeys

Global hotkey

### category_hotkeys

Hotkey for specific category override global hotkey

### mode_hotkeys

Hotkey for specific mode override global, category hotkey

### content

#### behavior

##### !Toggle [InputCategory, InputCategory]

Toggle Left and Right category

##### !Switch InputCategory

Switch to specific category

##### !Mode InputMode

Enable specific mode, see [Input modes](#input-modes) for what each mode
does

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

| default |`[Noto Sans CJK KR, 15.0]`|
|---------|--------------------------|

## candidate_font

Font name for the `kime-candidate-window` popup used by
[Hanja mode](#hanja)

| default |`Noto Sans CJK KR`|
|---------|------------------|

## latin

Set latin setting

### preferred_direct

Pass latin key events through to the application directly instead of letting
kime remap them, so the OS/firmware keyboard layout is used as-is.

| default |`true`|
|---------|------|

**Note:** when `preferred_direct` is `true` the embedded [`layout`](#layout)
below is **ignored**, because keys are handed to the OS layout untouched. To use
an embedded latin layout such as `Dvorak` or `Colemak`, set
`preferred_direct: false`. (kime maps physical keycodes, so the hangul layout is
unaffected by this setting.) See [#626].

[#626]: https://github.com/Riey/kime/issues/626

### layout

Set latin layout.

Only takes effect when [`preferred_direct`](#preferred_direct) is `false`.

| default |`Qwerty`|
|---------|--------|

### embedded layouts

* `Qwerty`
* `Dvorak`
* `Colemak`

## hangul

Set hangul setting

### preedit_johab

Set preedit johab encoding level

| default |`Needed`|
|---------|-------|

### word_commit

Let commit by word

| default |`false`|
|---------|-------|

### layout

Set hangul layout

| default |`dubeolsik`|
|---------|-------|

#### Embedded layouts

* `direct`
* `qwerty`
* `colmak`
* `dubeolsik`(두벌식)
* `sebeolsik-3-90`(세벌식 390)
* `sebeolsik-3-91`(세벌식 최종)
* `sebeolsik-3sin-1995`(신세벌식 1995)
* `sebeolsik-3sin-p2`(신세벌식 p2 *옛한글은 미구현*)

Custom layout can be added by creating layout YAML files
at `$XDG_CONFIG_HOME/kime/layouts/` directory. See [dubeolsik.yaml] for the
structure of keyboard layout file.

[dubeolsik.yaml]: ../src/engine/backends/hangul/data/dubeolsik.yaml

### layout_addons

Adjust layout addons

format is `layout_name: [Addon]`, `all` applies all layouts

#### default

```yaml
all:
  - ComposeChoseongSsang
dubeolsik:
  - TreatJongseongAsChoseongg
sebeolsik-3sin-p2:
  - ComposeJongseongSsang
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

Compose choseong, jungseong, and jongseong even order is reversed it could be help for fix typo error.

```txt
ㅏ + ㄱ = 가
ㅚ + ㄱ = 괴
ㅏ + $ㅁ + ㅁ = 맘
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

# Input modes

Besides the `Latin`/`Hangul` input categories kime has three input
*modes*: `Math`, `Hanja` and `Emoji`. A mode is entered with a `!Mode`
hotkey (see [hotkeys](#hotkeys)) and works on top of the current
category.

Default hotkeys:

| mode    | default hotkey                  | available in            |
|---------|---------------------------------|-------------------------|
| `Math`  | `M-C-Backslash` (Alt+Ctrl+`\`)  | everywhere              |
| `Emoji` | `M-C-E` (Alt+Ctrl+E)            | everywhere              |
| `Hanja` | `F9`, `HangulHanja`, `ControlR` | `Hangul` category only  |

In every mode `Enter` and `Tab` are bound to `Commit` by default, which
commits the current input. Global hotkeys keep working inside a mode
unless overridden in `mode_hotkeys`, so for example the default `Esc`
hotkey (`!Switch Latin`) also leaves a mode.

## Math

Math mode inputs Unicode math symbols by (mostly) LaTeX names. The mode
is persistent: it stays active after committing a symbol until you
switch category (e.g. `Esc` or the hangul toggle key).

* `\` starts a symbol name. Type the name, then commit with
  `Enter`/`Tab`: `\pi` → π, `\Pi` → Π (names are case-sensitive).
* Keys typed while no symbol name is open are committed as normal latin
  characters.
* `\\` commits a literal backslash `\`.
* `Backspace` removes the last character of the name; when the name is
  empty it closes symbol entry.
* A style prefix can be put before the name as `\<style>.<name>`. The
  styles are `sf` (sans-serif), `bf` (bold), `it` (italic), `tt`
  (monospace), `bb` (double-struck), `scr` (script), `cal`
  (calligraphic) and `frak` (fraktur), and they can be concatenated in
  any order: `\bfit.alpha` → 𝜶.
* An unknown name commits nothing. An unparsable style prefix is
  silently ignored and the unstyled symbol is committed instead.
* Some non-LaTeX names also exist, e.g. `\squotl` → 「. See
  [symbol_map.json] for the full list.

[symbol_map.json]: ../src/engine/dict/data/symbol_map.json

## Emoji

Emoji mode searches emoji by name.

* Type part of the emoji's English name — candidates are matched by
  substring against the English [Unicode CLDR][cldr] annotation names,
  so Korean input won't match anything.
* The preedit shows the query followed by up to 5 matching candidates.
* `Space` is part of the query (e.g. `red apple`).
* `Enter`/`Tab` commits the first candidate and leaves the mode.
* `Backspace` removes the last character of the query; when the query is
  empty it leaves the mode.

[cldr]: https://cldr.unicode.org

## Hanja

Hanja mode converts the hangul text you are composing into hanja, so it
only works in the `Hangul` category while a preedit exists (e.g. type 한
then press `F9`).

* It opens the `kime-candidate-window` popup listing the candidates with
  their meanings. The `kime-candidate-window` binary must be installed
  in `PATH`.
* Click a candidate with the mouse to commit it; `Esc` closes the popup
  without converting.

The popup's keyboard handling is currently minimal and is being
reworked.
