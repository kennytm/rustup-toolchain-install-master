rustup-toolchain-install-master
===============================

[![Travis status](https://travis-ci.com/kennytm/rustup-toolchain-install-master.svg?branch=master)](https://travis-ci.com/kennytm/rustup-toolchain-install-master)

Installs compiler artifacts generated fresh from Rust's CI into `rustup`.

```
USAGE:
    rustup-toolchain-install-master [FLAGS] [OPTIONS] [--] [commits]...

FLAGS:
    -a, --alt                     download the alt build instead of normal build
        --dry-run                 Only log the URLs, without downloading the artifacts
    -f, --force                   Replace an existing toolchain of the same name
    -h, --help                    Prints help information
    -k, --keep-going              Continue downloading toolchains even if some of them failed
        --no-default-components   do not install rustc and rust-std component unless explicitly specified
    -V, --version                 Prints version information

OPTIONS:
        --channel <channel>              specify the channel of the commits instead of detecting it automatically
    -c, --component <components>...      additional components to install, besides rustc and rust-std
        --github-token <github-token>    An authorization token to access GitHub APIs
    -i, --host <host>                    the triples of host platform
    -n, --name <name>                    the name to call the toolchain
    -p, --proxy <proxy>                  the HTTP proxy for all download requests
    -s, --server <server>                the server path which stores the compilers [default: https://ci-artifacts.rust-lang.org]
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
$ rustup-toolchain-install-master 4fb54ed484e2239a3e9eff3be17df00d2a162be3
detecting the channel of the `4fb54ed484e2239a3e9eff3be17df00d2a162be3` toolchain...
downloading <https://ci-artifacts.rust-lang.org/rustc-builds/4fb54ed484e2239a3e9eff3be17df00d2a162be3/rustc-nightly-x86_64-unknown-linux-gnu.tar.xz>...
47.39 MB / 47.39 MB [=======================================] 100.00 % 10.20 MB/s
downloading <https://ci-artifacts.rust-lang.org/rustc-builds/4fb54ed484e2239a3e9eff3be17df00d2a162be3/rust-std-nightly-x86_64-unknown-linux-gnu.tar.xz>...
15.91 MB / 15.91 MB [=======================================] 100.00 % 9.95 MB/s
toolchain `4fb54ed484e2239a3e9eff3be17df00d2a162be3` is successfully installed!
```

Use it:

```console
$ rustc +4fb54ed484e2239a3e9eff3be17df00d2a162be3 -vV
rustc 1.46.0-nightly (4fb54ed48 2020-06-14)
binary: rustc
commit-hash: 4fb54ed484e2239a3e9eff3be17df00d2a162be3
commit-date: 2020-06-14
host: x86_64-unknown-linux-gnu
release: 1.46.0-nightly
LLVM version: 10.0
```

Remove it using `rustup`:

```console
$ rustup uninstall 4fb54ed484e2239a3e9eff3be17df00d2a162be3
info: uninstalling toolchain '4fb54ed484e2239a3e9eff3be17df00d2a162be3'
info: toolchain '4fb54ed484e2239a3e9eff3be17df00d2a162be3' uninstalled
```
