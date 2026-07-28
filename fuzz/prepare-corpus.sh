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

# The layouts kime ships are the best possible seeds for the layout parser;
# read them where they live rather than keeping a second copy under seeds/.
if [ "$target" = layout_yaml ]; then
	cp -n ../src/engine/backends/hangul/data/*.yaml "$dest"/ 2>/dev/null || true
fi
