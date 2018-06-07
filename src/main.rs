#[macro_use]
extern crate failure;
extern crate home;
extern crate pbr;
extern crate reqwest;
#[macro_use]
extern crate structopt;
extern crate tar;
extern crate tee;
extern crate tempdir;
extern crate xz2;

use std::borrow::Cow;
use std::env::set_current_dir;
use std::fs::{create_dir_all, rename};
use std::io::{stdout, stderr, Write};
use std::iter::once;
use std::path::{Path, PathBuf};
use std::process::exit;
use std::time::Duration;

use failure::Error;
use pbr::{ProgressBar, Units};
use reqwest::header::{Accept, Authorization, ContentLength};
use reqwest::{Client, ClientBuilder, Proxy};
use structopt::StructOpt;
use tar::Archive;
use tee::TeeReader;
use tempdir::TempDir;
use xz2::read::XzDecoder;

#[derive(StructOpt, Debug)]
struct Args {
    #[structopt(
        help = "full commit hashes of the rustc builds, all 40 digits are needed; \
                if omitted, the latest master commit will be installed"
    )]
    commits: Vec<String>,

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

    #[structopt(short = "p", long = "proxy", help = "the HTTP proxy for all download requests")]
    proxy: Option<String>,

    #[structopt(long = "github-token", help = "An authorization token to access GitHub APIs")]
    github_token: Option<String>,

    #[structopt(long = "dry-run", help = "Only log the URLs, without downloading the artifacts")]
    dry_run: bool,
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
            .get::<ContentLength>()
            .map(|h| h.0)
            .unwrap_or(0);

        let err = stderr();
        let mut lock = err.lock();
        let mut bar = ProgressBar::on(lock, length);
        bar.set_units(Units::Bytes);
        bar.set_max_refresh_rate(Some(Duration::from_secs(1)));

        {
            let response = TeeReader::new(response, &mut bar);
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

        bar.finish_print("completed");
    }

    Ok(())
}

fn install_single_toolchain(
    client: Option<&Client>,
    prefix: &str,
    toolchains_path: &Path,
    commit: &str,
    alt: bool,
    rustc_filename: &str,
    rust_std_targets: &[&str],
) -> Result<(), Error> {
    let dest = if alt {
        Cow::Owned(format!("{}-alt", commit))
    } else {
        Cow::Borrowed(commit)
    };
    let toolchain_path = toolchains_path.join(&*dest);
    if toolchain_path.is_dir() {
        eprintln!("toolchain `{}` is already installed", dest);
        return Ok(());
    }

    // download rustc.
    download_tar_xz(
        client,
        &format!("{}/{}/{}.tar.xz", prefix, commit, rustc_filename),
        &path_buf![&rustc_filename, "rustc"],
        Path::new(&*dest),
    )?;

    // download libstd.
    for target in rust_std_targets {
        let rust_std_filename = format!("rust-std-nightly-{}", target);
        download_tar_xz(
            client,
            &format!("{}/{}/{}.tar.xz", prefix, commit, rust_std_filename),
            &path_buf![&rust_std_filename, &format!("rust-std-{}", target), "lib"],
            &path_buf![&dest, "lib"],
        )?;
    }

    // install.
    if client.is_some() {
        rename(&*dest, toolchain_path)?;
        eprintln!("toolchain `{}` is successfully installed!", dest);
    } else {
        eprintln!(
            "toolchain `{}` will be installed to `{}` on real run",
            dest,
            toolchain_path.display()
        );
    }

    Ok(())
}

fn fetch_master_commit(client: &Client, github_token: Option<&str>) -> Result<String, Error> {
    eprint!("fetching master commit hash... ");

    let mut req = client.get("https://api.github.com/repos/rust-lang/rust/commits/master");
    req.header(Accept(vec![
        "application/vnd.github.VERSION.sha".parse().unwrap(),
    ]));
    if let Some(token) = github_token {
        req.header(Authorization(format!("token {}", token)));
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
        client_builder.proxy(Proxy::all(&proxy)?);
    }
    let client = client_builder.build()?;

    let mut toolchains_path = home::rustup_home().expect("$RUSTUP_HOME is undefined?");
    toolchains_path.push("toolchains");
    if !toolchains_path.is_dir() {
        eprintln!(
            "`{}` is not a directory. please reinstall rustup.",
            toolchains_path.display()
        );
        exit(1);
    }

    let host = args.host.as_ref().map(|s| &**s).unwrap_or(env!("HOST"));
    let rustc_filename = format!("rustc-nightly-{}", host);

    let rust_std_targets = args
        .targets
        .iter()
        .map(|s| &**s)
        .chain(once(host))
        .collect::<Vec<_>>();

    let toolchains_dir = TempDir::new("toolchains")?;
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
        if let Err(e) = install_single_toolchain(
            dry_run_client,
            &prefix,
            &toolchains_path,
            &commit,
            args.alt,
            &rustc_filename,
            &rust_std_targets,
        ) {
            eprintln!("skipping {} due to failure:\n{:?}", commit, e);
        }
    }

    Ok(())
}

fn main() {
    run().unwrap();
}
