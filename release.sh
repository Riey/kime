cargo build --release

mkdir -pv build

cp target/release/kime-xim build/kime-xim
cp target/release/libkime_gtk3.so build/im-kime.so

strip -s build/kime-xim
strip -s build/im-kime.so

