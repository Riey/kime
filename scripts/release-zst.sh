#!/usr/bin/env bash

source $(dirname $0)/tool.sh

if [ -z "$1" ]; then
    echo "Usage: <release-zst.sh> <out-path>"
    exit 1
fi

tar -cvf - -C $KIME_OUT . | zstd -T0 -15 -o "${1}/kime.tar.zst"
