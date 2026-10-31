# Runner image for the rieypc self-hosted GitHub Actions runners.
#
# Built on myoung34's Ubuntu 24.04 ("noble") runner image, which installs the
# runner itself (its tag pins the runner version — see its install_actions
# layer) plus GitHub's node toolchain. This image additionally bakes the kime
# devshell + fuzz closures so docker-ci jobs never evaluate the repo flake.
#
# The myoung34 entrypoint handles registration/removal per container; the
# vanilla RUNNER_TOKEN env is what keeps this simple (see the docker pool's
# run-slot.sh). Ephemeral containers run with DISABLE_AUTO_UPDATE=true, so the
# runner version this image carries is the fleet's.
#
# Build via ci/actions-runner-docker/build.sh (runner-image.yaml on develop,
# or locally with KIME_REPO=<tree>): it stages the kime flake files into a
# clean context dir and `docker build`s THAT as the context, so this file has
# no `staging/` prefix — the staged files are the build context root.

ARG RUNNER_BASE=myoung34/github-runner:ubuntu-noble
FROM ${RUNNER_BASE}

ENV RUNNER_SCOPE=repo \
    ORG_NAME=Riey \
    REPO_NAME=kime \
    RUNNER_LABELS='docker-ci' \
    RUNNER_GROUP=default

# The LAN cache server serves the cache API only; the patched worker must not
# override ACTIONS_RESULTS_URL with the real service URL, or artifact upload
# from our cache server becomes a 404 (see the "Artifact endpoint" steps in
# ci.yaml/fuzz.yaml). The runner binary stores the env name as a UTF-16LE
# string, so this swaps `ACTIONS_RESULTS_URL` -> `ACTIONS_RESULTS_ORL`
# byte-for-byte (same length, `\0` fill) — length-preserving, offsets intact.
RUN sed -i 's/\x41\x00\x43\x00\x54\x00\x49\x00\x4F\x00\x4E\x00\x53\x00\x5F\x00\x52\x00\x45\x00\x53\x00\x55\x00\x4C\x00\x54\x00\x53\x00\x5F\x00\x55\x00\x52\x00\x4C\x00/\x41\x00\x43\x00\x54\x00\x49\x00\x4F\x00\x4E\x00\x53\x00\x5F\x00\x52\x00\x45\x00\x53\x00\x55\x00\x4C\x00\x54\x00\x53\x00\x5F\x00\x4F\x00\x52\x00\x4C\x00/g' /actions-runner/bin/Runner.Worker.dll

# Tooling the deterministic jobs need that isn't in the runner base.
ARG CARGO_DENY_VERSION=0.20.2
ARG TYPOS_VERSION=v1.48.0
ENV RUSTUP_HOME=/usr/local/rustup \
    CARGO_HOME=/usr/local/cargo \
    PATH=/usr/local/cargo/bin:/nix/var/nix/profiles/default/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin
RUN curl --proto '=https' --tlsv1.2 -fsSL https://sh.rustup.rs \
        | sh -s -- -y --default-toolchain stable --profile minimal --component rustfmt \
    && curl -fsSL "https://github.com/EmbarkStudios/cargo-deny/releases/download/${CARGO_DENY_VERSION}/cargo-deny-${CARGO_DENY_VERSION}-x86_64-unknown-linux-musl.tar.gz" \
        | tar xz -C /usr/local/bin --strip-components=1 "cargo-deny-${CARGO_DENY_VERSION}-x86_64-unknown-linux-musl/cargo-deny" \
    && curl -fsSL "https://github.com/crate-ci/typos/releases/download/${TYPOS_VERSION}/typos-${TYPOS_VERSION}-x86_64-unknown-linux-musl.tar.gz" \
        | tar xz -C /usr/local/bin ./typos

# Deterministic Nix installer with a first-class local substituter (no init,
# so nix apps run inside the container without systemd).
RUN curl --proto '=https' --tlsv1.2 -fsSL https://install.determinate.systems/nix \
        | sh -s -- install linux --init none --no-confirm \
            --extra-conf "substituters = http://172.30.0.251:8080 https://cache.nixos.org" \
            --extra-conf "trusted-public-keys = cache.nixos.org-1:6NCHdD59X431o0gWypbMrAURkbJ16ZPMQFGspcDShjY="

# The full devshell + fuzz closure baked in at build time (build.sh stages
# KIME_REPO into the build context). Building these profiles here means a
# docker-ci job only ever runs `nix develop $profile`.
COPY . /opt/kime-flake/
RUN nix develop --no-write-lock-file /opt/kime-flake --profile /opt/kime-devshell --command rustc --version \
    && nix develop --no-write-lock-file /opt/kime-devshell --command true
RUN nix develop --no-write-lock-file /opt/kime-flake#fuzz --profile /opt/kime-fuzz-shell --command cargo fuzz --version \
    && nix develop --no-write-lock-file /opt/kime-fuzz-shell --command true