rustup-toolchain-install-master
===============================

[![Travis status](https://travis-ci.com/kennytm/rustup-toolchain-install-master.svg?branch=master)](https://travis-ci.com/kennytm/rustup-toolchain-install-master)

Installs compiler artifacts generated fresh from Rust's CI into `rustup`.

```
USAGE:
    rustup-toolchain-install-master [FLAGS] [OPTIONS] <commits>...

FLAGS:
    -a, --alt        download the alt build instead of normal build
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -c, --component <components>... additional components to install, besides rustc and rust-std
    -i, --host <host>               the triples of host platform
    -p, --proxy <proxy>             the HTTP proxy for all download requests
    -s, --server <server>           the server path which stores the compilers [default: https://s3-us-west
                                    -1.amazonaws.com/rust-lang-ci2]
    -t, --targets <targets>...      additional target platforms to install, besides the host platform

ARGS:
    <commits>...    full commit hashes of the rustc builds; all 40 digits are needed
```

Installation
------------

Install `rustup`, and then install from Cargo.

```console
$ cargo install rustup-toolchain-install-master
```

Usage
-----

Download a normal toolchain:

```console
$ rustup-toolchain-install-master def3269a71be2e737cad27418a3dad9f5bd6cd32
downloading <https://s3-us-west-1.amazonaws.com/rust-lang-ci2/rustc-builds/def3269a71be2e737cad27418a3dad9f5bd6cd32/rustc-nightly-x86_64-apple-darwin.tar.xz>...
completed
downloading <https://s3-us-west-1.amazonaws.com/rust-lang-ci2/rustc-builds/def3269a71be2e737cad27418a3dad9f5bd6cd32/rust-std-nightly-x86_64-apple-darwin.tar.xz>...
completed
toolchain `def3269a71be2e737cad27418a3dad9f5bd6cd32` is successfully installed!
```

Use it:

```console
$ rustc +def3269a71be2e737cad27418a3dad9f5bd6cd32 -vV
rustc 1.25.0-nightly (def3269a7 2018-01-30)
binary: rustc
commit-hash: def3269a71be2e737cad27418a3dad9f5bd6cd32
commit-date: 2018-01-30
host: x86_64-apple-darwin
release: 1.25.0-nightly
LLVM version: 4.0
```

Remove it using `rustup`:

```console
$ rustup uninstall def3269a71be2e737cad27418a3dad9f5bd6cd32
info: uninstalling toolchain 'def3269a71be2e737cad27418a3dad9f5bd6cd32'
info: toolchain 'def3269a71be2e737cad27418a3dad9f5bd6cd32' uninstalled
```
