# DetectItEasy-Rust

Native Rust Bindings for Detect-It-Easy

## Description
This module provides Rust bindings for the [Detect-It-Easy](https://github.com/horsicq/Detect-It-Easy) library, developed by [@horsicq](https://github.com/horsicq). Detect-It-Easy is a powerful tool for analyzing and identifying executable files, allowing users to determine file types, formats, and various characteristics of binary files.

## Features
- Access to core functionalities of Detect-It-Easy from Rust.
- Safe and idiomatic Rust interfaces for interacting with the library.
- Comprehensive error handling and type safety.

### Build

> [!Warning]
> `Detect-It-Easy` has a hard requirement for Qt6 libraries. `cargo build` will manage the entire building process in an automated, but to be able to link against Qt6, it will use the library [`aqtinstall`](). Therefore `python` must be installed on the system. Note that downloading the Qt libraries may take some time, depending on your Internet connection.

The installation can be done using `cargo`.

```console
git clone https://github.com/elastic/die-rust.git
cd die-rust
cargo build
```

Set `QT6_LIB_PATH` to link against an existing Qt6 installation instead of the one `aqtinstall`
downloads under `libdie++/build`. A relative value is resolved against the crate root.

### Using `die` as a dependency

Qt6 and `libdie` are not on the default loader path, and cargo does not propagate a dependency's
`rustc-link-arg` to the crates that depend on it. A crate depending on `die` therefore links fine
out of the box, but its binaries will not *run* until it embeds an rpath itself. This crate
declares `links = "die"` and exports the two directories needed for that, so a dependent's
`build.rs` can do:

```rust,no_run
fn main(){
    // [...]
    let qt = std::env::var("DEP_DIE_QT_LIB_PATH").unwrap();
    let die = std::env::var("DEP_DIE_INSTALL_LIB_PATH").unwrap();

    // [...]
    println!("cargo:rustc-link-arg=-Wl,-rpath,{qt}");
    println!("cargo:rustc-link-arg=-Wl,-rpath,{die}");
}
```

Setting `LD_LIBRARY_PATH` (`DYLD_LIBRARY_PATH` on macOS) to those directories at run time works
just as well.


## Examples

### `scan_file.rs`

A simple command line tool acting as a wrapper for resp. `die::scan_file()` and `die::scan_file_with_db()` functions, to scan
a file depending on whether a database path was provided, using the `--database-path` flag.

```console
$ cargo run --quiet --example scan_file -- /bin/ls
[2025-01-03T22:58:15Z INFO  scan_file] ELF64
        Unknown: Unknown

$ git clone --quiet https://github.com/horsicq/Detect-It-Easy
$ export DIE_DB_PATH=`pwd`/Detect-It-Easy/db
$ cargo run --quiet --example scan_file -- /bin/ls --database-path ${DIE_DB_PATH}
[2025-01-06T18:58:30Z INFO  scan_file] ELF64
        Library: GLIBC(2.4)[DYN AMD64-64]
```


## Tests

Specify directory containing the DiE signature through the ` DIE_DB_PATH` environment variable as such

```console
export DIE_DB_PATH=${HOME}/.local/lib/python3.10/site-packages/die/db/db
```

Then run `cargo test`


## License

This project is licensed under the Apache License v2. See the LICENSE file for more details.

## Contributing

Contributions are welcome! Please follow the guidelines provided in the repository for contributing to this project.
