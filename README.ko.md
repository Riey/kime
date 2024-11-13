# kime

[<img src="./docs/assets/kime-roundy-default-without-text-bluegrey.png" height="100">](https://github.com/Riey/kime)

한글 입력기

## 다른 언어로 보기

[English](./README.md), [**한국어**](./README.ko.md)

---

[<img alt="build" src="https://img.shields.io/github/actions/workflow/status/Riey/kime/ci.yaml?style=for-the-badge&branch=develop" height="25">](https://github.com/Riey/kime/actions?query=workflow%3ACI)
[<img alt="discord" src="https://img.shields.io/discord/801107569505992705.svg?style=for-the-badge" height="25">](https://discord.gg/YPnEfZqC6y)
[<img alt="release version" src="https://img.shields.io/github/v/release/Riey/kime?style=for-the-badge" height="25">](https://github.com/Riey/kime/releases)
[<img alt="aur version" src="https://img.shields.io/aur/version/kime?style=for-the-badge" height="25">](https://aur.archlinux.org/packages/kime/)
[<img alt="license" src="https://img.shields.io/github/license/Riey/kime?style=for-the-badge" height="25">](https://github.com/Riey/kime/blob/master/LICENSE)

## [Changelog](docs/CHANGELOG.md)

## kime을 써야 하는 이유?

* 잘 테스트된 입력 엔진
* 빠른 [속도](https://github.com/Riey/kime/wiki/Performance)
* 적은 메모리 사용량
* 대부분의 코드가 세그멘테이션 오류가 없는 Rust로 작성됨
* 사용자 설정 자판 지원

## 궁금한 게 있으신가요?

[디스코드](https://discord.gg/YPnEfZqC6y) 채널에 와서 연락하시거나 이슈를 올려주세요.

## 지원되는 프론트엔드

- [x] XIM
- [x] Wayland
- [x] GTK3
- [x] GTK4
- [x] Qt5
- [x] Qt6

## 설치

### NixOS

이 코드를 configuration.nix에 추가해주세요

```nix
i18n = {
  defaultLocale = "en_US.UTF-8";
  inputMethod = {
    enable = true;
    type = "kime";
    kime.config = {
      indicator.icon_color = "White";
    };
  };
};
```

### 아치 리눅스

최신 릴리스는 AUR의 [kime](https://aur.archlinux.org/packages/kime)에 있으며, 만약 소스에서 빌드하시려면 [kime-git](https://aur.archlinux.org/packages/kime-git)에서 설치할 수 있습니다.

### 데비안, 우분투

[releases](https://github.com/Riey/kime/releases) 탭에 있는 .deb 파일을 설치할 수 있습니다.

### 페도라

비공식 패키지가 [Fedora Copr](https://copr.fedorainfracloud.org/coprs/toroidalfox/kime/) 에서 운영되고 있습니다.

```sh
dnf copr enable toroidalfox/kime
dnf install kime # 개발 버전은 `kime-git`
```

### 젠투

```sh
eselect repository add riey git https://github.com/Riey/overlay
eselect repository enable riey
emaint sync -r riey
emerge -av kime
```

### 소스에서 빌드하기

### 도커

도커를 쓰시는 경우엔 따로 의존성을 설치하지 않아도 되어서 편리합니다.

```sh
git clone https://github.com/riey/kime
cd kime

docker build --file build-docker/<배포판 경로>/Dockerfile --tag kime-build:git .
docker run --name kime kime-build:git
docker cp kime:/opt/kime-out/kime.tar.xz .
# deb 파일을 얻으시려면 대신 이 명령어를 실행하세요
# docker cp kime:/opt/kime-out/kime_amd64.deb .
```

### 직접 빌드

빌드하기 전에 **cargo** 및 아래 나열되어 있는 기타 종속성이 설치되어 있는지 확인하세요.

```sh
git clone https://github.com/riey/kime
cd kime

scripts/build.sh -ar
```

이제 모든 파일은 build/out 경로에 있습니다. 만약 수동 설치를 원하시면 쓰시면 됩니다.

`scripts/install.sh <install-prefix>` 스크립트를 쓸 수도 있습니다. 패키징할 때 유용합니다.

`scripts/release-deb.sh <deb-out-path>` 스크립트를 사용하시면 `deb` 파일을 생성합니다.

#### GTK

대부분 배포판들은 이걸 자동으로 해주므로

패키지로 설치하실 경우에는 필요 없을 수도 있습니다.

```sh
# GTK3 설치 시
sudo gtk-query-immodules-3.0 --update-cache
# GTK4 설치 시
sudo gio-querymodules /usr/lib/gtk-4.0/4.0.0/immodules
```

## 개발

### C/C++

`./scripts/generate_properties.sh`을 실행해서 vscode에서 C/C++ 코드의 인텔리센스 기능을 사용할 수 있습니다.

## 설정

### 데비안 계열

언어 설정에서 입력기 `kime`를 선택해주세요.

### 그 외

init 스크립트에 다음을 추가하세요.

```sh
export GTK_IM_MODULE=kime
export QT_IM_MODULE=kime
export XMODIFIERS=@im=kime
```

만약 X를 사용하신다면 .xprofile에 설정하시면 됩니다.

### 추가적인 서버를 실행

kime은 kime 데몬을 위한 kime.desktop 파일을 /etc/xdg/autostart에 설치합니다

혹시 `i3`나 `sway`처럼 `시작 프로그램`을 지원하지 않는다면 해당 WM의 설정파일에서 `kime` 혹은 원하시는 서버 커맨드를 실행해주세요

### KDE Plasma Wayland

시스템 설정 > 하드웨어 > 입력 장치 > 가상 키보드에서 `kime 데몬`을 선택해야 합니다.  
이후에 로그아웃을 하는 것을 권장합니다.

### Weston
`~/.config/weston.ini`에 해당 내용이 있어야 합니다.
```
[input-method]
path=/usr/bin/kime
```

### Configuration

자세한 옵션은 [CONFIGURATION.md](docs/CONFIGURATION.ko.md)를 참고하세요.

## 종속성 목록

### 런타임 종속성

참고로 필요하신 종속성만 있으면 됩니다
예를 들어 qt6를 사용하지 않으신다면 필요하지 않습니다.

* gtk3
* gtk4
* qt5
* qt6
* libdbus (indicator)
* xcb (candidate)
* fontconfig (xim)
* freetype (xim)
* libxkbcommon (wayland)

### 빌드타임 종속성 (바이너리 실행 시엔 필요 없습니다)

#### 필수

* cmake
* cargo
* libclang
* pkg-config

#### 선택적

* gtk3
* gtk4
* qtbase5-private
* qtbase6-private
* libdbus
* xcb
* fontconfig
* freetype
* libxkbcommon
