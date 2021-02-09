#!/bin/sh

cbindgen -o kime_engine.hpp ../capi

bindgen \
    --disable-name-namespacing \
    --whitelist-var 'kime::.+' \
    --whitelist-type 'kime::.+' \
    --whitelist-function 'kime::.+' \
    --rustified-enum 'kime::InputResultType' \
    -o src/ffi.rs \
    ./kime_engine.hpp
