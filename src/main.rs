use anyhow::Result;
use log::debug;
use std::env;
use std::fs::create_dir_all;
use std::fs::File;
use std::io::prelude::*;
use structopt::StructOpt;
use tempfile::TempDir;

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

#[must_use]
fn mk_jcr_root_dir(path: &str) -> Result<TempDir> {
    let tmp_dir_new = TempDir::new()?;
    let jcr_root_path_new = tmp_dir_new.path().join("jcr_root");
    debug!("jcr_root path: {}", jcr_root_path_new.display());
    create_dir_all(jcr_root_path_new)?;

    let vault_path = tmp_dir_new.path().join("META-INF/vault");
    create_dir_all(&vault_path)?;

    let parts: Vec<&str> = path.split("jcr_root").collect();
    assert_eq!(parts.len(), 2);
    let content_path = parts[1];

    let mut filter_file = File::create(format!("{}/filter.xml", vault_path.display()))?;
    filter_file.write_all(filter_content(content_path).as_bytes())?;

    let mut filter_file = File::create(format!("{}/properties.xml", vault_path.display()))?;
    filter_file.write_all(
        format!(
            r#"<?xml version="1.0" encoding="UTF-8" standalone="no"?>
<!DOCTYPE properties SYSTEM "http://java.sun.com/dtd/properties.dtd">
<properties>
    <entry key="name">$(to_xml $pkgName)</entry>
    <entry key="version">$(to_xml $pkgVersion)</entry>
    <entry key="group">$(to_xml $pkgGroup)</entry>
</properties>"#
        )
        .as_bytes(),
    )?;
    Ok(tmp_dir_new)
}

fn filter_content<S: Into<String>>(path: S) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<workspaceFilter version="1.0">
    <filter root="{}"/>
</workspaceFilter>
        "#,
        path.into()
    )
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
    use super::filter_content;
    use super::mk_jcr_root_dir;
    use anyhow::Result;
    use std::fs::read_to_string;
    use std::path::Path;

    #[test]
    fn test_mk_jcr_root_dir() -> Result<()> {
        let tmp_dir_path = mk_jcr_root_dir("/home/user/project/jcr_root/content/client")?;
        assert_eq!(
            Path::new(&format!("{}/jcr_root", tmp_dir_path.path().display())).exists(),
            true
        );
        assert_eq!(
            Path::new(&format!(
                "{}/META-INF/vault/filter.xml",
                tmp_dir_path.path().display()
            ))
            .exists(),
            true
        );
        let filter_contents = read_to_string(format!(
            "{}/META-INF/vault/filter.xml",
            tmp_dir_path.path().display()
        ))?;
        assert_eq!(filter_contents, filter_content("/content/client"));
        assert_eq!(
            Path::new(&format!(
                "{}/META-INF/vault/properties.xml",
                tmp_dir_path.path().display()
            ))
            .exists(),
            true
        );

        Ok(())
    }
}
