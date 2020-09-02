FROM rust:latest


RUN set -xe; \
    rustup component add clippy; \
    cargo install cargo-tarpaulin cargo-audit;