# Developer Guide

## Setting up Rust

The easiest way to set up Rust toolchain is with [https://rustup.rs/](https://rustup.rs/).
By default, only stable toolchain is installed. Active Rust development
also requires nightly toolchain:

```sh
rustup install nightly
```

## Building and testing

Build is done with `cargo` tool, just like any other Rust project.

```sh
cargo build
# or
cargo build --release
```

Tests can also be done with `cargo test` command, but a much better way is to
use `nextest` tool. To install it, do `cargo install cargo-nextest`. Then run
tests with the following command:

```sh
cargo nextest r
```

`nextest` is much faster than the default test runner.

### Running check tests

There are also check-tests, which are very similar to LLVM Integrated Tests.
An easy and quick way to run those is to invoke `cargo run --bin check-runner`.
However, a more convenient way for day-to-day use is `cargo-make`:

```sh
cargo install cargo-make
cargo make check # run build and check tests
cargo make check-only # only run check whithout re-building TIR
cargo make test # run build, cargo tests and check
```

### Running fuzz tests

We also have fuzzing set up for each user input parser, like a disassembler
or an IR parser. These tests also require an external tool, that can be
installed with a command like `cargo install cargo-fuzz`. The usage is very
simple:

```sh
# List tests
cargo fuzz list
# Run specific test
cargo +nightly fuzz run fuzz_riscv_disassembler -- -max_total_time=60 -max_len=16384
```

### Collecting coverage info


**WARNING!!!** Coverage tool creates a lot of temp files in your working
directory. You better commit all your changes to be able to use git to
clean up.

Install dependencies:

```sh
rustup component add llvm-tools-preview
cargo install grcov
```

Run tests with special flags:

```sh
CARGO_INCREMENTAL=0 RUSTFLAGS='-Cinstrument-coverage' LLVM_PROFILE_FILE='cargo-test-%p-%m.profraw' cargo test
grcov . --binary-path target/debug/ -s . -t coveralls+ --branch --llvm \
    --ignore '../*' --ignore "/*" --ignore 'macros/*' --ignore 'fuzz/*' \
    --ignore '**/tests/**' -o target/coverage/html
```

Open `target/coverage/html/index.html` to see the report.

Also `main` branch reports are available at
https://coveralls.io/github/perf-toolbox/tir.
