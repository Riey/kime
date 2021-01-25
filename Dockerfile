FROM riey/kime-build:0.1.0

WORKDIR /opt/kime

RUN mkdir -pv /opt/kime-out

COPY docs ./docs
COPY xtask ./xtask
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

# vendor cargo deps
RUN cargo vendor >> .cargo/config

CMD cargo xtask build XIM WAYLAND GTK2 GTK3 GTK4 QT5 QT6 && 7z a /opt/kime-out/kime.7z ./build/out/* && cargo xtask release-deb /opt/kime-out
