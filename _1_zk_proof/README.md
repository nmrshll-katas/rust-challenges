# 1: ZK knowledge proof

## Problem

Please find the attached Python source file. Please implement the same primitive in Rust. 

Please use any suitable from RustCrypto. 

## Solution

The type DlogProof has the same public interface as the python type:
- prove
- verify

## Developer quickstart

Setup using `nix develop` (needs Nix) or `direnv allow` (needs Nix and nix-direnv).

> Alternatively, install dependencies manually: `Rust stable 1.80+ with target "wasm32-unknown-unknown"`, `python3`, `wasm-pack`, `nodejs 18` (nodejs_20 and 22 have issues) and any dependency listed in `./flake.nix`

Then, you can use these commands:
- Run all unit tests: `utest`
- Run unit tests for challenge 1 only: `test1` or `cargo test --package _1_zk_proof -- --nocapture`

