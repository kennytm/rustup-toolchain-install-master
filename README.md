rustup-toolchain-install-master
===============================

[![Travis status](https://travis-ci.com/kennytm/rustup-toolchain-install-master.svg?branch=master)](https://travis-ci.com/kennytm/rustup-toolchain-install-master)

Installs compiler artifacts generated fresh from Rust's CI into `rustup`.

```
USAGE:
    rustup-toolchain-install-master [FLAGS] [OPTIONS] [--] [commits]...

FLAGS:
    -a, --alt           download the alt build instead of normal build
        --dry-run       Only log the URLs, without downloading the artifacts
    -f, --force         Replace an existing toolchain of the same name
    -h, --help          Prints help information
    -k, --keep-going    Continue downloading toolchains even if some of them failed
    -V, --version       Prints version information

OPTIONS:
        --channel <channel>              specify the channel of the commits instead of detecting it automatically
    -c, --component <components>...      additional components to install, besides rustc and rust-std
        --github-token <github-token>    An authorization token to access GitHub APIs
    -i, --host <host>                    the triples of host platform
    -n, --name <name>                    the name to call the toolchain
    -p, --proxy <proxy>                  the HTTP proxy for all download requests
    -s, --server <server>                the server path which stores the compilers [default: https://rust-lang-ci2.s3-us-west-1.amazonaws.com]
    -t, --targets <targets>...           additional target platforms to install rust-std for, besides the host platform

ARGS:
    <commits>...    full commit hashes of the rustc builds, all 40 digits are needed; if omitted, the latest master
                    commit will be installed
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
$ rustup-toolchain-install-master 10a52c25cad963986cace7a22c167363afca0d74
detecting the channel of the `10a52c25cad963986cace7a22c167363afca0d74` toolchain...
downloading <https://rust-lang-ci2.s3-us-west-1.amazonaws.com/rustc-builds/10a52c25cad963986cace7a22c167363afca0d74/rustc-nightly-x86_64-unknown-linux-gnu.tar.xz>...
56.96 MB / 56.96 MB [=======================================] 100.00 % 10.20 MB/s
downloading <https://rust-lang-ci2.s3-us-west-1.amazonaws.com/rustc-builds/10a52c25cad963986cace7a22c167363afca0d74/rust-std-nightly-x86_64-unknown-linux-gnu.tar.xz>...
17.97 MB / 17.97 MB [=======================================] 100.00 % 9.95 MB/s
toolchain `10a52c25cad963986cace7a22c167363afca0d74` is successfully installed!
```

Use it:

```console
$ rustc +10a52c25cad963986cace7a22c167363afca0d74 -vV
rustc 1.40.0-nightly (10a52c25c 2019-10-24)
binary: rustc
commit-hash: 10a52c25cad963986cace7a22c167363afca0d74
commit-date: 2019-10-24
host: x86_64-unknown-linux-gnu
release: 1.40.0-nightly
LLVM version: 9.0
```

Remove it using `rustup`:

```console
$ rustup uninstall 10a52c25cad963986cace7a22c167363afca0d74
info: uninstalling toolchain '10a52c25cad963986cace7a22c167363afca0d74'
info: toolchain '10a52c25cad963986cace7a22c167363afca0d74' uninstalled
```
