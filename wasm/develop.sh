#!/bin/bash

set -euo pipefail
current_path="$(realpath $0)"
current_dir="$(dirname $current_path)"

function pack() {
	rm -rf "$current_dir/pkg"
	RUSTFLAGS="-C opt-level=s" \
		wasm-pack build --mode no-install \
		--out-name "wasm" \
		--target nodejs \
		--release
}

function help() {
	echo "Usage: $(basename "$0") [OPTIONS]

Commands:
  pack       Run wasm-pack
  help       Show help
"
}

if [[ $1 =~ ^(pack|help)$ ]]; then
	"$@"
else
	help
	exit 1
fi
