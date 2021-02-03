FROM riey/kime-build:latest

WORKDIR /opt/kime

RUN mkdir -pv /opt/kime-out

COPY ci ./ci
COPY docs ./docs
COPY res ./res
COPY src ./src

COPY Cargo.toml .
COPY Cargo.lock .
COPY LICENSE .
COPY .cargo ./.cargo

ENTRYPOINT [ "ci/build_release.sh" ]
