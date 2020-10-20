# `dispose` - Linear wrappers for Rust

This is a small crate I made when working with `gfx-hal` to simplify working with linear resources
that must be consumed at the end of their life, rather than borrowed as `Drop` does.  For more
information, see [the docs](https://docs.rs/dispose).
