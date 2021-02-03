# CHANGELOG

## Dev

* Fix wrong behavior in neovide ([#179])
* Fix xim crash when typing fast ([#170])
* Fix xim doesn't work not en_US locale ([#177])
* Key repeat implemented in wayland frontend by **[@simnalamburt]** ([#171])

[@simnalamburt]: https://github.com/simnalamburt
[#171]: https://github.com/Riey/kime/pull/171

## 1.0.0-pre1

* Add more help messages for binary tools
* Show more version info
* Make CONFIGURATION.md more newbie friendly
* Create Korean version of documents
* Show hangul/english state on tray icon
* Support global hangul state
* Set gtk log domain to `kime`
* Let `kime-wayland` exit when IO Error occured
* Fix wayland input bug
* `kime-xtask` now read `KIME_CARGO_ARGS`, `KIME_CMAKE_ARGS`, `KIME_NINJA_ARGS`

## 0.9.1

* Add donation link
* Fix gtk link error

## 0.9.0

* Rework build scripts
* Fix NumLock bug again
* Add GTK2, GTK4 immodule
* Add Qt6 immodule
* Add Wayland frontend

## 0.8.1

* Fix GTK3 space commit bug
* Fix Qt5 backspace bug
* Use C in GTK3

## 0.8.0

* Add Qt5 immodule
* Complete moum backspace (e.g. ㅚ -> ㅗ, ㅞ -> ㅜ)

## 0.7.0

* Fix hangul bug on NumLock, CapLock, ScrollLock
* Fix xim preedit window delete bug
* Fix deb file
* Add `--log` option in `kime-xim`
* Add size in `xim-preedit-font`

## 0.6.0

* Don't reset on XIM set_ic_values
* Redraw when XIM preedit state changed
* Update packaging scripts
* Add Install guide
* Now kime-xim print version when pass `--version`

## 0.5.1

* Fix XIM modifier bug

## 0.5.0

* Now `dubeolsik`, `sebeolsik-390`, `sebeolsik-391` layouts are embeded and no need local file
* Fix intellij issue
* Fix Super key bug
* Package deb file

## 0.4.1

* Fix unhandled keycode doesn't occur reset

## 0.4.0

* Create CHANGELOG
* Decrease binary sizes with make engine cdylib
* Reset when focus_out (XIM)
* Add compose config
* Fix XIM start bug when XIM_SERVER is not set
* Support Control modifiers
* Support Super modifiers
* Commit forwarded events

## 0.3.0

* Add `compose_ssangjaum` config
* Allow `Hangul` key
* Reset when focus_out (GTK3)
* Fix firefox backspace bug
* Fix firefox enter, esc bug

## 0.2.1

* Bypass shift keys

## 0.2.0

* Support compose jungseong (ㅑ + ㅣ = ㅒ)
* Fix jongseong to next choseong when jungseong is entered (옹 + ㅏ = 오아)
* Support ESC to disable hangul mode for VIM users
* Bypass ctrl chars
* Implement config file
* Reset when unhandled keysym is entered
* Add sebeolsik 390, 391 layouts
