use anyhow::Result;
use log::debug;
use std::env;
use std::fs::create_dir_all;
use std::path::Path;
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
    mk_jcr_root_dir(&path)?;
    copy_files()?;
    build_pkg()?;
    upload_pkg()?;
    install_pkg()?;
    cleanup_tmp()?;
    debug!("downloading path {}", path);
    Ok(())
}

fn mk_jcr_root_dir(path: &str) -> Result<()> {
    let parts: Vec<&str> = path.split("jcr_root/").collect();
    assert_eq!(parts.len(), 2);
    let path = parts[1];
    let tmp_dir = env::temp_dir();
    let tmp_dir_path = tmp_dir.as_path().to_str().unwrap_or("/tmp");
    let path = format!("{}/je/jcr_root/{}", tmp_dir_path, path);
    create_dir_all(path)?;
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
        mk_jcr_root_dir("/home/zbychu/test/jcr_root/some/dir")?;
        assert_eq!(Path::new("/tmp/je/jcr_root").exists(), true);
        Ok(())
    }
}
