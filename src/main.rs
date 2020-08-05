use crate::cfg::Cfg;
use crate::cmd::Opt;
use anyhow::Result;
use log::debug;
use std::fs::{create_dir_all, remove_dir_all, OpenOptions};
use std::io::prelude::*;
use std::thread;
use std::time::Duration;
use structopt::StructOpt;
use tempfile::TempDir;

mod cfg;
mod cmd;
mod pkg;
mod pkgdir;
mod pkgmgr;

fn main() -> Result<()> {
    pretty_env_logger::init();
    let opt = Opt::from_args();
    debug!("parsed opts: {:#?}", opt);
    let cfg = Cfg::load()?;
    debug!("read config: {:#?}", cfg);
    match opt {
        Opt::Get { path } => get(&cfg, path)?,
        Opt::Init => init()?,
    }
    Ok(())
}

fn get<S: Into<String>>(cfg: &Cfg, path: S) -> Result<()> {
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

fn cleanup_files(_tmp_dir: &TempDir) -> Result<()> {
    unimplemented!("not implemented yet");
}

fn copy_files() -> Result<()> {
    unimplemented!("not implemented yet");
}

fn init() -> Result<()> {
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

#[cfg(test)]
mod test {
    use super::*;
    use anyhow::Result;
    use std::env;
    use std::fs::{read_to_string, File};
    use std::path::Path;
    use tempfile::TempDir;

    #[test]
    fn test_zip_pkg() -> Result<()> {
        // given
        let tmp_dir = TempDir::new()?;

        // when
        pkg::zip_pkg(&tmp_dir)?;

        // then
        assert_eq!(Path::new(&tmp_dir.path().join("pkg.zip")).exists(), true);
        Ok(())
    }

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
