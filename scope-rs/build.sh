cargo build --target=wasm32-unknown-unknown --release
wasm-bindgen ./target/wasm32-unknown-unknown/release/eth_toolkit.wasm --out-dir wbg_out --target web --no-typescript
cp -r ../scope-js/wbg_out ../scope-js/wbg_out_prev 
cp -r ./wbg_out ../scope-js/
