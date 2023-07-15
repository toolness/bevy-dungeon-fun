set CARGO_TARGET_WASM32_UNKNOWN_UNKNOWN_RUNNER=wasm-server-runner
set RUSTFLAGS=--cfg=web_sys_unstable_apis
cargo run --target wasm32-unknown-unknown
