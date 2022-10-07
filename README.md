# Axum Zero2Prod

## Prerequisites

- Mold linker: `brew install mold`
- Rust nightly: `rustup install nightly`
- Docker to host database
- Bunyan `npm i -g bunyan` or rust `rust-bunyan` crate
- Run `cp configuration.example.yaml configuration.yaml` and update your config

## Start the web server

- Run `cargo run` to start the webserver
- Run `cargo watch -x run` to run in watch mode

## Running tests

- Run `cargo test` to run all tests
- Run `cargo watch -x test` to run test in watch mode
- Run `TEST_LOG=true cargo test | bunyan` to run tests with log enabled

## Running linter and formatter

- Run `cargo clippy` to lint and `cargo fmt` to format code
- Run `cargo deny check` to run security check all dependencies

## LICENSE

MIT
