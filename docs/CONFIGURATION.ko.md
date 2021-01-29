# 설정

`/etc/kime/config.yaml`에 기본값으로 설정된 예시 파일이 있습니다. [default_config.yaml](default_config.yaml)에서 온라인으로 볼 수도 있습니다. `~/.config/kime/config.yaml`이나 `$XDG_CONFIG_HOME/kime/config.yaml`에 `/etc/kime/config.yaml`의 내용을 복사해 보세요.

## layout

키보드 자판을 설정합니다. `dubeolsik`(두벌식), `sebeolsik-390`(세벌식 390), `sebeolsik-391`(세벌식 391)이 기본으로 내장되어 있습니다. `$XDG_CONFIG_HOME/kime/layouts/`에 여기에 없는 키보드 자판을 YAML 파일로 직접 만들 수도 있습니다. [dubeolsik.yaml]을 참고해 보세요.

[dubeolsik.yaml]: engine/core/data/dubeolsik.yaml

| 기본값 |`dubeolsik`|
|--------|-----------|

## esc_turn_off

`ESC` 버튼을 누르면 영문 모드로 전환됩니다. vim을 쓸 때 유용한 기능입니다.

| 기본값 |`true`|
|--------|------|

## hangul_keys

한/영 모드를 전환하는 데 사용할 키들입니다.

| 기본값 |`[Hangul, Muhenkan, AltR, Super-Space]`|
|--------|---------------------------------------|

## xim_preedit_font

XIM에서 쓸 편집창 글꼴과 크기입니다.

| 기본값 |`[D2Coding, 15.0]`|
|--------|------------------|

## compose

자모 합성/분해 방식을 조정합니다.

### compose_choseong_ssang

같은 자음을 두 번 누를 때 쌍자음을 합성합니다.

```
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

#### default

| 기본값 |`false`|
|--------|-------|

### compose_jungseong_ssang

```
ㅑ + ㅣ = ㅒ
ㅕ + ㅣ = ㅖ
```

#### default

| 기본값 |`false`|
|--------|-------|


### decompose_jungseong_ssang

#### default

| 기본값 |`false`|
|--------|-------|

### compose_jongseong_ssang

```
ㄱ + ㄱ = ㄲ
ㅅ + ㅅ = ㅆ
```

#### default

| 기본값 |`false`|
|--------|-------|

### decompose_jongseong_ssang

#### default

| 기본값 |`false`|
|--------|-------|
