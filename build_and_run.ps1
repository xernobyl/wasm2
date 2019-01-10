cargo build --target wasm32-unknown-unknown
wasm-bindgen .\target\wasm32-unknown-unknown\debug\wasm2.wasm --out-dir .
npm run serve
