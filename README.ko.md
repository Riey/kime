# kime

한글 입력기

[English](./README.md), [한국어](./README.ko.md)

[<img alt="discord" src="https://img.shields.io/discord/801107569505992705.svg?style=for-the-badge" height="25">](https://discord.gg/YPnEfZqC6y)
[<img alt="build status" src="https://img.shields.io/github/workflow/status/Riey/kime/CI/master?style=for-the-badge" height="25">](https://github.com/Riey/kime/actions?query=workflow%3ACI)
[<img alt="release version" src="https://img.shields.io/github/v/release/Riey/kime?style=for-the-badge" height="25">](https://github.com/Riey/kime/releases)
[<img alt="aur version" src="https://img.shields.io/aur/version/kime?style=for-the-badge" height="25">](https://aur.archlinux.org/packages/kime/)
[<img alt="license" src="https://img.shields.io/github/license/Riey/kime?style=for-the-badge" height="25">](https://github.com/Riey/kime/blob/master/LICENSE)
[<img src="https://d1u4yishnma8v5.cloudfront.net/mobile-gift.png" alt="donaricano-btn" height="50">](https://donaricano.com/mypage/1610220543_mjZDXO)


## [Changelog](docs/CHANGELOG.md)

## kime을 써야 하는 이유?

* 잘 테스트된 입력 엔진
* 작은 메모리 사용량
* 세그멘테이션 오류가 없는 Rust로 대부분 작성됨
* 사용자 설정 자판 지원

## 지원되는 프론트엔드

- [x] XIM
- [x] Wayland
- [x] GTK2
- [x] GTK3
- [x] GTK4
- [x] Qt5
- [x] Qt6

## 설치

### 아치 리눅스

최신 릴리스는 [kime](https://aur.archlinux.org/packages/kime) 만약 소스에서 빌드를 하시려면 [kime-git](https://aur.archlinux.org/packages/kime-git)에서 설치 할 수 있습니다.

### 데비안

[releases](https://github.com/Riey/kime/releases) 탭에 있는 .deb 파일를 설치할 수 있습니다.

### 소스에서 빌드하기

빌드하기 전에 **cargo** 및 아래 나열되어 있는 기타 종속성이 설치되어 있는지 확인하세요.

```sh
git clone https://github.com/Riey/kime
cd kime

cargo xtask build XIM GTK3 QT5

# 이제 build/out에서 파일을 설치할 수 있습니다.
# 아니면 install을 사용하세요.
# cargo xtask install <target-path>
# 또는 데비안 유저이면, release-deb를 사용할 수 있습니다.
# cargo xtask release-deb <deb-out-path>
```

자세한 내용은 `cargo xtask --help`를 참고하세요.

#### GTK

```sh
# 만약 gtk2를 설치하려면
sudo gtk-query-immodules-2.0 --update-cache
# 만약 gtk3를 설치하려면
sudo gtk-query-immodules-3.0 --update-cache
# 만약 gtk4를 설치하려면
sudo gio-querymodules /usr/lib/gtk-4.0/4.0.0/immodules
```

## 설정

init 스크립트에 다음을 추가하세요.

```sh
export GTK_IM_MODULE=kime
export QT_IM_MODULE=kime
export XMODIFIERS=@im=kime
```

그리고 세션 초기화 후 `kime-xim` 또는 `kime-wayland` 바이너리를 실행합니다.

만약 X를 사용히사면 .xprofile에서 실행을 하실 수 있습니다.

자세한 옵션은 [CONFIGURATION.md](docs/CONFIGURATION.ko.md)를 참고하세요.

## 종속성 목록

### XIM

* libxcb
* cairo

### 다른 툴킷들의 immodule을 쓰는 경우

* 해당 툴킷(예: gtk3, qt5 등)
