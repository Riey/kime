#!/bin/bash

source $(dirname $0)/tool.sh

if [ -z "$1" ]; then
    echo "Usage: <release-zst.sh> <out-path>"
    exit 1
fi

tar --zstd -cvf "${1}/kime.tar.zst" -C ./build/out .
