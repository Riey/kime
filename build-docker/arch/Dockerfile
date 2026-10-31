FROM archlinux:base-devel

ENV MESON_ARGS="-Dgtk3=enabled -Dgtk4=enabled -Dqt5=enabled -Dqt6=enabled"

WORKDIR /opt/kime

RUN pacman -Syu --noconfirm
RUN pacman -S --noconfirm --needed rustup meson ninja clang llvm libxcb cairo
RUN pacman -S --noconfirm --needed gtk3 gtk4
RUN pacman -S --noconfirm --needed qt5-base qt6-base
RUN pacman -S --noconfirm --needed git
RUN mkdir -pv /opt/kime-out

COPY src ./src
COPY .git ./.git

COPY Cargo.toml .
COPY Cargo.lock .
COPY rust-toolchain.toml .

RUN rustup install

RUN cargo fetch

COPY res ./res
COPY ci ./ci
COPY packaging ./packaging
COPY docs ./docs
COPY meson.build .
COPY meson.options .
COPY LICENSE .
COPY NOTICE.md .
COPY README.ko.md .
COPY README.md .
COPY VERSION .

ENTRYPOINT [ "ci/build_zst.sh" ]
