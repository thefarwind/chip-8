FROM rust:latest


RUN set -xe; \
    rustup component add clippy; \
    CARGO_TARGET_DIR=/cache cargo install cargo-tarpaulin cargo-audit; \
    rm -rf /cache;