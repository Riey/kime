# CHANGELOG

## Unreleased

### Breaking

### Improve
- Fix mismatched cargoDeps in nix and update attribute syntax **[@nakoo]**

## 3.1.1

### Improve

* Update Unicode CLDR version to 45 [#674](https://github.com/Riey/kime/issues/674)

## 3.1.0

### Improve

* Add ubuntu-22.10 Dockerfile **[@OctopusET]**
* Fix KDE autostart [#576](https://github.com/Riey/kime/issues/576)
* Add unicode prime symbols to math mode. (prime, double prime, triple prime, quadruple prime)
* Fix to work on wlroots>=0.17.1 (Sway 1.9) [#664](https://github.com/Riey/kime/issues/664)
* Add wayland zwp_input_method_v1 support **[@Jhyub]**

## 3.0.2

### Improve

* Update configuration.md **[@Riey]** [#601](https://github.com/Riey/kime/issues/601)
* Correct scan code to properly recognize F11 and F12 key **[@xnuk]** [#602](https://github.com/Riey/kime/issues/602)
* Fix KDE plasmashell crash **[@kpqi5858]** [#609](https://github.com/Riey/kime/issues/609)
* Set default font to `Noto Sans CJK KR` **[@Riey]** [#618](https://github.com/Riey/kime/issues/618)
* Fix [#611](https://github.com/Riey/kime/issues/611) **[@Riey]** [#618](https://github.com/Riey/kime/issues/618)

## 3.0.1

### Improve

* Update default config file

## 3.0.0

### Breaking

* `FlexibleComposeOrder` can change jongseong order [#534](https://github.com/Riey/kime/issues/534)
* Implement hanja candidate window [#383](https://github.com/Riey/kime/issues/383)
* Builtin sebeolsik `3-90` and `3-91` changed with all jungseong to uncomposable except for `ㅑ`, `ㅕ`,`ㅡ`, `ㅜ(9)`, `ㅗ(/)` [#542](https://github.com/Riey/kime/issues/542)
* Respect NUMLOCK state [#591](https://github.com/Riey/kime/issues/591)
* Remove support GTK2
* ***Config file format has changed*** see [wiki](https://github.com/Riey/kime/wiki/3.0.0-Migration-guide) for more information

### Improve

* `FlexibleComposeOrder` can change compose jungseong order [#542](https://github.com/Riey/kime/issues/542)
* Fix preedit character error on chromium family [#535](https://github.com/Riey/kime/issues/535)
* Let incomplete character can commit multiple jamos
* Support johab encoding for preedit string
* Fix jongseong input bug `$ㅋㅕ + $ㅋㅕ = ㅋㅋ`
* Added Qt 5.12.9 library build
* Fix sebeolsik-391 "S-Equal" key
* Let indicator shown on Gnome tray (requires Gnome shell extension, https://extensions.gnome.org/extension/615/appindicator-support/)
* Fix space error in some firefox sites [#561](https://github.com/Riey/kime/issues/561).
* Fix duplicated commit string in some firefox sites [#562](https://github.com/Riey/kime/issues/562).
* Delaying preedit, bypass processes in gtk module [#570](https://github.com/Riey/kime/issues/570)
* Fix typo in symbol name for U+2193(↓): downaroow -> downarrow
* Adding translation layer feature [#586](https://github.com/Riey/kime/issues/586)

## 2.5.6

### Improve

* Update dependencies [#508](https://github.com/Riey/kime/issues/508)
* Fix sebeolsik-3sin-p2 '"' character [#509](https://github.com/Riey/kime/issues/509)
* Fix sebeolsik-391 "S-F" key [#521](https://github.com/Riey/kime/issues/521)
* Don't compose choseong when FlexibleComposeOrder is on [#520](https://github.com/Riey/kime/issues/520)
* Fix choseong converted into jongseong even `TreatJongseongAsChoseong` is off [#529](https://github.com/Riey/kime/issues/529)

## 2.5.5

### Improve

* Fix kime print outdated version [#506](https://github.com/Riey/kime/issues/506)

## 2.5.4

### Improve

* Fix indicator crash in autostart [#471](https://github.com/Riey/kime/issues/471)
* Fix ownership of files in deb package [#499](https://github.com/Riey/kime/issues/499)
* Fix sebeolsik-3sin-p2 'ㅌ' jongsung [#503](https://github.com/Riey/kime/issues/503)

## 2.5.3

### Breaking

* `--verbose` argument now deleted use `--log`

### Improve

* Fix CONFIGURATION typo [#484](https://github.com/Riey/kime/issues/484)
* Now indicator initial icon_color follow user config [#461](https://github.com/Riey/kime/issues/461)
* Don't exit xim when get `ServerError` [#23](https://github.com/Riey/kime/issues/23)
* Can set logging level either config file or command argument

## 2.5.2

### Improve

* Fix wrong symbol name (Gammma -> Gamma)
* Fix config loading in capi [#465](https://github.com/Riey/kime/issues/465)
* Fix key repeat bug on XIM [#467](https://github.com/Riey/kime/issues/467)

## 2.5.1

### Improve

* Fix indicator doesn't change icon properly [#457](https://github.com/Riey/kime/issues/457)

## 2.5.0

### Breaking

* Change config file layout
* Default config is no more installed just in the doc folder

### Improve

* Add `--kill` flag in kime daemon
* Install docs

## 2.4.0

### Improve

* Don't use git for check version [#441](https://github.com/Riey/kime/issues/441)
* Remove click event filter for Qt
* Support `kime` daemon [#440](https://github.com/Riey/kime/issues/440)

## 2.3.3

### Improve

* Really fix [#425](https://github.com/Riey/kime/issues/425)
* Support nix `shell.nix`, `default.nix`

## 2.3.2

### Improve

* Rollback `preferred_direct` config
* Fix qt preedit handling [#425](https://github.com/Riey/kime/issues/425)

## 2.3.1

### Improve

* Bypass shift input for shortcut [#418](https://github.com/Riey/kime/issues/418)
* Add `preferred_direct` config for some bugs [#425](https://github.com/Riey/kime/issues/425)

## 2.3.0

### Improve

* Install desktop files into autostart [#413](https://github.com/Riey/kime/issues/413)
* Improve hanja select ui with paging [#416](https://github.com/Riey/kime/issues/416)
* Mapping numpad numbers

## 2.2.1

### Improve

* Fix Home, End, PageUp, PageDown don't clear preedit [#410](https://github.com/Riey/kime/issues/410)

## 2.2.0

### Improve

* Support preedit string for XIM [#401](https://github.com/Riey/kime/issues/401)

## 2.1.5

### Improve

* Improve keymap lookup speed +50%

## 2.1.4

### Improve

* Detect qt mouse click for clear preedit [#400](https://github.com/Riey/kime/issues/400)
* Fix qt preedit style bug
* Try prevent hanja panic

## 2.1.3

### Improve

* Fix shift input error [#396](https://github.com/Riey/kime/issues/396)

## 2.1.2

### Improve

* Make unhandled key `Commit` and `Bypass`
* Add `Ignore` hotkey behaviour
* Optimize Layout
* New `array-keymap` optional feature that super fast but take more memory

## 2.1.1

### Improve

* Add more math symbol data [#385](https://github.com/Riey/kime/issues/385)
* Terminate previous servers in deb [#387](https://github.com/Riey/kime/issues/387)
* kime-indicator terminate previous process
* Set `Default`, `Insert` key commit
* Fix kime-check failed

## 2.1.0

### Improve

* Let Esc exit math mode [#379](https://github.com/Riey/kime/issues/379)
* Add font style specifier for math symbols [#377](https://github.com/Riey/kime/issues/377)
* Can't select hanja [#381](https://github.com/Riey/kime/issues/381)

## 2.0.1

### Improve

* Make more key to commit hangul [#373](https://github.com/Riey/kime/issues/373)

## 2.0.0

### Breaking

* Include english layouts [#347](https://github.com/Riey/kime/issues/347)
* Now change InputCategory clear preedit state
* Let select hanja, emoji in preedit string

### Improve

* Prevent double key press [#344](https://github.com/Riey/kime/issues/344)
* Using signal connect client's window instead of `gdk_window_add_filter`
* Insert emoji with `rofimoji`
* Embed hanja dict
* Support multiple architectures
* Hide desktop entries from DE menu and application launchers [#357](https://github.com/Riey/kime/pull/357)
* Integrate kime-indicator again
* Add `icon_color` config
* Add dvorak layout
* Replace `libappindicator` to `ksni`
* Now InputEngine implementations are split several crates for support many InputCategories
* Implement math backend

## 1.3.1

### Breaking

* Rename sebeolsik layouts

```txt
sebeolsik-390 -> sebeolsik-3-90
sebeolsik-391 -> sebeolsik-3-90
sebeolsik-sin1995 -> sebeolsik-3sin-1995
```

### Improve

* Add `TreatJongseongAsChoseongCompose` addon [#332](https://github.com/Riey/kime/issues/332)
* Fix 3-91 layout Z, S-Z bug [#335](https://github.com/Riey/kime/issues/335)
* Fix libreoffice-calc bug [#339](https://github.com/Riey/kime/issues/339)
* Add sebeolsik-3sin-p2 layout (except yet hangul) [#222](https://github.com/Riey/kime/issues/222)

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
* Fix preedit string disappear when press hotkey [#310](https://github.com/Riey/kime/issues/310)
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
* Let `kime-wayland` exit when IO Error occurred
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

* Now `dubeolsik`, `sebeolsik-390`, `sebeolsik-391` layouts are embedded and no need local file
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

<!-- Contributors -->

[@simnalamburt]: https://github.com/simnalamburt
[@xnuk]: https://github.com/xnuk
[@Riey]: https://github.com/Riey
[@kpqi5858]: https://github.com/kpqi5858
