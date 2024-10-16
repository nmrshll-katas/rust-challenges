
# 2: Simple web application: Wasm + web sockets.

## Problem

A Rust library that should be compiled to Wasm and export one function (Typescript syntax)

wsPing(endpoint: string, message: string): Promise<string>

This function should establish a web socket connection to the "endpoint" and send the text message, receive a message, and return its content.

Any UI will be sufficient, but a simple test executed by nodejs/deno/bun would be OK. 

Rust + wasm-bindings + anything else you need.

## Solution

Contains:
- a browser UI where you can click to ping the websocket
- unit tests via a headless browser (firefox)


## Developer quickstart

Setup using `nix develop` (needs Nix) or `direnv allow` (needs Nix and nix-direnv).

> Alternatively, install dependencies manually: `Rust stable 1.80+ with target "wasm32-unknown-unknown"`, `python3`, `wasm-pack`, `nodejs 18`, `pnpm`, `geckodriver`, `firefox` and any dependency listed in `./flake.nix`

Then, you can use these commands:

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


## Testing

Unit testing is done via a headless browser (firefox) and geckodriver. Run them with `test2`

For manual testing:
- launch the UI using `run2` or 
```
    wasm-pack build _2_websocket_wasm --target web --out-dir .cache/my-wasm-web
    cd _2_websocket_wasm/my_vite_web_app; rm -rf node_modules/my-wasm-web;
    pnpm i; pnpm start
```
- navigate to `http://localhost:3000`
- click the button `ping websocket`



