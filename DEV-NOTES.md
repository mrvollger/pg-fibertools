# notes

- Download vscode locally
- Install cargo/rust
- Install rust-analyzer extension in vscode
- clone the repo
- Open the repo in vscode

# misc notes on cargo

- use `cargo check` to see if the code is compiling
- use `cargo clippy` to see if the code is linting
- use `cargo fmt` to see if the code is formatted
- use `cargo --bin sync-tags run -- --help` to see if the executable can be compiled and run

# once you are running real data

be sure to use `--release` in `cargo run` and/or `cargo build` to get the best performance.
