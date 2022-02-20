#!/bin/bash
TARGET="${CARGO_TARGET_DIR:-target}"
set -e
cd "`dirname $0`"

cargo build --all --target wasm32-unknown-unknown --release
cp $TARGET/wasm32-unknown-unknown/release/nft_factory.wasm ./res/
cp $TARGET/wasm32-unknown-unknown/release/nft_loot_box.wasm ./res/
cp $TARGET/wasm32-unknown-unknown/release/nft_hero.wasm ./res/