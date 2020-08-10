use crate::cfg::Cfg;
use crate::path::Path;
use crate::pkg;
use crate::pkgdir;
use crate::pkgmgr;
use anyhow::Result;
use fs_extra::{dir, dir::CopyOptions as DirOpts};
use fs_extra::{file, file::CopyOptions as FileOpts};
use log::{debug, info};
use std::fs::{self, File, OpenOptions};
use std::io::prelude::*;
use std::io::BufReader;
use std::path::{Path as OsPath, PathBuf};
use std::thread;
use std::time::Duration;
use structopt::StructOpt;
use tempfile::TempDir;
use walkdir::{DirEntry, WalkDir};

#[derive(Debug, StructOpt)]
#[structopt(
    name = "je",
    about = "Jcr Exchange - easy download and upload files to and from JCR"
)]
pub(crate) struct Opt {
    /// Enables INFO logs
    #[structopt(short, long, parse(from_occurrences))]
    pub(crate) verbose: u8,
    #[structopt(subcommand)]
    pub(crate) cmd: Cmd,
}

#[derive(Debug, StructOpt)]
pub(crate) enum Cmd {
    /// Download content to local file system
    Get {
        /// path to download
        path: String,
    },
    /// Upload content to AEM instance
    Put {
        /// path to upload
        path: String,
    },
    Init,
}

pub(crate) fn init() -> Result<()> {
    info!("initializing config file ./.je");
    let cfg = Cfg::default();
    let mut config_file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(".je")?;
    config_file.write_all(toml::to_string(&cfg)?.as_bytes())?;
    Ok(())
}

pub(crate) fn get(cfg: &Cfg, path: &Path) -> Result<()> {
    info!("executing 'get {}'", path.full());
    let pkg = pkgdir::Pkg::default();
    let tmp_dir = pkgdir::mk(&path, &pkg)?;
    pkg::zip_pkg(&tmp_dir)?;
    pkgmgr::upload_pkg(cfg, &tmp_dir)?;
    pkgmgr::build_pkg(cfg, &pkg)?;
    thread::sleep(Duration::from_millis(100));
    pkgdir::clean(&tmp_dir)?;
    pkgmgr::download_pkg(cfg, &tmp_dir, &pkg)?;
    pkgmgr::delete_pkg(cfg, &pkg)?;
    pkg::unzip_pkg(&tmp_dir)?;
    cleanup_files(cfg, &tmp_dir)?;
    mv_files_back(&tmp_dir, &path)?;
    Ok(())
}

fn cleanup_files(cfg: &Cfg, tmp_dir: &TempDir) -> Result<()> {
    info!("cleaning files from unwanted properties");
    for entry in WalkDir::new(tmp_dir.path().join("jcr_root"))
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| is_xml_file(e))
    {
        debug!("cleaning file {}", entry.path().display());
        let mut file = File::open(entry.path())?;
        let reader = BufReader::new(&mut file);
        let lines: Vec<_> = reader
            .lines()
            .map(|l| l.expect("could not read line"))
            .filter_map(|l| allowed_prop(l, &cfg.ignore_properties))
            .collect();

        let mut file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(entry.path())?;
        debug!("writing cleaned lines back to file");
        for line in lines {
            file.write_all(line.as_bytes())?;
        }
    }

    Ok(())
}

fn is_xml_file(e: &DirEntry) -> bool {
    is_file(e) && is_xml(e)
}

fn is_file(e: &DirEntry) -> bool {
    let is_file = e.path().is_file();
    debug!(
        "{} {} a file",
        e.path().display(),
        if is_file { "is" } else { "is not" }
    );
    is_file
}

fn is_xml(e: &DirEntry) -> bool {
    let is_xml = e.path().ends_with(".content.xml");
    debug!(
        "{} {} xml file",
        e.path().display(),
        if is_xml { "is" } else { "is not" }
    );
    is_xml
}

fn allowed_prop<S: Into<String>>(line: S, ignore_properties: &[String]) -> Option<String> {
    let line = line.into();
    let mut result = true;
    for ignore_prop in ignore_properties {
        debug!("checking if {} contains {}", line, ignore_prop);
        if line.contains(ignore_prop) {
            debug!("line {} contains not allowed property, removing", line);
            result = false;
            break;
        }
    }
    if result {
        Some(format!("{}\n", line))
    } else {
        None
    }
}

fn mv_files_back(tmp_dir: &TempDir, path: &Path) -> Result<()> {
    let from = tmp_dir.path().join(path.from_root());
    info!("moving files from {} to {}", from.display(), path.full());
    list_files(&from);
    if path.is_dir() {
        fs::remove_dir_all(path.full())?;
    }
    fs::rename(from, path.full())?;
    Ok(())
}

fn list_files<P: AsRef<OsPath>>(path: P) {
    debug!("files under {}:", path.as_ref().display());
    for entry in WalkDir::new(path).into_iter().filter_map(Result::ok) {
        debug!("\t- {}", entry.path().display());
    }
}

pub(crate) fn put(cfg: &Cfg, path: &Path) -> Result<()> {
    info!("executing 'put {}'", path.full());
    let pkg = pkgdir::Pkg::default();
    let tmp_dir = pkgdir::mk(path, &pkg)?;
    cp_files_to_pkg(path, &tmp_dir)?;
    pkg::zip_pkg(&tmp_dir)?;
    pkgmgr::upload_pkg(cfg, &tmp_dir)?;
    pkgmgr::install_pkg(cfg, &pkg)?;
    pkgmgr::delete_pkg(cfg, &pkg)?;
    Ok(())
}

fn cp_files_to_pkg(path: &Path, tmp_dir: &TempDir) -> Result<()> {
    let dst_path = dst_path(path, tmp_dir)?;
    info!(
        "copying files from {} to {}",
        path.full(),
        dst_path.display()
    );
    fs::create_dir_all(tmp_dir.path().join(path.parent_from_root()?))?;
    if path.is_dir() {
        debug!("{} is a dir", path.full());
        dir::copy(path.full(), dst_path, &DirOpts::new())?;
    } else {
        debug!("{} is a file", path.full());
        file::copy(path.full(), dst_path, &FileOpts::new())?;
    }
    Ok(())
}

fn dst_path(path: &Path, tmp_dir: &TempDir) -> Result<PathBuf> {
    let result = if path.is_dir() {
        tmp_dir.path().join(path.parent_from_root()?)
    } else {
        tmp_dir.path().join(path.from_root())
    };
    Ok(result)
}

#[cfg(test)]
mod test {
    use super::*;
    use anyhow::Result;
    use std::env;
    use std::fs::{read_to_string, File};
    use tempfile::TempDir;

    #[test]
    fn test_init() -> Result<()> {
        // given
        let initial_dir = env::current_dir()?;
        let tmp_dir = TempDir::new()?;
        env::set_current_dir(&tmp_dir)?;

        // when
        init()?;

        // then
        let cfg_content = read_to_string("./.je")?;
        assert_eq!(
            cfg_content,
            r#"ignore_properties = []

[instance]
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
        init()?;

        // then
        let cfg_content = read_to_string("./.je")?;
        assert_eq!(
            cfg_content,
            r#"ignore_properties = []

[instance]
addr = "http://localhost:4502"
user = "admin"
pass = "admin"
"#
        );
        env::set_current_dir(initial_dir)?;
        Ok(())
    }
}
