# 3: Cloud sync-point.

## Problem 

Small web service with one endpoint: /wait-for-second-party/:unique-id

This endpoint allows two parties to sync. When one party makes a POST request, the response will be delayed until the second party requests the same URL. In other words, the first party is blocked until the second party arrives or a timeout occurs (let it be 10 seconds).

Rust: tokio + anything else you need.

## Solution

An axum/tokio server with the one endpoint.
Checks if there is already another party waiting with a matching ID, else waits 10 seconds for it.
Returns response as Server-sent event (SSE).

We want to allow concurrent requests, so no locking on a shared resource is allowed.
Instead we use an actor (a tokio task owning state, in this case the state is a mapping of ID -> party waiting on match).
We send messages between the request handler task and the actor task.
We make sure the request handler task can be interrupted (paused) while waiting (by just using .await-ing a match from another party).

## Developer quickstart

Setup using `nix develop` (needs Nix) or `direnv allow` (needs Nix and nix-direnv).

> Alternatively, install dependencies manually: `Rust stable 1.80+ with target "wasm32-unknown-unknown"`, `python3`, `wasm-pack`, `nodejs 18` (nodejs_20 and 22 have issues) and any dependency listed in `./flake.nix`

Then, you can use these commands: 

- Run all unit tests: `utest`
- Run unit tests for challenge 3 only: `test3` or `cargo test --package _3_sync_endpoint -- --nocapture`
- Launch the server for challenge 3: `run3` or `cargo run --package _3_sync_endpoint --`


