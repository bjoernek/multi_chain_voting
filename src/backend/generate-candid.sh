#!/usr/bin/env bash
set -e

SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )

candid-extractor "$SCRIPT_DIR/../../target/wasm32-unknown-unknown/release/backend.wasm" > "$SCRIPT_DIR/backend.did"
echo OK