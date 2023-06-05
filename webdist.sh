cargo build --release --target wasm32-unknown-unknown
wasm-bindgen --out-dir out --target web ./target/wasm32-unknown-unknown/release/bevy-fun2.wasm
rm -rf dist
mkdir dist
cp -r out dist/
cp index.html dist/
cp -r assets dist/
