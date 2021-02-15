FROM archlinux:base-devel

WORKDIR /opt/kime

# WORKAROUND for glibc 2.33 and old Docker
# See https://github.com/actions/virtual-environments/issues/2658
# Thanks to https://github.com/lxqt/lxqt-panel/pull/1562
RUN patched_glibc=glibc-linux4-2.33-4-x86_64.pkg.tar.zst && \
    curl -LO "https://repo.archlinuxcn.org/x86_64/$patched_glibc" && \
    bsdtar -C / -xvf "$patched_glibc"

RUN mkdir -pv /var/lib/pacman

RUN pacman -Syu --noconfirm
RUN pacman -S --noconfirm --needed rust cmake clang llvm libxcb cairo
RUN pacman -S --noconfirm --needed gtk2 gtk3 gtk4
RUN pacman -S --noconfirm --needed qt5-base qt6-base
RUN pacman -S --noconfirm --needed libappindicator-gtk3
RUN pacman -S --noconfirm --needed git
RUN mkdir -pv /opt/kime-out

COPY ci ./ci
COPY docs ./docs
COPY scripts ./scripts
COPY res ./res
COPY src ./src
COPY .git ./.git

COPY Cargo.toml .
COPY Cargo.lock .
COPY LICENSE .

RUN cargo fetch

ENTRYPOINT [ "ci/build_xz.sh" ]
