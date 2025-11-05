cwd := `pwd`

format:
    just --fmt --unstable

    cargo fmt

    find ./ -iname *.proto | xargs clang-format -style=Google -i

lint:
    #!/usr/bin/env bash
    set -euxo pipefail

    cargo fmt --all -- --check

    disallow=(
        warnings
        unsafe_code
        trivial_casts
        trivial_numeric_casts
        missing_docs
        unused_extern_crates
        unused_import_braces
        unused_qualifications
        clippy::clone_on_ref_ptr
        clippy::all
        clippy::correctness
        clippy::suspicious
        clippy::complexity
        clippy::perf
        clippy::style
        clippy::pedantic
        clippy::nursery
        clippy::missing_errors_doc
        clippy::missing_panics_doc
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
        clippy::struct_field_names
    )

    cargo clippy --workspace --no-default-features \
        -- ${disallow[@]/#/-D } ${allow[@]/#/-A }
    cargo clippy --workspace --all-features \
        -- ${disallow[@]/#/-D } ${allow[@]/#/-A }

test:
    cargo test --workspace --all-features -- --nocapture
    cargo test --workspace --doc --all-features -- --nocapture

    cargo test --workspace --no-default-features --features testing -- --nocapture
    cargo test --workspace --doc --no-default-features --features testing -- --nocapture

docs:
    cargo doc --workspace --all-features --no-deps

docs-open:
    cargo doc --workspace --all-features --no-deps --open

publish:
    #!/usr/bin/env bash
    set -euxo pipefail

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
