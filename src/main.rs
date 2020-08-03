use anyhow::Result;
use base64::encode;
use log::debug;
use reqwest::blocking::multipart;
use reqwest::blocking::Client;
use std::env;
use std::fs::create_dir_all;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::path::PathBuf;
use std::thread;
use std::time::Duration;
use std::time::{SystemTime, UNIX_EPOCH};
use structopt::StructOpt;
use tempfile::TempDir;
use walkdir::WalkDir;
use zip::write::FileOptions;
use zip::ZipWriter;

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

#[derive(Debug)]
struct Pkg {
    name: String,
    version: String,
    group: String,
}

impl Pkg {
    fn path(&self) -> String {
        format!("{}/{}-{}.zip", self.group, self.name, self.version)
    }
}

impl Default for Pkg {
    fn default() -> Self {
        Self {
            name: "je-pkg".into(),
            version: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("failed to count secs from EPOCH")
                .as_secs()
                .to_string(),
            group: "je".into(),
        }
    }
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
    let path = path.into();
    let pkg = Pkg::default();

    let tmp_dir = mk_pkg_dir(&path, &pkg)?;
    zip_pkg(&tmp_dir)?;
    upload_pkg(&tmp_dir)?;
    build_pkg(&pkg)?;
    thread::sleep(Duration::from_millis(100));
    download_pkg(&tmp_dir, &pkg)?;
    unzip_pkg()?;
    copy_files()?;
    Ok(())
}

fn mk_pkg_dir(path: &str, pkg: &Pkg) -> Result<TempDir> {
    let tmp_dir = TempDir::new()?;
    mk_jcr_root_dir(&tmp_dir)?;
    mk_vault_dir(&tmp_dir)?;
    write_filter_content(&tmp_dir, content_path(path))?;
    write_properties_content(&tmp_dir, pkg)?;
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

fn write_properties_content(tmp_dir: &TempDir, pkg: &Pkg) -> Result<()> {
    let mut prop_file = File::create(format!("{}/properties.xml", vault_path(&tmp_dir).display()))?;
    prop_file.write_all(
        format!(
            r#"<?xml version="1.0" encoding="UTF-8" standalone="no"?>
<!DOCTYPE properties SYSTEM "http://java.sun.com/dtd/properties.dtd">
<properties>
    <entry key="name">{}</entry>
    <entry key="version">{}</entry>
    <entry key="group">{}</entry>
</properties>"#,
            pkg.name, pkg.version, pkg.group
        )
        .as_bytes(),
    )?;
    Ok(())
}

fn copy_files() -> Result<()> {
    Ok(())
}

fn zip_pkg(tmp_dir: &TempDir) -> Result<()> {
    let initial_dir = env::current_dir()?;

    env::set_current_dir(tmp_dir)?;

    let writer = File::create(tmp_dir.path().join("pkg.zip"))?;
    let mut zip = ZipWriter::new(writer);
    let options = FileOptions::default();

    for path in vec!["./jcr_root", "./META-INF"].iter() {
        let walkdir = WalkDir::new(path);
        let mut buffer = Vec::new();
        for entry in &mut walkdir.into_iter().flat_map(|e| e.ok()) {
            let path = entry.path();
            if path.is_file() {
                zip.start_file_from_path(path, options)?;
                let mut f = File::open(path)?;
                f.read_to_end(&mut buffer)?;
                zip.write_all(&*buffer)?;
                buffer.clear();
            } else {
                zip.add_directory_from_path(Path::new(path), options)?;
            }
        }
    }

    zip.finish()?;

    env::set_current_dir(initial_dir)?;
    Ok(())
}

fn upload_pkg(tmp_dir: &TempDir) -> Result<()> {
    let form = multipart::Form::new().file("package", tmp_dir.path().join("pkg.zip"))?;
    let client = Client::new();
    let resp = client
        .post("http://localhost:4502/crx/packmgr/service/.json?cmd=upload")
        .header("Authorization", format!("Basic {}", encode("admin:admin")))
        .multipart(form)
        .send()?;
    debug!("resp: {:#?}", resp);
    Ok(())
}

fn build_pkg(pkg: &Pkg) -> Result<()> {
    let client = Client::new();
    let resp = client
        .post(&format!(
            "http://localhost:4502/crx/packmgr/service/script.html/etc/packages/{}?cmd=build",
            pkg.path(),
        ))
        .header("Authorization", format!("Basic {}", encode("admin:admin")))
        .send()?;
    debug!("build pkg response: {:#?}", resp);
    Ok(())
}

fn download_pkg(tmp_dir: &TempDir, pkg: &Pkg) -> Result<()> {
    let client = Client::new();
    let resp = client
        .get(&format!(
            "http://localhost:4502/etc/packages/{}",
            pkg.path(),
        ))
        .header("Authorization", format!("Basic {}", encode("admin:admin")))
        .send()?;
    debug!("download pkg response: {:#?}", resp);
    let mut pkg_file = File::create(tmp_dir.path().join("res.zip"))?;
    pkg_file.write_all(&resp.bytes()?)?;
    Ok(())
}

fn unzip_pkg() -> Result<()> {
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
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
        let pkg = Pkg::default();

        // when
        write_properties_content(&tmp_dir, &pkg)?;

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
            format!(
                r#"<?xml version="1.0" encoding="UTF-8" standalone="no"?>
<!DOCTYPE properties SYSTEM "http://java.sun.com/dtd/properties.dtd">
<properties>
    <entry key="name">{}</entry>
    <entry key="version">{}</entry>
    <entry key="group">{}</entry>
</properties>"#,
                pkg.name, pkg.version, pkg.group
            ),
        );
        Ok(())
    }

    #[test]
    fn test_mk_pkg_dir() -> Result<()> {
        // given
        let file_path = "/home/user/project/jcr_root/content/client";
        let pkg = Pkg::default();

        // when
        let tmp_dir_path = mk_pkg_dir(file_path, &pkg)?;

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

    #[test]
    fn test_zip_pkg() -> Result<()> {
        // given
        let tmp_dir = TempDir::new()?;

        // when
        zip_pkg(&tmp_dir)?;

        // then
        assert_eq!(Path::new(&tmp_dir.path().join("pkg.zip")).exists(), true);
        Ok(())
    }
}
