use anyhow::Result;
use log::debug;
use std::fs::create_dir_all;
use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;
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
    mk_pkg_dir(&path)?;
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

fn mk_pkg_dir(path: &str) -> Result<TempDir> {
    let tmp_dir = TempDir::new()?;
    mk_jcr_root_dir(&tmp_dir)?;
    mk_vault_dir(&tmp_dir)?;
    write_filter_content(&tmp_dir, content_path(path))?;
    write_properties_content(&tmp_dir)?;
    Ok(tmp_dir)
}

fn mk_jcr_root_dir(tmp_dir: &TempDir) -> Result<()> {
    create_dir_all(tmp_dir.path().join("jcr_root"))?;
    Ok(())
}

fn mk_vault_dir(tmp_dir: &TempDir) -> Result<()> {
    create_dir_all(&vault_path(tmp_dir))?;
    Ok(())
}

fn vault_path(tmp_dir: &TempDir) -> PathBuf {
    tmp_dir.path().join("META-INF/vault")
}

fn content_path<S: Into<String>>(path: S) -> String {
    let path = path.into();
    let parts: Vec<&str> = path.split("jcr_root").collect();
    assert_eq!(parts.len(), 2);
    parts[1].into()
}

fn write_filter_content<S: Into<String>>(tmp_dir: &TempDir, content_path: S) -> Result<()> {
    let mut filter_file = File::create(format!("{}/filter.xml", vault_path(&tmp_dir).display()))?;
    filter_file.write_all(filter_content(content_path).as_bytes())?;
    Ok(())
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

fn write_properties_content(tmp_dir: &TempDir) -> Result<()> {
    let mut prop_file = File::create(format!("{}/properties.xml", vault_path(&tmp_dir).display()))?;
    prop_file.write_all(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="no"?>
<!DOCTYPE properties SYSTEM "http://java.sun.com/dtd/properties.dtd">
<properties>
    <entry key="name">je-package</entry>
    <entry key="version">1.0.0</entry>
    <entry key="group">je</entry>
</properties>"#
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
    use super::content_path;
    use super::mk_jcr_root_dir;
    use super::mk_pkg_dir;
    use super::mk_vault_dir;
    use super::write_filter_content;
    use super::write_properties_content;
    use anyhow::Result;
    use std::fs::create_dir_all;
    use std::fs::read_to_string;
    use std::path::Path;
    use tempfile::TempDir;

    #[test]
    fn test_mk_jcr_root_dir() -> Result<()> {
        // given
        let tmp_dir = TempDir::new()?;

        // when
        mk_jcr_root_dir(&tmp_dir)?;

        // then
        assert_eq!(
            Path::new(&format!("{}/jcr_root", tmp_dir.path().display())).exists(),
            true
        );
        Ok(())
    }

    #[test]
    fn test_mk_vault_dir() -> Result<()> {
        // given
        let tmp_dir = TempDir::new()?;

        // when
        mk_vault_dir(&tmp_dir)?;

        // then
        assert_eq!(
            Path::new(&format!("{}/META-INF/vault", tmp_dir.path().display())).exists(),
            true
        );
        Ok(())
    }

    #[test]
    fn test_write_filter_content() -> Result<()> {
        // given
        let tmp_dir = TempDir::new()?;
        create_dir_all(&format!("{}/META-INF/vault", tmp_dir.path().display()))?;

        // when
        write_filter_content(&tmp_dir, "/content/path")?;

        // then
        assert_eq!(
            Path::new(&format!(
                "{}/META-INF/vault/filter.xml",
                tmp_dir.path().display()
            ))
            .exists(),
            true
        );
        let filter_contents = read_to_string(format!(
            "{}/META-INF/vault/filter.xml",
            tmp_dir.path().display()
        ))?;
        assert_eq!(
            filter_contents,
            r#"<?xml version="1.0" encoding="UTF-8"?>
<workspaceFilter version="1.0">
    <filter root="/content/path"/>
</workspaceFilter>
        "#,
        );
        Ok(())
    }

    #[test]
    fn test_content_path_with_correct_paths() {
        // given
        let path = "/home/zbychu/project/test/jcr_root/content/abc";

        // when
        let content_path = content_path(path);

        // then
        assert_eq!(content_path, "/content/abc");
    }

    #[test]
    #[should_panic]
    fn test_content_path_with_broken_paths() {
        // given
        let path = "/home/zbychu/project/test/content/abc";

        // should panic
        content_path(path);
    }

    #[test]
    fn test_write_properties_content() -> Result<()> {
        // given
        let tmp_dir = TempDir::new()?;
        create_dir_all(&format!("{}/META-INF/vault", tmp_dir.path().display()))?;

        // when
        write_properties_content(&tmp_dir)?;

        // then
        assert_eq!(
            Path::new(&format!(
                "{}/META-INF/vault/properties.xml",
                tmp_dir.path().display()
            ))
            .exists(),
            true
        );
        let properties_contents = read_to_string(format!(
            "{}/META-INF/vault/properties.xml",
            tmp_dir.path().display()
        ))?;
        assert_eq!(
            properties_contents,
            r#"<?xml version="1.0" encoding="UTF-8" standalone="no"?>
<!DOCTYPE properties SYSTEM "http://java.sun.com/dtd/properties.dtd">
<properties>
    <entry key="name">je-package</entry>
    <entry key="version">1.0.0</entry>
    <entry key="group">je</entry>
</properties>"#,
        );
        Ok(())
    }

    #[test]
    fn test_mk_pkg_dir() -> Result<()> {
        // given
        let file_path = "/home/user/project/jcr_root/content/client";

        // when
        let tmp_dir_path = mk_pkg_dir(file_path)?;

        // then
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
        assert_eq!(
            filter_contents,
            r#"<?xml version="1.0" encoding="UTF-8"?>
<workspaceFilter version="1.0">
    <filter root="/content/client"/>
</workspaceFilter>
        "#,
        );
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
