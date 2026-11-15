#!/usr/bin/env bash
set -euo pipefail
web_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
repo_dir="$(cd -- "$web_dir/.." && pwd)"
build_dir="${ORACLE_BLOCKS_BUILD:-$repo_dir/build}"
catalog="${ORACLE_BLOCKS_CATALOG:-$build_dir/cards-input.jsonl}"
output_file="${1:-$build_dir/index.html}"
mkdir -p "$build_dir"
export RUST_MIN_STACK=67108864
export CARGO_BUILD_JOBS="${CARGO_BUILD_JOBS:-8}"
if [[ ! -f "$catalog" ]]; then
    python3 "$repo_dir/tools/catalog.py" fetch "$catalog"
fi
cargo build --locked --manifest-path "$repo_dir/Cargo.toml" -p oracle-blocks-wasm --features oracle-text --release --target wasm32-unknown-unknown
cargo run --locked --manifest-path "$repo_dir/Cargo.toml" -p oracle-blocks-catalog -- "$catalog" "$build_dir"
target_dir="$(cargo metadata --no-deps --format-version 1 --manifest-path "$repo_dir/Cargo.toml" | python3 -c 'import json,sys;print(json.load(sys.stdin)["target_directory"])')"
wasm-bindgen "$target_dir/wasm32-unknown-unknown/release/oracle_blocks_wasm.wasm" --target no-modules --out-dir "$build_dir/wasm" --out-name validator
python3 "$web_dir/pack.py" "$build_dir" "$output_file"
