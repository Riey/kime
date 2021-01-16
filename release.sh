#!/bin/sh

set -e

cargo build --release

pkg/release-7z.sh
pkg/release-deb.sh
