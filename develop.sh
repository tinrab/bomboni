#!/bin/bash

set -euo pipefail
current_path="$(realpath $0)"
current_dir="$(dirname $current_path)"

function format() {
	cargo fmt
}

function lint() {
	cargo fmt --all -- --check
	cargo clippy --workspace --all-targets --all-features
}

function test() {
	cargo test --workspace --all-targets --all-features -- --nocapture
	cargo test --workspace --doc --all-features -- --nocapture
}

function publish() {
	if [[ $2 =~ ^(--actually-do-it)$ ]]; then
		cargo publish -p bomboni_common
		cargo publish -p bomboni_prost
		cargo publish -p bomboni_proto
		cargo publish -p bomboni_derive
		cargo publish -p bomboni_request
		cargo publish -p bomboni
	else
		cargo publish -p bomboni_common --dry-run --allow-dirty
		cargo publish -p bomboni_prost --dry-run --allow-dirty
		cargo publish -p bomboni_proto --dry-run --allow-dirty
		cargo publish -p bomboni_derive --dry-run --allow-dirty
		cargo publish -p bomboni_request --dry-run --allow-dirty
		cargo publish -p bomboni --dry-run --allow-dirty
	fi
}

function help() {
	echo "Usage: $(basename "$0") [OPTIONS]

Commands:
  lint           Run lints
  test           Run all tests
  help           Show help
"
}

if [[ $1 =~ ^(format|lint|test|publish|help)$ ]]; then
	"$@"
else
	help
	exit 1
fi
