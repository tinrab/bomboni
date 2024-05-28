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

	disallow=(
		warnings
		unsafe_code
		trivial_casts
		trivial_numeric_casts
		# unreachable_pub
		# missing_docs
		unused_extern_crates
		unused_import_braces
		unused_qualifications
		clippy::all
		clippy::correctness
		clippy::suspicious
		clippy::complexity
		clippy::perf
		clippy::style
		clippy::pedantic
		# clippy::nursery
		# clippy::missing_errors_doc
		# clippy::missing_panics_doc
	)
	allow=(
		unused_braces
		clippy::module_name_repetitions
		clippy::cast_possible_truncation
		clippy::cast_possible_wrap
		clippy::must_use_candidate
		clippy::cast_sign_loss
		clippy::too_many_lines
		clippy::needless_pass_by_value
		clippy::struct_excessive_bools
		clippy::missing_errors_doc
		clippy::missing_panics_doc
		clippy::struct_field_names
	)

	cargo clippy --workspace --all-targets --all-features \
		-- ${disallow[@]/#/-D } ${allow[@]/#/-A }

	cargo clippy --workspace --target wasm32-unknown-unknown \
		--features wasm,derive,prost,proto,request,serde,chrono \
		-- ${disallow[@]/#/-D } ${allow[@]/#/-A }
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
		cargo publish -p bomboni_wasm --allow-dirty
		cargo publish -p bomboni_common --allow-dirty
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
