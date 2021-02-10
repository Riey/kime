# kime

한글 입력기

[English](./README.md), [한국어](./README.ko.md)

[<img alt="build" src="https://img.shields.io/github/workflow/status/Riey/kime/CI?style=for-the-badge" height="25">](https://github.com/Riey/kime/actions?query=workflow%3ACI)
[<img alt="discord" src="https://img.shields.io/discord/801107569505992705.svg?style=for-the-badge" height="25">](https://discord.gg/YPnEfZqC6y)
[<img alt="release version" src="https://img.shields.io/github/v/release/Riey/kime?style=for-the-badge" height="25">](https://github.com/Riey/kime/releases)
[<img alt="aur version" src="https://img.shields.io/aur/version/kime?style=for-the-badge" height="25">](https://aur.archlinux.org/packages/kime/)
[<img alt="license" src="https://img.shields.io/github/license/Riey/kime?style=for-the-badge" height="25">](https://github.com/Riey/kime/blob/master/LICENSE)
[<img src="https://d1u4yishnma8v5.cloudfront.net/mobile-gift.png" alt="donaricano-btn" height="50">](https://donaricano.com/mypage/1610220543_mjZDXO)

## [Changelog](docs/CHANGELOG.md)

## kime을 써야 하는 이유?

* 잘 테스트된 입력 엔진
* 빠른 [속도](https://github.com/Riey/kime/wiki/Performance)
* 적은 메모리 사용량
* 대부분의 코드가 세그멘테이션 오류가 없는 Rust로 작성됨
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

최신 릴리스는 [kime](https://aur.archlinux.org/packages/kime) 만약 소스에서 빌드하시려면 [kime-git](https://aur.archlinux.org/packages/kime-git)에서 설치할 수 있습니다.

### 데비안

[releases](https://github.com/Riey/kime/releases) 탭에 있는 .deb 파일을 설치할 수 있습니다.

### 소스에서 빌드하기

빌드하기 전에 **cargo** 및 아래 나열되어 있는 기타 종속성이 설치되어 있는지 확인하세요.

```sh
git clone https://github.com/Riey/kime
cd kime

scripts/build.sh -ar
```

이제 모든 파일들은 build/out 경로에 있습니다 만약 수동설치를 원하시면 쓰시면 됩니다

`scripts/install.sh <install-prefix>` 스크립트를 쓸 수도 있습니다 패키징할때 유용합니다

`scripts/release-deb.sh <deb-out-path>` 스크립트를 사용하시면 `deb` 파일을 생성합니다.

#### GTK

```sh
# GTK2 설치 시
sudo gtk-query-immodules-2.0 --update-cache
# GTK3 설치 시
sudo gtk-query-immodules-3.0 --update-cache
# GTK4 설치 시
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

만약 X를 사용하신다면 .xprofile에서 실행하실 수 있습니다.

또 언제든 `kime-indicator`를 실행하면 한영 상태를 appindicator에서 볼 수 있습니다.

자세한 옵션은 [CONFIGURATION.md](docs/CONFIGURATION.ko.md)를 참고하세요.

## 종속성 목록

참고로 필요하신 종속성만 있으면 됩니다
예를들어 qt6를 사용하지 않으신다면 필요하지 않습니다.

* gtk2
* gtk3
* gtk4
* qt5
* qt6
* libappindicator-gtk3 (indicator)
* libxcb (xim)
* cairo (xim)
