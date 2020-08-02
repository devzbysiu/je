use anyhow::Result;
use log::debug;
use std::env;
use std::fs::create_dir_all;
use std::fs::File;
use std::io::prelude::*;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "je",
    about = "Jcr Exchange - easy download and upload files to and from JCR"
)]
enum Opt {
    /// Download server content to local file server
    Get {
        /// path to download
        path: String,
    },
}

fn main() -> Result<()> {
    pretty_env_logger::init();
    let opt = Opt::from_args();
    debug!("parsed opts: {:#?}", opt);
    match opt {
        Opt::Get { path } => get(path)?,
    }
    Ok(())
}

fn get<S: Into<String>>(path: S) -> Result<()> {
    // mkdir -p "$tmpDir/jcr_root$filterDirname"
    let path = path.into();
    mk_jcr_root_dir()?;
    build_pkg()?;
    upload_pkg()?;
    install_pkg()?;
    download_pkg()?;
    unzip_pkg()?;
    copy_files()?;
    cleanup_tmp()?;
    debug!("downloading path {}", path);
    Ok(())
}

fn mk_jcr_root_dir() -> Result<()> {
    let tmp_dir = env::temp_dir();
    let tmp_dir_path = tmp_dir.as_path().to_str().unwrap_or("/tmp");
    let jcr_root_path = format!("{}/je/jcr_root", tmp_dir_path);
    create_dir_all(jcr_root_path)?;

    let vault_path = format!("{}/je/META-INF/vault", tmp_dir_path);
    create_dir_all(&vault_path)?;

    let mut filter_file = File::create(format!("{}/filter.xml", vault_path))?;
    filter_file.write_all(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<workspaceFilter version="1.0">
    <filter root="$(to_xml $filter)"/>
</workspaceFilter>
        "#
        .as_bytes(),
    )?;
    Ok(())
}

fn copy_files() -> Result<()> {
    Ok(())
}

fn build_pkg() -> Result<()> {
    Ok(())
}

fn upload_pkg() -> Result<()> {
    Ok(())
}

fn install_pkg() -> Result<()> {
    Ok(())
}

fn download_pkg() -> Result<()> {
    Ok(())
}

fn unzip_pkg() -> Result<()> {
    Ok(())
}

fn cleanup_tmp() -> Result<()> {
    Ok(())
}

#[cfg(test)]
mod test {
    use super::mk_jcr_root_dir;
    use anyhow::Result;
    use std::path::Path;

    #[test]
    fn test_mk_jcr_root_dir() -> Result<()> {
        mk_jcr_root_dir()?;
        assert_eq!(Path::new("/tmp/je/jcr_root").exists(), true);
        assert_eq!(
            Path::new("/tmp/je/META-INF/vault/filter.xml").exists(),
            true
        );
        Ok(())
    }
}
