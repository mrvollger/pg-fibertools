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

be sure to use `--release` in `cargo run` and/or `cargo build` to get the best performance. e.g.

```bash
cargo run --bin sync-tags --release -- --help
```

(also don't forget about query sorting the input bam files)

# documentation on rust-htslib (reading and writing bam files)

https://docs.rs/rust-htslib/latest/rust_htslib/

git clone .. && git checkout -b anna
