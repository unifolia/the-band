#!/usr/bin/env bash
set -euo pipefail

export PATH="/rust/bin:$PATH"

rustup target add wasm32-unknown-unknown

npm run build
