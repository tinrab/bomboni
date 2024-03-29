#!/bin/bash

set -eo pipefail
current_path="$(realpath $0)"
current_dir="$(dirname $current_path)"

function format() {
	cargo fmt
	find ./ -iname *.proto | xargs clang-format -style=Google -i
}

function lint() {
	cargo fmt --all -- --check

	# -D unreachable_pub \
	# -D missing_docs \
	# -A clippy::struct_excessive_bools
	# -D clippy::missing_errors_doc \
	# -D clippy::missing_panics_doc
	# -D clippy::nursery
	cargo clippy --workspace --all-targets --all-features -- \
		-D warnings \
		-D unsafe_code \
		-D trivial_casts \
		-D trivial_numeric_casts \
		-D unused_extern_crates \
		-D unused_import_braces \
		-D unused_qualifications \
		-D clippy::all \
		-D clippy::correctness \
		-D clippy::suspicious \
		-D clippy::complexity \
		-D clippy::perf \
		-D clippy::style \
		-D clippy::pedantic \
		-A unused_braces \
		-A clippy::module_name_repetitions \
		-A clippy::cast_possible_truncation \
		-A clippy::cast_possible_wrap \
		-A clippy::must_use_candidate \
		-A clippy::cast_sign_loss \
		-A clippy::too_many_lines \
		-A clippy::needless_pass_by_value \
		-A clippy::struct_excessive_bools \
		-A clippy::missing_errors_doc \
		-A clippy::missing_panics_doc \
		-A clippy::struct_field_names

}

function test() {
	if [[ "$2" =~ ^(--no-default-features)$ ]]; then
		cargo test --workspace --all-targets --no-default-features -- --nocapture
		cargo test --workspace --doc --no-default-features -- --nocapture
	else
		cargo test --workspace --all-targets --all-features -- --nocapture
		cargo test --workspace --doc --all-features -- --nocapture
	fi
}

function publish() {
	if [[ "$2" =~ ^(--actually-do-it)$ ]]; then
		cargo publish -p bomboni_core --allow-dirty
		cargo publish -p bomboni_wasm_core --allow-dirty
		cargo publish -p bomboni_wasm_derive --allow-dirty
		cargo publish -p bomboni_common --allow-dirty
		cargo publish -p bomboni_wasm --allow-dirty
		cargo publish -p bomboni_prost --allow-dirty
		cargo publish -p bomboni_proto --allow-dirty
		cargo publish -p bomboni_request_derive --allow-dirty
		cargo publish -p bomboni_request --allow-dirty
		cargo publish -p bomboni_template --allow-dirty
		cargo publish -p bomboni_fs --allow-dirty
		cargo publish -p bomboni --allow-dirty
	else
		cargo publish -p bomboni_core --allow-dirty --dry-run
		cargo publish -p bomboni_wasm_core --allow-dirty --dry-run
		cargo publish -p bomboni_wasm_derive --allow-dirty --dry-run
		cargo publish -p bomboni_common --allow-dirty --dry-run
		cargo publish -p bomboni_wasm --allow-dirty --dry-run
		cargo publish -p bomboni_prost --allow-dirty --dry-run
		cargo publish -p bomboni_proto --allow-dirty --dry-run
		cargo publish -p bomboni_request_derive --allow-dirty --dry-run
		cargo publish -p bomboni_request --allow-dirty --dry-run
		cargo publish -p bomboni_template --allow-dirty --dry-run
		cargo publish -p bomboni_fs --allow-dirty --dry-run
		cargo publish -p bomboni --allow-dirty --dry-run
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
