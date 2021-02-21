#!/bin/sh

cbindgen -o kime_engine.h -c ../capi/cbindgen-c.toml ../capi
cbindgen -o kime_engine.hpp  -c ../capi/cbindgen-cpp.toml ../capi

bindgen \
    --disable-name-namespacing \
    --whitelist-var 'kime::.+' \
    --whitelist-type 'kime::.+' \
    --whitelist-function 'kime::.+' \
    --rustified-enum 'kime::InputResult' \
    -o src/ffi.rs \
    ./kime_engine.hpp
