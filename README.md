# 1: ZK knowledge proof

Docs [here](./_1_zk_proof/)


# 2: Simple web application: Wasm + web sockets.

Docs [here](./_2_websocket_wasm/README.md)



# 3: Cloud sync-point.

Docs [here](./_3_sync_endpoint/README.md)




## Developer quickstart

Setup using `nix develop` (needs Nix) or `direnv allow` (needs Nix and nix-direnv).

> Alternatively, install dependencies manually: `Rust stable 1.80+ with target "wasm32-unknown-unknown"`, `python3`, `wasm-pack`, `nodejs 18`, `pnpm`, `geckodriver`, `firefox` and any dependency listed in `./flake.nix`

Then, you can use these commands:
- Run all unit tests: `utest`
- Run unit tests for challenge 1 only: `test1` or `cargo test --package _1_zk_proof -- --nocapture`
- Run unit tests for challenge 2: websockets: `test2` or 

```
    cd _2_websocket_wasm/
    wasm-pack test --firefox --headless --geckodriver ${path_to_geckodriver_on_your_machine} --
```

- Launch UI for challenge 2: websockets: `run2` or 

```
    wasm-pack build _2_websocket_wasm --target web --out-dir .cache/my-wasm-web
    cd _2_websocket_wasm/my_vite_web_app; rm -rf node_modules/my-wasm-web;
    pnpm i; pnpm start
```

- Run unit tests for challenge 3 only: `test3` or `cargo test --package _3_sync_endpoint -- --nocapture`
- Launch the server for challenge 3: `run3` or `cargo run --package _3_sync_endpoint --`



