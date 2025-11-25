rustup-toolchain-install-master
===============================

[![Travis status](https://travis-ci.com/kennytm/rustup-toolchain-install-master.svg?branch=master)](https://travis-ci.com/kennytm/rustup-toolchain-install-master)

Installs compiler artifacts generated fresh from Rust's CI into `rustup`.

```
Usage: rustup-toolchain-install-master [OPTIONS] [COMMITS]...

Arguments:
  [COMMITS]...  full commit hashes of the rustc builds, all 40 digits are needed; if omitted, the latest HEAD commit will be installed

Options:
  -n, --name <NAME>                  the name to call the toolchain
  -a, --alt                          download the alt build instead of normal build
  -s, --server <SERVER>              the server path which stores the compilers [default: https://ci-artifacts.rust-lang.org]
  -i, --host <HOST>                  the triple of the host platform
  -t, --targets <TARGETS>            additional target platforms to install rust-std for, besides the host platform
  -c, --component <COMPONENTS>       additional components to install, besides rustc and rust-std
      --channel <CHANNEL>            specify the channel of the commits instead of detecting it automatically
  -p, --proxy <PROXY>                the HTTP proxy for all download requests
      --github-token <GITHUB_TOKEN>  An authorization token to access GitHub APIs
      --dry-run                      Only log the URLs, without downloading the artifacts
  -f, --force                        Replace an existing toolchain of the same name
  -k, --keep-going                   Continue downloading toolchains even if some of them failed
  -h, --help                         Print help
  -V, --version                      Print version
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
