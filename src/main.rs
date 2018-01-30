#[macro_use]
extern crate failure;
extern crate pbr;
extern crate reqwest;
extern crate structopt;
#[macro_use]
extern crate structopt_derive;
extern crate tar;
extern crate tee;
extern crate tempdir;
extern crate xz2;

use std::env::{home_dir, set_current_dir, var_os};
use std::fs::{create_dir, create_dir_all, read_dir, remove_dir_all, rename};
use std::iter::once;
use std::path::{Path, PathBuf};
use std::process::exit;
use std::str::FromStr;

use failure::{Error, ResultExt};
use pbr::{ProgressBar, Units};
use reqwest::{Client, ClientBuilder, Proxy};
use reqwest::header::ContentLength;
use structopt::StructOpt;
use tar::Archive;
use tee::TeeReader;
use tempdir::TempDir;
use xz2::read::XzDecoder;

#[derive(Copy, Clone, Debug)]
enum Build {
    /// Normal master builds
    Master,
    /// Try build
    Try,
    /// Alt build
    Alt,
}

#[derive(Debug, Fail)]
#[fail(display = "unknown build type")]
struct UnknownBuildError;

impl FromStr for Build {
    type Err = UnknownBuildError;
    fn from_str(input: &str) -> Result<Self, Self::Err> {
        match input {
            "master" => Ok(Build::Master),
            "try" => Ok(Build::Try),
            "alt" => Ok(Build::Alt),
            _ => Err(UnknownBuildError),
        }
    }
}

impl Build {
    fn tag(self) -> &'static str {
        match self {
            Build::Master => "",
            Build::Try => "-try",
            Build::Alt => "-alt",
        }
    }
}

#[derive(StructOpt, Debug)]
struct Args {
    #[structopt(help = "full commit hashes of the rustc builds; all 40 digits are needed",
                required_raw = "true")]
    commits: Vec<String>,

    #[structopt(short = "b", long = "build", help = "build type",
                possible_values_raw = r#"&["master", "try", "alt"]"#, default_value = "master")]
    build: Build,

    #[structopt(short = "s", long = "server",
                help = "the server path which stores the compilers",
                default_value = "https://s3-us-west-1.amazonaws.com/rust-lang-ci2")]
    server: String,

    #[structopt(short = "i", long = "host", help = "the triples of host platform")]
    host: Option<String>,

    #[structopt(short = "t", long = "targets",
                help = "additional target platforms to install, besides the host platform")]
    targets: Vec<String>,

    #[structopt(short = "p", long = "proxy", help = "the HTTP proxy for all download requests")]
    proxy: Option<String>,
}

macro_rules! path_buf {
    ($($e:expr),*$(,)*) => { [$($e),*].iter().collect::<PathBuf>() }
}

fn download_tar_xz(client: &Client, url: &str) -> Result<(), Error> {
    let _ = remove_dir_all("downloads");
    let _ = create_dir("downloads");

    eprintln!("downloading <{}>...", url);

    let response = client.get(url).send()?.error_for_status()?;

    let length = response
        .headers()
        .get::<ContentLength>()
        .map(|h| h.0)
        .unwrap_or(0);
    let mut bar = ProgressBar::new(length);
    bar.set_units(Units::Bytes);

    {
        let response = TeeReader::new(response, &mut bar);
        let response = XzDecoder::new(response);
        Archive::new(response)
            .unpack("downloads")
            .context("unarchiving")?;
    }

    bar.finish_print("completed");

    Ok(())
}

fn install_single_toolchain(
    client: &Client,
    prefix: &str,
    toolchains_path: &Path,
    commit: &str,
    build: Build,
    rustc_filename: &str,
    rust_std_targets: &[&str],
) -> Result<(), Error> {
    let dest = format!("{}{}", commit, build.tag());
    let toolchain_path = toolchains_path.join(&dest);
    if toolchain_path.is_dir() {
        eprintln!("toolchain `{}` is already installed", dest);
        return Ok(());
    }

    // download rustc.
    let rustc_url = format!("{}/{}/{}.tar.xz", prefix, commit, rustc_filename);
    download_tar_xz(&client, &rustc_url).context("downloading rustc")?;
    let rustc_src_folder = path_buf!["downloads", &rustc_filename, "rustc"];
    rename(&rustc_src_folder, &dest).context("staging rustc")?;

    // download libstd.
    for target in rust_std_targets {
        let rust_std_filename = format!("rust-std-nightly-{}", target);
        let rust_std_url = format!("{}/{}/{}.tar.xz", prefix, commit, rust_std_filename);
        download_tar_xz(&client, &rust_std_url).context("downloading libstd")?;
        let rust_std_src_folder = path_buf![
            "downloads",
            &rust_std_filename,
            &format!("rust-std-{}", target),
            "lib",
            "rustlib",
            target,
            "lib",
        ];
        let rust_std_dest_folder = path_buf![&dest, "lib", "rustlib", target, "lib"];
        create_dir_all(&rust_std_dest_folder).context("preparing libstd")?;
        // We cannot simply move the entire folder, since the rustc tarball
        // starts to populate the `rustlib/` folder after migrating to LLVM 6.
        for lib in read_dir(rust_std_src_folder)? {
            let lib = lib?;
            let src = lib.path();
            let dest = rust_std_dest_folder.join(lib.file_name());
            rename(src, dest)?;
        }
    }

    // install.
    rename(&dest, toolchain_path).context("installing")?;
    Ok(())
}

fn run() -> Result<(), Error> {
    let args = Args::from_args();

    let mut client_builder = ClientBuilder::new();
    if let Some(proxy) = args.proxy {
        client_builder.proxy(Proxy::all(&proxy)?);
    }
    let client = client_builder.build()?;

    let mut toolchains_path = match var_os("RUSTUP_HOME") {
        Some(h) => PathBuf::from(h),
        None => {
            let mut home = home_dir().expect("$HOME is undefined?");
            home.push(".rustup");
            home
        }
    };
    toolchains_path.push("toolchains");
    if !toolchains_path.is_dir() {
        eprintln!(
            "`{}` is not a directory. please install rustup.",
            toolchains_path.display()
        );
        exit(1);
    }

    let host = args.host.as_ref().map(|s| &**s).unwrap_or(env!("HOST"));
    let rustc_filename = format!("rustc-nightly-{}", host);

    let rust_std_targets = args.targets
        .iter()
        .map(|s| &**s)
        .chain(once(host))
        .collect::<Vec<_>>();

    let toolchains_dir = TempDir::new("toolchains")?;
    set_current_dir(toolchains_dir.path())?;

    let prefix = format!("{}/rustc-builds{}", args.server, args.build.tag());

    for commit in args.commits {
        if let Err(e) = install_single_toolchain(
            &client,
            &prefix,
            &toolchains_path,
            &commit,
            args.build,
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
