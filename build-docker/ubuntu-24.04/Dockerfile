FROM ubuntu:24.04

ENV DEBIAN_FRONTEND=noninteractive \
  PATH=/root/.cargo/bin:$PATH
ENV MESON_ARGS="-Dgtk3=enabled -Dgtk4=enabled -Dqt5=enabled -Dqt6=enabled"

WORKDIR /opt/kime

RUN apt-get update
RUN apt-get install -y curl apt-utils \
                       build-essential git gcc libclang-dev ninja-build pkg-config zstd \
                       libpango1.0-dev libcairo2-dev libgtk-4-dev libgtk-3-dev libglib2.0 libxcb1 \
                       qtbase5-dev qtbase5-private-dev libqt5gui5 \
					   qt6-base-dev qt6-base-private-dev \
                       libxcb-shape0-dev libxcb-xfixes0-dev \
                       meson
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable --no-modify-path --profile minimal
RUN rustc --version
RUN mkdir -pv /opt/kime-out

COPY src /opt/kime/src

COPY Cargo.toml /opt/kime
COPY Cargo.lock /opt/kime

RUN cargo fetch

COPY res /opt/kime/res
COPY ci /opt/kime/ci
COPY packaging /opt/kime/packaging
COPY docs /opt/kime/docs
COPY meson.build /opt/kime
COPY meson.options /opt/kime
COPY LICENSE /opt/kime
COPY NOTICE.md /opt/kime
COPY README.ko.md /opt/kime
COPY README.md /opt/kime
COPY VERSION /opt/kime

ENTRYPOINT [ "ci/build_deb.sh" ]
