use crate::cfg::Cfg;
use crate::path::Path;
use crate::pkg;
use crate::pkgdir;
use crate::pkgmgr;
use anyhow::Result;
use log::{debug, info};
use std::fs::{self, OpenOptions};
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
pub(crate) struct Opt {
    /// Enables INFO logs
    #[structopt(short, long)]
    pub(crate) verbose: bool,
    #[structopt(subcommand)]
    pub(crate) cmd: Cmd,
}

#[derive(Debug, StructOpt)]
pub(crate) enum Cmd {
    /// Download server content to local file server
    Get {
        /// path to download
        path: String,
    },
    Init,
}

pub(crate) fn get(cfg: &Cfg, path: Path) -> Result<()> {
    info!("executing 'get {}'", path.full());
    let pkg = pkgdir::Pkg::default();
    let tmp_dir = pkgdir::mk(&path, &pkg)?;
    pkg::zip_pkg(&tmp_dir)?;
    pkgmgr::upload_pkg(&cfg, &tmp_dir)?;
    pkgmgr::build_pkg(&cfg, &pkg)?;
    thread::sleep(Duration::from_millis(100));
    pkgdir::clean(&tmp_dir)?;
    pkgmgr::download_pkg(&tmp_dir, &pkg)?;
    pkg::unzip_pkg(&tmp_dir)?;
    cleanup_files(&tmp_dir)?;
    mv_files(&tmp_dir, &path)?;
    Ok(())
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

fn cleanup_files(_tmp_dir: &TempDir) -> Result<()> {
    info!("cleaning files from unwanted properties");
    Ok(())
}

fn mv_files(tmp_dir: &TempDir, path: &Path) -> Result<()> {
    let from = tmp_dir.path().join(path.with_root());
    info!("moving files from {} to {}", from.display(), path.full());
    fs::rename(from, path.full())?;
    Ok(())
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
