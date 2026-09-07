#!/usr/bin/env bash
# Build the kime-gha-runner docker image used by the rieypc self-hosted
# runner pool (docker-ci label). Stage the kime flake files from KIME_REPO
# into a clean build context (the context IS the staged flake, so the
# Dockerfile's `COPY . /opt/kime-flake/` sees exactly the flake files), then
# build the image with the devshell + fuzz closures baked in.
#
# Invoked by .github/workflows/runner-image.yaml on develop (KIME_REPO set to
# the checkout), or manually:
#     KIME_REPO=/home/riey/repos/kime ./build.sh
set -euo pipefail

KIME_REPO="${KIME_REPO:-$(pwd)}"
if [[ ! -d "$KIME_REPO" || ! -f "$KIME_REPO/flake.nix" ]]; then
    echo "KIME_REPO must point at a kime checkout (contains flake.nix etc.)" >&2
    exit 1
fi

cd "$(dirname "$0")"

# The image bakes an absolute /opt/kime-flake path inside the container, so
# the source tree is copied into the context rather than bind-mounted.
# OCI invents bind-mount paths when the source is missing; always start clean
# so a rerun never bakes a stale flake.
BLD=context
rm -rf "$BLD"
mkdir -p "$BLD"/nix
cp "$KIME_REPO"/flake.nix "$KIME_REPO"/flake.lock \
   "$KIME_REPO"/shell.nix "$KIME_REPO"/default.nix \
   "$KIME_REPO"/VERSION "$BLD"/
cp "$KIME_REPO"/nix/deps.nix "$BLD"/nix/

# Keep the previous good image around for instant rollback.
docker tag kime-gha-runner:latest kime-gha-runner:previous || true
docker pull myoung34/github-runner:ubuntu-noble
# Bake on the gha-ci network so the nix substituter (172.30.0.251:8080,
# restored with the runner pool) is reachable at build time; the default
# bridge cannot see it and the bake would fall back to cache.nixos.org.
docker build --network gha-ci -f Dockerfile -t kime-gha-runner:latest "$BLD"
rm -rf "$BLD"
echo "kime-gha-runner:latest rebuilt from $KIME_REPO"