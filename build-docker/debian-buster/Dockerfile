FROM rust:slim-buster

ENV DEBIAN_FRONTEND=noninteractive

WORKDIR /opt/kime

RUN echo " \n\
deb http://ftp.kr.debian.org/debian/ stable main contrib non-free\n\
deb http://ftp.kr.debian.org/debian/ stable-updates main contrib non-free\n\
deb http://security.debian.org/ stable/updates main\n\
deb http://ftp.debian.org/debian buster-backports main\n\
" > /etc/apt/sources.list
RUN apt-get update
RUN apt-get install -y build-essential git gcc clang libclang-dev cmake extra-cmake-modules pkg-config zstd
RUN apt-get install -y libpango1.0-dev libcairo2-dev libgtk2.0-dev libgtk-3-dev libglib2.0 libxcb1
RUN apt-get install -y qt5-default qtbase5-dev qtbase5-private-dev libqt5gui5
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
