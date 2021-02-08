#!/bin/sh

cbindgen -o kime_engine.h ../capi

bindgen --whitelist-var 'Kime.*' \
    --whitelist-function 'kime_.*' \
    --whitelist-type 'kime_.*' \
    --rustified-enum 'Kime.*' \
    -o src/ffi.rs \
    ./kime_engine.h
