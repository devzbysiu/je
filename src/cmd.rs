use crate::cfg::Cfg;
use crate::pkg;
use crate::pkgdir;
use crate::pkgmgr;
use anyhow::Result;
use log::debug;
use std::fs::{create_dir_all, remove_dir_all, OpenOptions};
use std::io::prelude::*;
use std::thread;
use std::time::Duration;
use structopt::StructOpt;
use tempfile::TempDir;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "je",
    about = "Jcr Exchange - easy download and upload files to and from JCR"
)]
pub(crate) enum Opt {
    /// Download server content to local file server
    Get {
        /// path to download
        path: String,
    },
    Init,
}

pub(crate) fn get<S: Into<String>>(cfg: &Cfg, path: S) -> Result<()> {
    let path = path.into();
    debug!("executing 'get {}'", path);
    let pkg = pkgdir::Pkg::default();
    let tmp_dir = pkgdir::mk(&path, &pkg)?;
    pkg::zip_pkg(&tmp_dir)?;
    pkgmgr::upload_pkg(&cfg, &tmp_dir)?;
    pkgmgr::build_pkg(&cfg, &pkg)?;
    thread::sleep(Duration::from_millis(100));
    remove_dir_all(&tmp_dir)?;
    create_dir_all(&tmp_dir)?;
    pkgmgr::download_pkg(&tmp_dir, &pkg)?;
    pkg::unzip_pkg(&tmp_dir)?;
    cleanup_files(&tmp_dir)?;
    copy_files()?;
    Ok(())
}

pub(crate) fn init() -> Result<()> {
    debug!("initializing config file ./.je");
    let cfg = Cfg::default();
    let mut config_file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(".je")?;
    config_file.write_all(toml::to_string(&cfg)?.as_bytes())?;
    Ok(())
}

fn cleanup_files(_tmp_dir: &TempDir) -> Result<()> {
    unimplemented!("not implemented yet");
}

fn copy_files() -> Result<()> {
    unimplemented!("not implemented yet");
}
