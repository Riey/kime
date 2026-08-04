# CHANGELOG

## Unreleased

### Breaking

### Improve

* fix(xim): GTK3 apps on the XIM bridge (`GTK_IM_MODULE=xim`) no longer hang on the first composing key — such clients draw the preedit themselves and report the moved cursor back with `SetICValues(XNSpotLocation)`, which kime answered by redrawing the preedit, moving the cursor again: an endless PreeditDone/PreeditStart/PreeditDraw echo at ~100k messages per second that never committed. The spot location is now only acted on for the server-drawn preedit popup [#783](https://github.com/Riey/kime/issues/783) [#786](https://github.com/Riey/kime/pull/786)
* fix(wayland): kime no longer dies when the seat's keyboard has no keymap — the compositor reports it as an empty `no_keymap` keymap, which kime forwarded verbatim to its own virtual keyboard until the compositor killed the connection with a `no memory` error (naive headless/CI setups never got an input method). Empty keymaps are now skipped, as are key and modifier events sent before a real keymap arrives [#782](https://github.com/Riey/kime/issues/782) [#789](https://github.com/Riey/kime/pull/789)
* fix(wayland): only send the input-method-v2 `commit` request when protocol state actually changed — a bare commit on every key (still triggered by modifier and arrow keys) applied an empty-preedit transaction whose spurious `done` made Firefox/Chromium replace the current selection with an empty composition [#714](https://github.com/Riey/kime/issues/714) [#654](https://github.com/Riey/kime/issues/654) [#772](https://github.com/Riey/kime/pull/772)
* fix(wayland): key repeat now crosses the preedit/text boundary correctly — repeat ticks the engine no longer consumes (e.g. holding Backspace past the preedit) are forwarded to the client as synthetic release+press events at kime's uniform repeat rate, instead of stopping entirely (3.2.0 regression) or letting the compositor restart its full repeat delay (the original complaint) [#666](https://github.com/Riey/kime/issues/666) [#772](https://github.com/Riey/kime/pull/772)
* fix(gtk): GTK4 no longer swallows bypassed keys (Enter, Tab, arrows) while a preedit is visible — the GTK3-style deferred event re-injection can't reach the widget on GTK4, so the GTK4 build now updates the preedit synchronously and lets the widget handle bypassed control keys on the first press (Enter no longer needs to be pressed twice in Hangul mode) [#606](https://github.com/Riey/kime/issues/606) [#775](https://github.com/Riey/kime/pull/775)
* fix(gtk): harden IM context lifetime — hold a reference across signal emission so an app destroying the context from a `commit` handler (Inkscape tool teardown) can't leave dangling pointers, disconnect the button-press handler in finalize so a widget outliving the context can't call into freed memory, and chain up to the parent class finalize [#579](https://github.com/Riey/kime/issues/579) [#775](https://github.com/Riey/kime/pull/775)
* docs(README): document snap/flatpak sandbox limitation, im-config under Wayland+zsh, and GNOME Wayland status [#423](https://github.com/Riey/kime/issues/423) [#767](https://github.com/Riey/kime/pull/767)
* docs: document the Math/Hanja/Emoji input modes, the hotkey key format and `candidate_font`; fix the documented `xim_preedit_font` default (`Noto Sans CJK KR`, not `D2Coding`) [#671](https://github.com/Riey/kime/issues/671) [#583](https://github.com/Riey/kime/issues/583) [#572](https://github.com/Riey/kime/issues/572) [#773](https://github.com/Riey/kime/pull/773)
* fix(qt): track `NOT_READY` so focus loss to the hanja candidate window no longer resets the engine and kills the popup [#757](https://github.com/Riey/kime/issues/757) [#771](https://github.com/Riey/kime/pull/771)
* fix(engine): hotkey lookup falls back to the key without its own modifier bit — Wayland delivers a modifier key's press with its own modifier already set (X11 reports the pre-event state), so plain `AltR`/`ControlR` hotkeys never fired in Wayland-native apps (hangul toggle in Konsole and other Qt apps on KDE Plasma). Exact bindings such as `M-AltR` keep priority over the fallback; the redundant `M-AltR` default hotkey from [#719] is removed [#760](https://github.com/Riey/kime/pull/760)
* fix(engine): volume keys (Mute/VolumeDown/VolumeUp) no longer act as Hangul/Hanja keys — hardware keycodes 121/122/123 were raw evdev values added by mistake; only the real X11 keycodes 130 (`<HNGL>`) and 131 (`<HJCV>`) map now [#603](https://github.com/Riey/kime/issues/603) [#769](https://github.com/Riey/kime/pull/769)
* fix(engine): reap the candidate window process after killing it so dismissed hanja popups no longer accumulate as zombie (`<defunct>`) processes [#617](https://github.com/Riey/kime/issues/617) [#769](https://github.com/Riey/kime/pull/769)
* fix(engine): log config file open/parse errors instead of silently falling back to the default config, so a syntax error in `config.yaml` is diagnosable [#656](https://github.com/Riey/kime/issues/656) [#769](https://github.com/Riey/kime/pull/769)
* fix(wayland): an unchanged preedit is no longer re-sent — the engine reports `HAS_PREEDIT` for the keys it bypasses, so while composing every lone modifier or volume key produced another identical `set_preedit_string` + `commit` and a duplicate `preedit-changed` event at the client; a transaction is now sent only when it carries a commit string or a preedit that differs from the one the client already shows [#781](https://github.com/Riey/kime/issues/781) [#788](https://github.com/Riey/kime/pull/788)
* fix(qt5): pass the input context IID define to moc and the compiler so Qt5 apps load the kime input context again — broken since the meson migration ([#747]) which fixed only qt6 ([#756]) [#778](https://github.com/Riey/kime/issues/778) [#785](https://github.com/Riey/kime/pull/785)
* feat(engine): layout files support an optional `version:`/`keys:` format — files declaring a format version newer than kime supports are rejected with a clear error instead of breaking silently on a future format change, legacy flat-map layouts keep working unchanged, and user layout files that fail to load are logged with the reason instead of silently skipped [#540](https://github.com/Riey/kime/issues/540) [#774](https://github.com/Riey/kime/pull/774)
* fix(engine): dismissing the hanja candidate window with a global hotkey no longer leaks the popup process — Esc, the hangul/latin toggles and the emoji/math mode hotkeys all stay active in hanja mode, and they cleared the input mode without closing its candidate window, leaving `kime-candidate-window` running (and later `<defunct>`) [#780](https://github.com/Riey/kime/issues/780) [#787](https://github.com/Riey/kime/pull/787)
* fix(engine): a layout or translation-layer file naming a key with a modifier other than Shift no longer crashes kime — `KeyMap` keeps one slot per keycode for the unmodified and Shift variants, and `insert` indexed that two-element array with the raw modifier bits, so a single `C-T: <` line panicked with an index-out-of-bounds while loading. Such a key can never be looked up (`get` rejects it), so the entry is now dropped and the rest of the file keeps working [#793](https://github.com/Riey/kime/issues/793) [#795](https://github.com/Riey/kime/pull/795)
* test(fuzz): add a cargo-fuzz suite (`fuzz/`) fuzzed nightly in CI with a corpus that persists across runs — arbitrary key/op sequences against the engine (asserting the `HAS_COMMIT`/`HAS_PREEDIT` flags match the buffers), no-panic targets for the layout and config YAML parsers, and a differential target comparing kime's dubeolsik composition against libhangul, the engine behind ibus-hangul and fcitx5-hangul [#792](https://github.com/Riey/kime/pull/792)
* fix(gtk): a client calling `gtk_im_context_reset()` from inside its `commit` handler (the [#562] Firefox pattern) no longer gets the commit delivered twice — the immodule emitted the signal before clearing the engine's commit buffer and `reset` re-read it, so mid-word the re-entrant reset also swallowed the live preedit and destroyed the composition (오랜 became 오오래ㄴ). The buffer is now snapshotted and cleared before emitting, so a re-entrant reset only commits the live preedit; the [#563] `is_committing` guard for this was removed in [#570] [#562](https://github.com/Riey/kime/issues/562) [#799](https://github.com/Riey/kime/pull/799)
* fix(qt): the Qt input context (shared by the qt5 and qt6 plugins) had the same [#562]-class hole — it emitted the commit `QInputMethodEvent` through the synchronous `QCoreApplication::sendEvent` before clearing the engine's commit buffer, and `reset()` re-read it, so a widget calling `QInputMethod::reset()` from its `inputMethodEvent()` received the text twice. The buffer is now snapshotted and cleared before emitting, and a reset with nothing pending no longer sends an empty `QInputMethodEvent` [#562](https://github.com/Riey/kime/issues/562) [#799](https://github.com/Riey/kime/pull/799)
* fix(qt): the hanja candidate window no longer dies on focus-out — this completes the [#771] fix, which guarded `setFocusObject` but not `commit()`: Qt calls `commit()` first on focus-out, so the unconditional reset there still killed the just-spawned popup and discarded the syllable being converted. `commit()` now honours the same `engine_ready` guard, leaving normal commit-on-focus-out unchanged [#757](https://github.com/Riey/kime/issues/757) [#779](https://github.com/Riey/kime/issues/779) [#784](https://github.com/Riey/kime/pull/784)
* fix(wayland): held keys repeat through the engine again on compositors that deliver the repeats themselves — a `wl_seat` v10 compositor (KWin 6.7) announces `repeat_info(rate=0)`, meaning the client must not run its own repeat, and sends every repeat as a `wl_keyboard.key` event in the `repeated` state. kime tested for that state as `WEnum::Unknown(2)`, but `wayland-client` has carried the enum variant since wayland 1.24 and decodes it as `KeyState::Repeated`, so a repeat matched neither the pressed nor the released branch and fell through to the raw-key bypass. Bypassing is what Latin mode wants anyway, so only Hangul mode broke: holding a key emitted the literal QWERTY letters next to a stuck preedit (`ㄱrrrrrrrrrr`) instead of repeating the composed jamo, and with `rate=0` disabling kime's own repeat timer there was no fallback [#802](https://github.com/Riey/kime/issues/802) [#803](https://github.com/Riey/kime/pull/803)

## 3.2.0

### Breaking

### Improve

* fix(qt6): pass the input context IID define to moc so the Qt6 immodule plugin metadata is generated correctly **[@isac322]** [#756](https://github.com/Riey/kime/pull/756)
* fix(hangul): revert [#719] — pass keys (layout symbol/number layers such as the sebeolsik shifted right-half) are committed in Hangul mode again instead of being bypassed to the application. Shortcut-modified keys (Ctrl/Alt/Super) have no layout entry and are already passed through by the caller, so the special-case bypass was unnecessary and regressed every Hangul layout. [#754](https://github.com/Riey/kime/issues/754)
* Replace cmake build system with meson
  - cargo builds via `custom_target()`, GTK/Qt frontends as `shared_library()`
  - Per-component build options (gtk3, gtk4, qt5, qt6, check, indicator, etc.), enabled by default
  - `kime` and other binaries build by default when running `ninja`
  - `system_engine` mode for building frontends against system-installed engine
  - Qt plugin dir override options for nix compatibility
* Remove EOL distro Dockerfiles (ubuntu 18.04~22.10, debian buster)
* Add debian-trixie Dockerfile
* Update release workflow: Qt5/Qt6 multi-version matrix builds, softprops/action-gh-release@v2
* nix: switch to fetchCargoVendor, parameterize frontend toggles
* Fix mismatched cargoDeps in nix and update attribute syntax **[@nakoo]**
* Updated NixOS configuration example to match updated attribute syntax.
* Remove `kime-engine-cffi`
* fix(wayland input_method_v2): not return unwarp **[@racakenon]** [#715](https://github.com/Riey/kime/pull/715)
* feat(engine): Let default `Alt_R` hotkey accept `Alt` modifier [#719](https://github.com/Riey/kime/issues/719)
* fix(hangul): Don't consume pass keys (numbers, symbols) so app shortcuts like `@`/`#` fire in Hangul mode [#719](https://github.com/Riey/kime/issues/719)
* Add Opensuse Build Service repository and modify README
* fix(xim): handle None from from_hardware_code without panic [#721](https://github.com/Riey/kime/issues/721)
* Update dependencies:
  - wayland-client 0.29 → 0.31, wayland-protocols 0.29 → 0.32
  - wayland-protocols-misc 0.3 (신규), xdg 2.5 → 3.0, quick-xml 0.27 → 0.39
  - xkbcommon 0.7 → 0.9, ksni 0.2 → 0.3.3 + tokio
  - bitflags 2.10, nix 0.30, strum 0.27
  - x11rb 0.13, xim 0.5, image 0.25, imageproc 0.26
  - mio 1.0 with timerfd-mio, egui/eframe 0.33, itertools 0.14
  - bindgen 0.72.1, cbindgen 0.29.2
  - Replace rusttype with ab_glyph
  - Replace ansi_term with owo-colors
  - Replace daemonize with nix daemon
  - Replace unic with unicode-properties
* Remove unused dependencies:
  - Remove zwp-virtual-keyboard (merged into wayland-protocols-misc)
* Fix kime-wayland crash on KDE Plasma 6.5.5 by handling KeyState::Repeated
* Fix indicator tray icon not showing/updating on KDE with tokio async I/O (ksni 0.3.3)
* Rewrite kime-wayland for wayland-rs 0.31 API (Dispatch trait pattern)
* fix(xim): degrade gracefully when the preedit font is missing instead of panicking [#706](https://github.com/Riey/kime/issues/706)
* feat(engine): add `SuperL`/`SuperR` keycodes so Right-Super can be bound as a hotkey [#640](https://github.com/Riey/kime/issues/640)
* fix(hangul): enable `ComposeJongseongSsang` by default for sebeolsik-3sin-p2 (ㄲ 받침) [#646](https://github.com/Riey/kime/issues/646)
* fix(latin): repair the malformed Dvorak layout so it parses and works [#626](https://github.com/Riey/kime/issues/626)
* fix(hangul): Sebeolsik composition bug when typing `v` instead of `/` [#679](https://github.com/Riey/kime/issues/679)
* Qt6 immodule: append `.5.1` suffix to the plugin IID so it builds without `qt5-base` installed (fixes Arch/AUR build) [#732](https://github.com/Riey/kime/issues/732) [#736](https://github.com/Riey/kime/pull/736)
* Add `qt5` dependency to fix the frontend build failure on Arch Linux [#724](https://github.com/Riey/kime/pull/724)
* Release: build Qt6 immodules per-version like Qt5 and add Qt 6.9 / 6.10 to the build matrix [#749](https://github.com/Riey/kime/pull/749)
* Packaging: include gtk4 and qt6 immodules in the `.deb` packages, including ubuntu-22.04 [#734](https://github.com/Riey/kime/issues/734) [#750](https://github.com/Riey/kime/pull/750)
* Build Dockerfiles for Ubuntu 24.04 and Debian Bookworm [#726](https://github.com/Riey/kime/pull/726)
* fix(wayland): prevent text selection deletion on Ctrl/modifier shortcuts [#714](https://github.com/Riey/kime/issues/714) [#718](https://github.com/Riey/kime/pull/718)
* fix(wayland): bypass key events when input is not activated [#745](https://github.com/Riey/kime/pull/745)


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
* Fix preedit string sent to wrong client by **[@simnalamburt]** ([#205](https://github.com/Riey/kime/issues/205))
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
[@racakenon]: https://github.com/racakenon
[@strictpvp]: https://github.com/strictpvp/
