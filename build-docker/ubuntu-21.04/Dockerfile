FROM ubuntu:21.04

ENV DEBIAN_FRONTEND=noninteractive \
    PATH=/root/.cargo/bin:$PATH

WORKDIR /opt/kime

RUN apt-get update
RUN apt-get install -y curl apt-utils
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable --no-modify-path --profile minimal
RUN rustc --version
RUN apt-get install -y build-essential git gcc libclang-11-dev cmake extra-cmake-modules pkg-config zstd
RUN apt-get install -y libpango1.0-dev libcairo2-dev libgtk2.0-dev libgtk-3-dev libglib2.0 libxcb1
RUN apt-get install -y qtbase5-dev qtbase5-private-dev libqt5gui5 libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev
RUN mkdir -pv /opt/kime-out

COPY src ./src

COPY Cargo.toml .
COPY Cargo.lock .

RUN cargo fetch

COPY res ./res
COPY ci ./ci
COPY docs ./docs
COPY scripts ./scripts
COPY LICENSE .
COPY NOTICE.md .
COPY README.ko.md .
COPY README.md .
COPY VERSION .

ENTRYPOINT [ "ci/build_deb.sh" ]
