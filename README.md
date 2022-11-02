# bpf-script-derive
[![Build Status](https://github.com/arcjustin/bpf-script-derive/workflows/build/badge.svg)](https://github.com/arcjustin/bpf-script-derive/actions?query=workflow%3Abuild)
[![crates.io](https://img.shields.io/crates/v/bpf-script-derive.svg)](https://crates.io/crates/bpf-script-derive)
[![mio](https://docs.rs/bpf-script-derive/badge.svg)](https://docs.rs/bpf-script-derive/)
[![Lines of Code](https://tokei.rs/b1/github/arcjustin/bpf-script-derive?category=code)](https://tokei.rs/b1/github/arcjustin/bpf-script-derive?category=code)

Provides a derive macro for `AddToDatabase` to make adding Rust types to a `bpf_script::types::TypeDatabase` easier.

## Usage

For usage examples, see code located in [examples/](examples/) :

  | Examples | Description |
  |----------|-------------|
  |[custom-type](examples/custom-type.rs)| Creates and inserts a custom type into an empty BTF database|

## TODO
- Add proper error types.
