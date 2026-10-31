#!/bin/sh
# Populate corpus/<target> with the target's starting inputs.
#
# cargo-fuzz writes newly discovered inputs into the corpus directory it is
# given, so it must never be pointed at tracked files: seeds are copied in,
# never linked, and existing corpus entries are left alone (-n).
set -eu

cd "$(dirname "$0")"
target=${1:?usage: prepare-corpus.sh <target>}
dest="corpus/$target"
mkdir -p "$dest"

if [ -d "seeds/$target" ]; then
	cp -n "seeds/$target"/* "$dest"/ 2>/dev/null || true
fi

# What kime ships is the best possible seed for the parser that reads it,
# so those files are read where they live rather than kept as a second
# copy under seeds/.
case "$target" in
layout_yaml)
	cp -n ../src/engine/backends/hangul/data/*.yaml "$dest"/ 2>/dev/null || true
	;;
config_yaml)
	cp -n ../res/default_config.yaml "$dest"/ 2>/dev/null || true
	;;
esac
