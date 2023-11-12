#!/bin/bash

set -euo pipefail
current_path="$(realpath $0)"
current_dir="$(dirname $current_path)"

function format() {
	cargo fmt
}

function lint() {
	cargo fmt --all -- --check
	cargo clippy --workspace --all-targets --all-features -- \
		-D clippy::all \
		-D warnings \
		-D unsafe_code \
		-D trivial_casts \
		-D trivial_numeric_casts \
		-D unused_extern_crates \
		-D unused_import_braces \
		-D unused_qualifications \
		-D missing_docs
}

function test() {
	cargo test --all-targets --all-features -- --nocapture
	cargo test --doc --all-features -- --nocapture
}

function help() {
	echo "Usage: $(basename "$0") [OPTIONS]

Commands:
  lint           Run lints
  test           Run all tests
  help           Show help
"
}

if [[ $1 =~ ^(format|lint|test|help)$ ]]; then
	"$@"
else
	help
	exit 1
fi
