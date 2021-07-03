use crate::args::{GetArgs, GetBundleArgs, PutArgs};
use crate::cfg::Cfg;
use crate::fsops;
use crate::http::AemClient;
use crate::path::Path;
use crate::pkg;
use crate::pkgdir;
use crate::pkgmgr;
use anyhow::Result;
use fs_extra::{dir, dir::CopyOptions as DirOpts};
use fs_extra::{file, file::CopyOptions as FileOpts};
use log::{debug, info};
use std::fs::{self, OpenOptions};
use std::io::prelude::*;
use std::path::PathBuf;
use std::thread;
use std::time::Duration;
use structopt::StructOpt;
use tempfile::TempDir;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "je",
    about = "Jcr Exchange - easy download and upload files to and from JCR"
)]
pub(crate) struct Opt {
    /// Enables logs:
    /// -v - enables INFO log level
    /// -vv - enables DEBUG log level
    #[structopt(short, long, parse(from_occurrences))]
    pub(crate) verbose: u8,

    /// If enabled, deployed to AEM packages are left intact (are not deleted) to allow
    /// investigation
    #[structopt(short, long)]
    pub(crate) debug: bool,

    /// Profile selection.
    #[structopt(short, long)]
    pub(crate) profile: Option<String>,

    #[structopt(subcommand)]
    pub(crate) cmd: Cmd,
}

impl Default for Opt {
    fn default() -> Self {
        Self {
            verbose: 0,
            debug: false,
            profile: None,
            cmd: Cmd::Get { path: "".into() },
        }
    }
}

#[derive(Debug, StructOpt, Clone)]
pub(crate) enum Cmd {
    /// Downloads content to local file system
    Get {
        /// path to download
        path: String,
    },
    /// Downloads bundle (pack of crx paths) defined in config file
    GetBundle {
        /// name of the bundle
        name: String,
    },
    /// Uploads content to AEM instance
    Put {
        /// path to upload
        path: String,
    },
    /// Initializes configuration file
    Init,
}

pub(crate) fn init(cfg: &Cfg) -> Result<()> {
    info!("initializing config file ./.je");
    let mut config_file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(".je")?;
    config_file.write_all(toml::to_string(&cfg)?.as_bytes())?;
    Ok(())
}

pub(crate) fn get(args: &GetArgs) -> Result<()> {
    info!("executing 'get {}'", args.path().full());
    let pkg = pkgdir::Pkg::default();
    let tmp_dir = pkgdir::mksimple(args.path(), &pkg)?;
    let client = AemClient::new(args.instance());
    pkg::zip_pkg(&tmp_dir)?;
    pkgmgr::upload_pkg(&client, &tmp_dir)?;
    pkgmgr::build_pkg(&client, &pkg)?;
    thread::sleep(Duration::from_millis(100));
    pkgdir::clean(&tmp_dir)?;
    pkgmgr::download_pkg(&client, &tmp_dir, &pkg)?;
    pkgmgr::delete_pkg(&client, args.debug(), &pkg)?;
    pkg::unzip_pkg(&tmp_dir)?;
    fsops::cleanup_files(args.ignore_properties(), &tmp_dir)?;
    fsops::mv_files_back(&tmp_dir, args.path())?;
    Ok(())
}

pub(crate) fn put(args: &PutArgs) -> Result<()> {
    info!("executing 'put {}'", args.path().full());
    let pkg = pkgdir::Pkg::default();
    let tmp_dir = pkgdir::mksimple(args.path(), &pkg)?;
    let client = AemClient::new(args.instance());
    cp_files_to_pkg(args.path(), &tmp_dir)?;
    pkg::zip_pkg(&tmp_dir)?;
    pkgmgr::upload_pkg(&client, &tmp_dir)?;
    pkgmgr::install_pkg(&client, &pkg)?;
    pkgmgr::delete_pkg(&client, args.debug(), &pkg)?;
    Ok(())
}

pub(crate) fn get_bundle(args: &GetBundleArgs) -> Result<()> {
    info!("executing 'get bundle {:?}'", args.bundle());
    let pkg = pkgdir::Pkg::default();
    let tmp_dir = pkgdir::mkbundle(args.bundle(), &pkg)?;
    let client = AemClient::new(args.instance());
    pkg::zip_pkg(&tmp_dir)?;
    pkgmgr::upload_pkg(&client, &tmp_dir)?;
    pkgmgr::build_pkg(&client, &pkg)?;
    thread::sleep(Duration::from_millis(100));
    pkgdir::clean(&tmp_dir)?;
    pkgmgr::download_pkg(&client, &tmp_dir, &pkg)?;
    pkgmgr::delete_pkg(&client, args.debug(), &pkg)?;
    pkg::unzip_pkg(&tmp_dir)?;
    fsops::cleanup_files(args.ignore_properties(), &tmp_dir)?;
    fsops::mv_bundle_back(&tmp_dir, args.bundle())?;
    Ok(())
}

fn cp_files_to_pkg(path: &Path, tmp_dir: &TempDir) -> Result<()> {
    let dst_path = dst_path(path, tmp_dir);
    info!(
        "copying files from {} to {}",
        path.full(),
        dst_path.display()
    );
    fs::create_dir_all(tmp_dir.path().join(path.parent_from_root()))?;
    if path.is_dir() {
        debug!("{} is a dir", path.full());
        dir::copy(path.full(), dst_path, &DirOpts::new())?;
    } else {
        debug!("{} is a file", path.full());
        file::copy(path.full(), dst_path, &FileOpts::new())?;
    }
    Ok(())
}

fn dst_path(path: &Path, tmp_dir: &TempDir) -> PathBuf {
    let result = if path.is_dir() {
        tmp_dir.path().join(path.parent_from_root())
    } else {
        tmp_dir.path().join(path.from_root())
    };
    result
}

#[cfg(test)]
mod test {
    use super::*;
    use anyhow::Result;
    use std::env;
    use std::fs::{create_dir_all, read_to_string, File};
    use tempfile::TempDir;

    #[test]
    fn test_init() -> Result<()> {
        // given
        let initial_dir = env::current_dir()?;
        let tmp_dir = TempDir::new()?;
        env::set_current_dir(&tmp_dir)?;

        // when
        init(&Cfg::default())?;

        // then
        let cfg_content = read_to_string("./.je")?;
        assert_eq!(
            cfg_content,
            r#"version = "0.3.0"
ignore_properties = []

[[profile]]
name = "author"
addr = "http://localhost:4502"
user = "admin"
pass = "admin"
"#
        );
        env::set_current_dir(initial_dir)?;
        Ok(())
    }

    #[test]
    fn test_init_when_file_already_exists() -> Result<()> {
        // given
        let initial_dir = env::current_dir()?;
        let tmp_dir = TempDir::new()?;
        env::set_current_dir(&tmp_dir)?;
        let mut cfg_file = File::create(".je")?;
        cfg_file.write_all(b"not important")?;

        // when
        init(&Cfg::default())?;

        // then
        let cfg_content = read_to_string("./.je")?;
        assert_eq!(
            cfg_content,
            r#"version = "0.3.0"
ignore_properties = []

[[profile]]
name = "author"
addr = "http://localhost:4502"
user = "admin"
pass = "admin"
"#
        );
        env::set_current_dir(initial_dir)?;
        Ok(())
    }

    #[test]
    fn test_dst_path_when_path_is_a_file() -> Result<()> {
        // given
        let homepath = "/tmp/je-test/jcr_root/content/project/en_gb/home";
        create_dir_all(homepath)?;
        let filepath = format!("{}/.content.xml", homepath);
        File::create(&filepath)?;

        let tmp_dir = TempDir::new()?;
        let expected_path = tmp_dir
            .path()
            .join("jcr_root/content/project/en_gb/home/.content.xml");
        let path = Path::new(filepath);

        // when
        let dst = dst_path(&path, &tmp_dir);

        // then
        assert_eq!(dst, expected_path);
        Ok(())
    }

    #[test]
    fn test_dst_path_when_path_is_a_dir() -> Result<()> {
        // given
        let dirpath = "/tmp/je-test/jcr_root/content/project/en_gb/home/test";
        create_dir_all(dirpath)?;
        let path = Path::new(dirpath);

        let tmp_dir = TempDir::new()?;
        let expected_path = tmp_dir.path().join("jcr_root/content/project/en_gb/home");

        // when
        let dst = dst_path(&path, &tmp_dir);

        // then
        assert_eq!(dst, expected_path);
        Ok(())
    }
}
