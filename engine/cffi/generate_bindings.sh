#!/bin/sh

cbindgen ../capi -o kime_engine.h &&

bindgen kime_engine.h -o src/ffi.rs \
    --whitelist-function 'kime_.*' \
    --whitelist-type 'kime_.*' \
    --size_t-is-usize \
    --default-macro-constant-type \
    --default-enum-style rust

