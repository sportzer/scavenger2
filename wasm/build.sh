#!/bin/sh
cd -- "$(dirname -- "$0")" &&\
    cargo build --release --package scavenger-wasm --target wasm32-unknown-unknown &&\
    wasm-bindgen --no-typescript --no-modules ../target/wasm32-unknown-unknown/release/scavenger_wasm.wasm --out-dir . &&\
    zip scavenger.zip *.wasm *.js *.html rot.js/*
