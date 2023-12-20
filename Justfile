# Execute tests required to pass prior to be eligble to merge a pull request.
acceptance-tests:
    @ cargo +nightly fmt --check

    cargo +stable clippy --no-deps --all-features -- -Dwarnings
    cargo +stable build --all-features
    cargo +stable test --all-features

# Format source code using `cargo fmt`.
format:
    @ cargo +nightly fmt