# 0.5.2

* Don't reset on XIM set_ic_values

# 0.5.1

* Fix XIM modifier bug

# 0.5.0

* Now `dubeolsik`, `sebeolsik-390`, `sebeolsik-391` layouts are embeded and no need local file
* Fix intellij issue
* Fix Super key bug
* Package deb file 

# 0.4.1

* Fix unhandled keycode doesn't occur reset 

# 0.4.0

* Create CHANGELOG
* Decrease binary sizes with make engine cdylib
* Reset when focus_out (XIM)
* Add compose config
* Fix XIM start bug when XIM_SERVER is not set
* Support Control modifiers
* Support Super modifiers
* Commit forwarded events

# 0.3.0

* Add `compose_ssangjaum` config
* Allow `Hangul` key
* Reset when focus_out (GTK3)
* Fix firefox backspace bug
* Fix firefox enter, esc bug

# 0.2.1

* Bypass shift keys

# 0.2.0

* Support compose jungseong (ㅑ + ㅣ = ㅒ)
* Fix jongseong to next choseong when jungseong is entered (옹 + ㅏ = 오아)
* Support ESC to disable hangul mode for VIM users
* Bypass ctrl chars
* Implement config file
* Reset when unhandled keysym is entered
* Add sebeolsik 390, 391 layouts
