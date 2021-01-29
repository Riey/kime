FROM riey/kime-build:0.1.0

WORKDIR /opt/kime

RUN pacman -S --noconfirm llvm libappindicator-gtk3
RUN mkdir -pv /opt/kime-out

COPY ci ./ci
COPY docs ./docs
COPY xtask ./xtask
COPY daemon ./daemon
COPY engine ./engine
COPY xim ./xim
COPY wayland ./wayland
COPY gtk2 ./gtk2
COPY gtk3 ./gtk3
COPY gtk4 ./gtk4
COPY qt5 ./qt5
COPY qt6 ./qt6

COPY Cargo.toml .
COPY Cargo.lock .
COPY CMakeLists.txt .
COPY LICENSE .
COPY .cargo ./.cargo

ENTRYPOINT [ "ci/entrypoint.sh" ]
