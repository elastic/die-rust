# DetectItEasy-Rust

Native Rust Bindings for Detect-It-Easy

## Description
This module provides Rust bindings for the [Detect-It-Easy](https://github.com/horsicq/Detect-It-Easy) library, developed by [@horsicq](https://github.com/horsicq). Detect-It-Easy is a powerful tool for analyzing and identifying executable files, allowing users to determine file types, formats, and various characteristics of binary files.

## Features
- Access to core functionalities of Detect-It-Easy from Rust.
- Safe and idiomatic Rust interfaces for interacting with the library.
- Comprehensive error handling and type safety.

## Install

The installation can be done using `cargo`.

```console
cargo build --git https://github.com/elastic/die-rust.git
```

The build requires Qt6 libraries. On Linux/macOS they can usually be obtained from the system's package manager.
To use a specific Qt6 version, it is possible to use `aqtinstall` as follow

```console
export QT_BUILD_VERSION=6.2.2
git clone https://github.com/elastic/die-rust.git && cd die-rust
python -m pip install aqtinstall
python -m aqt install-qt -O build windows desktop ${QT_BUILD_VERSION} win64_msvc2019_64 # (windows)
python -m aqt install-qt -O build linux desktop ${QT_BUILD_VERSION} gcc_64 # (linux)
python -m aqt install-qt -O build mac desktop ${QT_BUILD_VERSION} clang_64 # (macos)
```

Then build `die-rust` by passing the paths to the Qt6 libraries with the `QT6_LIB_PATH` environment.
```console
export QT6_LIB_PATH=./libdie++/build/6.2.2/gcc_64/lib # linux
export QT6_LIB_PATH=./libdie++/build/6.2.2/clang_64/lib # macos
export QT6_LIB_PATH=./libdie++/build/6.2.2/win64_msvc2019_64/lib # windows
cargo build
```


## Examples

### `scan_file.rs`

A simple command line tool acting as a wrapper for resp. `die::scan_file()` and `die::scan_file_with_db()` functions, to scan
a file depending on whether a database path was provided, using the `--database-path` flag.

```console
$ cargo run --quiet --example scan_file -- /bin/ls
[2025-01-03T22:58:15Z INFO  scan_file] ELF64
        Unknown: Unknown

$ export DIE_DB_PATH=${HOME}.local/lib/python3.10/site-packages/die/db/db
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
