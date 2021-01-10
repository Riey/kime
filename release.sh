cargo build --release

mkdir -pv build
mkdir -pv build/layout

cp target/release/kime-xim build/kime-xim
cp target/release/libkime_gtk3.so build/im-kime.so

cp engine/data/dubeolsik.yaml build/layout
cp engine/data/config.yaml build/

strip -s build/kime-xim
strip -s build/im-kime.so

