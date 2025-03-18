# ConvCommits

ConvCommits is a library to parse [Conventional Commits][conv_commits].

* [Latest API Documentation][api_docs]

## Purpose

The purpose of the library is to provide a way to read conventional commits.

## Installation

```sh
cargo add convcommits
```

By default, the `std` feature is enabled.

### Alloc only

If the host environment has an allocator but does not have access to the Rust
`std` library:

```sh
cargo add --no-default-features --features alloc convcommits
```

### No allocator / core only

If the host environment does not have an allocator:

```sh
cargo add --no-default-features convcommits
```

## License

Licensed under either of [Apache License, Version 2.0][LICENSE_APACHE] or [MIT
License][LICENSE_MIT] at your option.

### Contributions

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.

[LICENSE_APACHE]: LICENSE-APACHE
[LICENSE_MIT]: LICENSE-MIT
[api_docs]: https://docs.rs/convcommits/
[conv_commits]: https://www.conventionalcommits.org/
