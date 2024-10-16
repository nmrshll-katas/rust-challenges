{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-24.05";
    utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
    my-utils = {
      url = "github:nmrshll/nix-utils";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.utils.follows = "utils";
    };
  };

  outputs = { self, nixpkgs, utils, rust-overlay, my-utils }:
    utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ (import rust-overlay) ];
        };
        customRust = pkgs.rust-bin.stable."1.80.0".default.override {
          extensions = [ "rust-src" "rust-analyzer" ];
          targets = [ "wasm32-unknown-unknown" ];
        };
        binaries = my-utils.binaries.${system} // {
          geckodriver = "${pkgs.geckodriver}/bin/geckodriver";
        };

        baseInputs = with pkgs; [
          customRust
          python3
          wasm-pack
          nodejs_18
          # bun
          # deno
          pnpm
        ] ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [
          pkgs.darwin.apple_sdk.frameworks.Security
          pkgs.darwin.apple_sdk.frameworks.SystemConfiguration
          pkgs.darwin.apple_sdk.frameworks.CoreServices
          pkgs.darwin.apple_sdk.frameworks.CoreFoundation
          pkgs.darwin.apple_sdk.frameworks.Foundation
          pkgs.libiconv
        ];

        devInputs = with pkgs; [
          nixpkgs-fmt
          cargo-nextest
          geckodriver
        ];

        env = {
          RUST_BACKTRACE = "1";
        };

        scripts = with pkgs; [
          (writeScriptBin "utest" ''cargo nextest run --workspace --nocapture -- $SINGLE_TEST '')

          (writeScriptBin "test1" ''cargo nextest run --package _1_zk_proof --nocapture -- $SINGLE_TEST '')

          (writeScriptBin "test2" ''set -euxo pipefail
            cd _2_websocket_wasm/
            wasm-pack test --firefox --headless --geckodriver ${binaries.geckodriver} --
          '')
          (writeScriptBin "build2-deno" ''
            wasm-pack build _2_websocket_wasm --target deno --out-dir .cache/my-wasm-deno
          '')
          (writeScriptBin "run2" ''
            wasm-pack build _2_websocket_wasm --target web --out-dir .cache/my-wasm-web
            cd _2_websocket_wasm/my_vite_web_app; rm -rf node_modules/my-wasm-web;
            pnpm i; pnpm start
          '')

          (writeScriptBin "test3" ''cargo nextest run --package _3_sync_endpoint --nocapture -- $SINGLE_TEST '')
          (writeScriptBin "run3" ''cargo run --package _3_sync_endpoint -- '')
        ];

      in
      {
        devShells.default = with pkgs; mkShell {
          inherit env;
          buildInputs = baseInputs ++ devInputs ++ scripts;
          shellHook = "
              ${my-utils.binaries.${system}.configure-vscode};
              dotenv
            ";
        };
      }
    );
}




