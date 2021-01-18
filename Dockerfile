FROM rust:slim

RUN apt-get update -y && apt-get install -y p7zip pkg-config libpango1.0-dev libcairo2-dev libgtk-3-dev libglib2.0 libxcb1

WORKDIR /opt/kime

COPY engine ./engine
COPY xim ./xim
COPY gtk3 ./gtk3

COPY Cargo.toml .
COPY Cargo.lock .

RUN mkdir .cargo
RUN cargo vendor > .cargo/config
RUN cargo install -f cargo-deb

COPY LICENSE .

COPY pkg ./pkg

CMD ./pkg/release-deb.sh
