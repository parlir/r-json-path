cargo +nightly build --target wasm32-unknown-unknown
wasm-bindgen target/wasm32-unknown-unknown/debug/json_path.wasm --nodejs --out-dir ./wasm
