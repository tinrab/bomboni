cwd := `pwd`
root-features := "derive,prost,proto,request,template,serde,chrono,tokio,tonic,fs,postgres,mysql"
workspace-excludes := "bookstore-api,bookstore-service"
exclude-flags := "--exclude " + replace(workspace-excludes, ",", " --exclude ")

format:
    just --fmt --unstable

    cargo fmt

    find ./ -iname *.proto | xargs clang-format -style=Google -i

check:
    #!/usr/bin/env bash
    set -euxo pipefail

    cargo check --workspace --no-default-features {{ exclude-flags }}
    cargo check --workspace --features "{{ root-features }}" {{ exclude-flags }}

    cargo check --workspace --features wasm --exclude bomboni_fs {{ exclude-flags }}
    cargo check --target wasm32-unknown-unknown -p bomboni_wasm_core
    cargo check --target wasm32-unknown-unknown -p bomboni_wasm --features derive,js
    cargo check --target wasm32-unknown-unknown -p bomboni_wasm_derive
    cargo check --target wasm32-unknown-unknown -p bomboni_common --features wasm,js

    examples=(
        grpc/bookstore/bookstore-api
        grpc/bookstore/bookstore-service
    )
    for example in "${examples[@]}"; do
        cd "{{ cwd }}/examples/${example}"
        if [[ "${example}" == *"api"* ]]; then
            cargo check --features server
            cargo check --features client
        else
            cargo check
        fi
    done

    integrations=(
        request-individual-crates
        request-root-crate
    )
    for integration in "${integrations[@]}"; do
        cd "{{ cwd }}/integrations/${integration}"
        cargo check
    done

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
        unused_imports
        missing_docs
        clippy::module_name_repetitions
        clippy::cast_possible_truncation
        clippy::cast_possible_wrap
        clippy::must_use_candidate
        clippy::cast_sign_loss
        clippy::too_many_lines
        clippy::needless_pass_by_value
        clippy::struct_excessive_bools
        clippy::struct_field_names
        clippy::doc_markdown
    )

    cargo clippy --workspace --no-default-features \
        -- ${disallow[@]/#/-D } ${allow[@]/#/-A }
    cargo clippy --workspace --all-features \
        -- ${disallow[@]/#/-D } ${allow[@]/#/-A }

test:
    # TODO: Figure out the best way to test feature matrix

    cargo test --workspace --features "{{ root-features }},testing" {{ exclude-flags }} -- --nocapture
    cargo test --workspace --doc --features "{{ root-features }},testing" {{ exclude-flags }} -- --nocapture

    cargo test --workspace --no-default-features --features testing {{ exclude-flags }} -- --nocapture
    cargo test --workspace --doc --no-default-features --features testing {{ exclude-flags }} -- --nocapture

    cd "{{ cwd }}/integrations/request-individual-crates"
    cargo test
    cd "{{ cwd }}/integrations/request-root-crate"
    cargo test

docs:
    cargo doc --workspace --all-features --no-deps

docs-open:
    cargo doc --workspace --all-features --no-deps --open

clean:
    cargo clean

actually := env("ACTUALLY_DO_IT", "0")

publish:
    #!/usr/bin/env bash
    set -euxo pipefail

    if [[ "{{ actually }}" != "1" ]]; then
        echo "This is a dry run."
    fi

    packages=(
        bomboni_core
        bomboni_wasm_core
        bomboni_wasm_derive
        bomboni_wasm
        bomboni_common
        bomboni_fs
        bomboni_macros
        bomboni_prost
        bomboni_proto
        bomboni_template
        bomboni_request_derive
        bomboni_request
    )

    max_attempts=5
    attempt=0
    remaining="$packages"

    while [[ -n "$remaining" ]] && [[ $attempt -lt $max_attempts ]]; do
        attempt=$((attempt + 1))
        still_remaining=""

        for package in $remaining; do
            if [[ "{{ actually }}" == "1" ]]; then
                if cargo publish -p "$package" 2>&1; then
                    echo "Published $package"
                else
                    still_remaining="$still_remaining $package"
                fi
            else
                if cargo publish -p "$package" --dry-run --allow-dirty 2>&1; then
                    echo "Dry-run OK: $package"
                else
                    still_remaining="$still_remaining $package"
                fi
            fi
        done

        if [[ "$still_remaining" == "$remaining" ]]; then
            echo "Failed to publish: $still_remaining"
            exit 1
        fi

        remaining="$still_remaining"
    done
