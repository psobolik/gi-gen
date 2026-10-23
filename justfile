default:
    just --list

# Run the package's binary in debub mode
run +args='':
    env RUST_BACKTRACE=full cargo run {{ args }}

# Run the package's binary in release mode
run-release +args='': (run '--release' args)

# Compile the package and its dependencies
build +args='':
    cargo build {{ args }}

# Run the package's binary in release mode
build-release +args='': (build '--release' args)

# Execute all unit and integration tests and build examples of the package
test +args='':
    cargo test {{ args }}

# Remove artifacts that cargo has generated
clean:
    cargo clean

# Build the package's docs
build-docs:
    cargo doc

# Build and serve the package's docs
docs: build-docs
    printf "URL: \e[30;107mhttp://localhost:8000/gi_gen\e[0m\n"
    python3 -m http.server 8000 -d ./target/doc

# Checks package using clippy
check +args='':
    cargo clippy

# Formats source files using rustfmt.
format +args='':
    cargo fmt
