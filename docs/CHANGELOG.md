# CHANGELOG

## Dev

### Breaking

### Improve

## 1.3.0

### Improve

* Add more keycodes (Enter, Tab, ControlL, ControlR, Delete, AltL, F1-F12, HangulHanja)
* Split kime-window into other repo
* Fix gtk preedit bug [#325](https://github.com/Riey/kime/issues/325)
* Support emoji, hanja input
* Add desktop files

## 1.2.0

### Improve

* Fix kime-check fail [#307](https://github.com/Riey/kime/issues/307)
* Fix preedit string disapear when press hotkey [#310](https://github.com/Riey/kime/issues/310)
* Make character typing order strict `ㅏ + ㄱ = ㅏㄱ`
* Add `TreatJongseongAsChoseong` addon
* Add `FlexibleComposeOrder` addon [#318](https://github.com/Riey/kime/issues/318)
* Check LANG env in kime-check [#317](https://github.com/Riey/kime/issues/317)
* Add `Commit` hotkey, `ConsumeIfProcessed` hotkey result [#315](https://github.com/Riey/kime/issues/315)
* Add white icon [#316](https://github.com/Riey/kime/issues/316)

## 1.1.3

### Improve

* Fix cho-jung bug `웬 + ㅊ$ㅜ = 웬ㅊ`
* Fix composable jungseong bug `ㅇ + $ㅆ$ㅜ + $ㅊ$ㅔ +  = 웇`
* Fix xim bug [#304](https://github.com/Riey/kime/issues/304)

## 1.1.2

### Improve

* Support word commit [#288](https://github.com/Riey/kime/issues/288)
* Make qt preedit string have underline style
* Make keycode 130 to Hangul [#291](https://github.com/Riey/kime/issues/291)
* Implement composition jungseong [#295](https://github.com/Riey/kime/issues/295)

## 1.1.1

### Improve

* Correct xim set_event_mask [#283](https://github.com/Riey/kime/issues/283)
* Detect mouse click event on gtk, qt [#282](https://github.com/Riey/kime/issues/282) [#280](https://github.com/Riey/kime/issues/280)
* Release qt6 binary in latest zst [#281](https://github.com/Riey/kime/issues/281)

## 1.1.0

### Breaking

* Now frontends check engine's version if it failed, must cause hard error
* Use xim async only it will break some apps like neovide but fix many wrong bevaivors and performance

### Improve

* Now release debian, ubuntu, arch binary package and many variants of qt module
* Add debian-buster, ubuntu-18.04 docker
* New tool `kime-check` for diagnostic kime [#270](https://github.com/Riey/kime/issues/270)
* Correct invalid sebolsik-390 builtin layout [#261](https://github.com/Riey/kime/issues/261)
* Fix indicator can't load icon data [#260](https://github.com/Riey/kime/issues/260)
* Fix choseong compose bug [#263](https://github.com/Riey/kime/issues/263)

## 1.0.3

* Rollback [#247](https://github.com/Riey/kime/issues/247)

## 1.0.2

* Fix xim freeze when typing fast [#251](https://github.com/Riey/kime/issues/251)
* Fix global config path [#252](https://github.com/Riey/kime/issues/252)

## 1.0.1

* Fix xim doesn't work [#246](https://github.com/Riey/kime/issues/246)

## 1.0.0

* Fix debian packaging [#140](https://github.com/Riey/kime/issues/140)
* Use shell script for Build [#231](https://github.com/Riey/kime/issues/231)
* Using C++ header for engine cffi [#229](https://github.com/Riey/kime/issues/229)
* Implement engine hotkeys [#223](https://github.com/Riey/kime/issues/223)
* Implement sebeolsik-sin1995 [#235](https://github.com/Riey/kime/issues/235)
* Implement layout_addons [#239](https://github.com/Riey/kime/issues/239)
* Fix gtk reset doesn't commit preedit char [#240](https://github.com/Riey/kime/issues/240)
* Compile C/C++ with `-fvisibility=hidden` [#241](https://github.com/Riey/kime/issues/241)
* Change release .tar.xz [#203](https://github.com/Riey/kime/issues/203)
* Make engine capi no panic [#201](https://github.com/Riey/kime/issues/201)
* Fix wayland focus change repeat bug [#207](https://github.com/Riey/kime/issues/207)
* Support ALT modifier [#190](https://github.com/Riey/kime/issues/190)
* Fix wayland doesn't close fd well [#194](https://github.com/Riey/kime/issues/194)
* Using xim sync mode [49d0ef3e](https://github.com/Riey/kime/commit/49d0ef3e0b473378881a396f394db09bff0d2a81)
* Improve indicator [#186](https://github.com/Riey/kime/issues/186)
* Workaround patch with xwayland input focus bug ([#137](https://github.com/Riey/kime/issues/137))
* Handle disabled key repeat properly by **[@simnalamburt]** ([#188](https://github.com/Riey/kime/issues/188))
* Fix unwanted key repeat bug on wayland by **[@simnalamburt]** ([#206](https://github.com/Riey/kime/issues/206))
* Fix preedit string sended to wrong client by **[@simnalamburt]** ([#205](https://github.com/Riey/kime/issues/205))
* Fix the key repeat regression by **[@simnalamburt]** ([#215](https://github.com/Riey/kime/issues/215))
* Fix wrong behavior in neovide ([#179](https://github.com/Riey/kime/issues/179))
* Fix xim crash when typing fast ([#170](https://github.com/Riey/kime/issues/170))
* Fix xim doesn't work not en_US locale ([#177](https://github.com/Riey/kime/issues/177))
* Key repeat implemented in wayland frontend by **[@simnalamburt]** ([#171](https://github.com/Riey/kime/issues/171))
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

[@simnalamburt]: https://github.com/simnalamburt

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
