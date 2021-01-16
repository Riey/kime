cargo build --release

mkdir -pv build
mkdir -pv build/layouts

cp target/release/kime-xim build/kime-xim
cp target/release/libkime_gtk3.so build/im-kime.so
cp target/release/libkime_engine.so build/libkime_engine.so

cp engine/cffi/kime_engine.h build

# cp engine/data/config.yaml build/

strip -s build/kime-xim
strip -s build/libkime_engine.so
strip -s build/im-kime.so
