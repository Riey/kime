FROM archlinux:base-devel

WORKDIR /opt/kime

RUN pacman -Syu --noconfirm
RUN pacman -S --noconfirm --needed rust cmake clang llvm libxcb cairo
RUN pacman -S --noconfirm --needed gtk2 gtk3 gtk4
RUN pacman -S --noconfirm --needed qt5-base qt6-base
RUN pacman -S --noconfirm --needed libappindicator-gtk3
RUN pacman -S --noconfirm --needed git
RUN mkdir -pv /opt/kime-out

COPY src ./src
COPY .git ./.git

COPY Cargo.toml .
COPY Cargo.lock .
COPY LICENSE .

RUN cargo fetch

COPY res ./res
COPY ci ./ci
COPY docs ./docs
COPY scripts ./scripts

ENTRYPOINT [ "ci/build_xz.sh" ]
