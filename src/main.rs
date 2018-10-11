#[macro_use]
extern crate failure;
extern crate home;
extern crate pbr;
extern crate reqwest;
#[macro_use]
extern crate structopt;
extern crate tar;
extern crate tee;
extern crate tempfile;
extern crate xz2;

use std::borrow::Cow;
use std::env::set_current_dir;
use std::fs::{create_dir_all, remove_dir_all, rename};
use std::io::{stderr, stdout, Write};
use std::iter::once;
use std::path::{Path, PathBuf};
use std::process::exit;
use std::time::Duration;
use std::process::Command;

use failure::Error;
use pbr::{ProgressBar, Units};
use reqwest::header::{ACCEPT, AUTHORIZATION, CONTENT_LENGTH};
use reqwest::{Client, ClientBuilder, Proxy};
use structopt::StructOpt;
use tar::Archive;
use tee::TeeReader;
use tempfile::{tempdir, tempdir_in};
use xz2::read::XzDecoder;

#[derive(StructOpt, Debug)]
struct Args {
    #[structopt(
        help = "full commit hashes of the rustc builds, all 40 digits are needed; \
                if omitted, the latest master commit will be installed"
    )]
    commits: Vec<String>,

    #[structopt(short = "n", long = "name", help = "the name to call the toolchain")]
    name: Option<String>,

    #[structopt(
        short = "a", long = "alt", help = "download the alt build instead of normal build"
    )]
    alt: bool,

    #[structopt(
        short = "s",
        long = "server",
        help = "the server path which stores the compilers",
        default_value = "https://s3-us-west-1.amazonaws.com/rust-lang-ci2"
    )]
    server: String,

    #[structopt(short = "i", long = "host", help = "the triples of host platform")]
    host: Option<String>,

    #[structopt(
        short = "t",
        long = "targets",
        help = "additional target platforms to install, besides the host platform"
    )]
    targets: Vec<String>,

    #[structopt(
        short = "c",
        long = "component",
        help = "additional components to install, besides rustc and rust-std"
    )]
    components: Vec<String>,

    #[structopt(short = "p", long = "proxy", help = "the HTTP proxy for all download requests")]
    proxy: Option<String>,

    #[structopt(long = "github-token", help = "An authorization token to access GitHub APIs")]
    github_token: Option<String>,

    #[structopt(long = "dry-run", help = "Only log the URLs, without downloading the artifacts")]
    dry_run: bool,

    #[structopt(long = "force", short = "f", help = "Replace an existing toolchain of the same name")]
    force: bool,
}

macro_rules! path_buf {
    ($($e:expr),*$(,)*) => { [$($e),*].iter().collect::<PathBuf>() }
}

fn download_tar_xz(
    client: Option<&Client>,
    url: &str,
    src: &Path,
    dest: &Path,
) -> Result<(), Error> {
    eprintln!("downloading <{}>...", url);

    if let Some(client) = client {
        let response = client.get(url).send()?.error_for_status()?;

        let length = response
            .headers()
            .get(CONTENT_LENGTH)
            .and_then(|h| h.to_str().ok())
            .and_then(|h| h.parse().ok())
            .unwrap_or(0);

        let err = stderr();
        let mut lock = err.lock();
        let mut progress_bar = ProgressBar::on(lock, length);
        progress_bar.set_units(Units::Bytes);
        progress_bar.set_max_refresh_rate(Some(Duration::from_secs(1)));

        {
            let response = TeeReader::new(response, &mut progress_bar);
            let response = XzDecoder::new(response);
            for entry in Archive::new(response).entries()? {
                let mut entry = entry?;
                let dest_path = match entry.path()?.strip_prefix(src) {
                    Ok(sub_path) => dest.join(sub_path),
                    Err(_) => continue,
                };
                create_dir_all(dest_path.parent().unwrap())?;
                entry.unpack(dest_path)?;
            }
        }

        progress_bar.finish_print("completed");
    }

    Ok(())
}

struct Toolchain<'a> {
    commit: &'a str,
    host_target: &'a str,
    rust_std_targets: &'a [&'a str],
    components: &'a [&'a str],
    dest: Cow<'a, String>,
}

fn install_single_toolchain(
    client: Option<&Client>,
    prefix: &str,
    toolchains_path: &Path,
    toolchain: &Toolchain,
    force: bool
) -> Result<(), Error> {

    let toolchain_path = toolchains_path.join(&*toolchain.dest);
    if toolchain_path.is_dir() {
        if force {
            if client.is_some() {
                remove_dir_all(&toolchain_path)?;
            }
        } else {
            eprintln!("toolchain `{}` is already installed", toolchain.dest);
            return Ok(());
        }
    }

    // download every component except rust-std.
    for component in once(&"rustc").chain(toolchain.components) {
        let component_filename = format!("{}-nightly-{}", component, toolchain.host_target);
        download_tar_xz(
            client,
            &format!(
                "{}/{}/{}.tar.xz",
                prefix, toolchain.commit, &component_filename
            ),
            &path_buf![&component_filename, *component],
            Path::new(&*toolchain.dest),
        )?;
    }

    // download rust-std for every toolchain.
    for target in toolchain.rust_std_targets {
        let rust_std_filename = format!("rust-std-nightly-{}", target);
        download_tar_xz(
            client,
            &format!(
                "{}/{}/{}.tar.xz",
                prefix, toolchain.commit, rust_std_filename
            ),
            &path_buf![&rust_std_filename, &format!("rust-std-{}", target), "lib"],
            &path_buf![&toolchain.dest, "lib"],
        )?;
    }

    // install.
    if client.is_some() {
        rename(&*toolchain.dest, toolchain_path)?;
        eprintln!("toolchain `{}` is successfully installed!", toolchain.dest);
    } else {
        eprintln!(
            "toolchain `{}` will be installed to `{}` on real run",
            toolchain.dest,
            toolchain_path.display()
        );
    }

    Ok(())
}

fn fetch_master_commit(client: &Client, github_token: Option<&str>) -> Result<String, Error> {
    eprintln!("fetching master commit hash... ");
    match fetch_master_commit_via_git() {
        Ok(hash) => return Ok(hash),
        Err(e) => eprint!("unable to fetch master commit via git, fallback to HTTP. Error: {}", e),
    }

    fetch_master_commit_via_http(client, github_token)
}

fn fetch_master_commit_via_git() -> Result<String, Error> {
    let mut output = Command::new("git")
        .args(&["ls-remote", "https://github.com/rust-lang/rust.git", "master"])
        .output()?;
    ensure!(output.status.success(), "git ls-remote exited with error");
    ensure!(output.stdout.get(..40).map_or(false, |h| h.iter().all(|c| c.is_ascii_hexdigit())), "git ls-remote does not return a commit");

    output.stdout.truncate(40);
    Ok(unsafe { String::from_utf8_unchecked(output.stdout) })
}

fn fetch_master_commit_via_http(client: &Client, github_token: Option<&str>) -> Result<String, Error> {
    let mut req = client.get("https://api.github.com/repos/rust-lang/rust/commits/master");
    req = req.header(ACCEPT, "application/vnd.github.VERSION.sha");
    if let Some(token) = github_token {
        req = req.header(AUTHORIZATION, format!("token {}", token));
    }
    let master_commit = req.send()?.error_for_status()?.text()?;
    if master_commit.len() == 40
        && master_commit
            .chars()
            .all(|c| '0' <= c && c <= '9' || 'a' <= c && c <= 'f')
    {
        let out = stdout();
        let mut lock = out.lock();
        lock.write_all(master_commit.as_bytes())?;
        lock.flush()?;
        eprintln!();
        Ok(master_commit)
    } else {
        bail!("unable to parse `{}` as a commit", master_commit)
    }
}

fn run() -> Result<(), Error> {
    let mut args = Args::from_args();

    let mut client_builder = ClientBuilder::new();
    if let Some(proxy) = args.proxy {
        client_builder = client_builder.proxy(Proxy::all(&proxy)?);
    }
    let client = client_builder.build()?;

    let rustup_home = home::rustup_home().expect("$RUSTUP_HOME is undefined?");
    let toolchains_path = rustup_home.join("toolchains");
    if !toolchains_path.is_dir() {
        eprintln!(
            "`{}` is not a directory. please reinstall rustup.",
            toolchains_path.display()
        );
        exit(1);
    }

    if args.commits.len() > 1 && args.name.is_some() {
        eprintln!("name argument can only be provided with a single commit");
        exit(1);
    }

    let host = args.host.as_ref().map(|s| &**s).unwrap_or(env!("HOST"));

    let components = args.components.iter().map(|s| &**s).collect::<Vec<_>>();

    let rust_std_targets = args
        .targets
        .iter()
        .map(|s| &**s)
        .chain(once(host))
        .collect::<Vec<_>>();

    let toolchains_dir = {
        let path = rustup_home.join("tmp");
        if path.is_dir() {
            tempdir_in(path)
        } else {
            tempdir()
        }
    }?;
    set_current_dir(toolchains_dir.path())?;

    let prefix = format!(
        "{}/rustc-builds{}",
        args.server,
        if args.alt { "-alt" } else { "" }
    );

    if args.commits.is_empty() {
        args.commits.push(fetch_master_commit(
            &client,
            args.github_token.as_ref().map(|s| &**s),
        )?);
    }

    let dry_run_client = if args.dry_run { None } else { Some(&client) };
    for commit in args.commits {
        let dest = if let Some(name) = args.name.as_ref() {
            Cow::Borrowed(name)
        } else if args.alt {
            Cow::Owned(format!("{}-alt", commit))
        } else {
            Cow::Borrowed(&commit)
        };
        if let Err(e) = install_single_toolchain(
            dry_run_client,
            &prefix,
            &toolchains_path,
            &Toolchain {
                commit: &commit,
                host_target: &host,
                rust_std_targets: &rust_std_targets,
                components: &components,
                dest,
            },
            args.force
        ) {
            eprintln!("skipping {} due to failure:\n{:?}", commit, e);
        }
    }

    Ok(())
}

fn main() {
    run().unwrap();
}
