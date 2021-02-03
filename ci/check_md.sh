#!/bin/bash

set -e

FILES=$(find . -name '*.md' -print)

for f in $FILES
do
    markdownlint --config .markdownlint.json $f
    markdown-link-check $f
done
