#!/usr/bin/env bash

source $(dirname $0)/tool.sh

cargo run -p properties_writer > .vscode/c_cpp_properties.json
