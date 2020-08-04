use anyhow::Result;
use base64::encode;
use log::debug;
use reqwest::blocking::multipart;
use reqwest::blocking::Client;
use serde_derive::{Deserialize, Serialize};
use std::env;
use std::fs::{create_dir_all, read_to_string, remove_dir_all, File};
use std::io;
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
use zip::ZipArchive;
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
    Init,
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

#[derive(Debug, Default, Serialize, Deserialize)]
struct Cfg {
    ignore_properties: Vec<String>,
    instance: Instance,
}

impl Cfg {
    fn load() -> Result<Cfg> {
        debug!("loading config from .je");
        Ok(toml::from_str(&read_to_string(".je")?)?)
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct Instance {
    addr: String,
    user: String,
    pass: String,
}

impl Default for Instance {
    fn default() -> Self {
        Self {
            addr: "http://localhost:4502".into(),
            user: "admin".into(),
            pass: "admin".into(),
        }
    }
}

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
    let pkg = Pkg::default();

    let tmp_dir = mk_pkg_dir(&path, &pkg)?;
    zip_pkg(&tmp_dir)?;
    upload_pkg(&cfg, &tmp_dir)?;
    build_pkg(&cfg, &pkg)?;
    thread::sleep(Duration::from_millis(100));
    remove_dir_all(&tmp_dir)?;
    create_dir_all(&tmp_dir)?;
    download_pkg(&tmp_dir, &pkg)?;
    unzip_pkg(&tmp_dir)?;
    cleanup_files(&tmp_dir)?;
    copy_files()?;
    Ok(())
}

fn mk_pkg_dir(path: &str, pkg: &Pkg) -> Result<TempDir> {
    debug!("creating pkg dir");
    let tmp_dir = TempDir::new()?;
    mk_jcr_root_dir(&tmp_dir)?;
    mk_vault_dir(&tmp_dir)?;
    write_filter_content(&tmp_dir, content_path(path))?;
    write_properties_content(&tmp_dir, pkg)?;
    Ok(tmp_dir)
}

fn mk_jcr_root_dir(tmp_dir: &TempDir) -> Result<()> {
    let jcr_root_dir_path = tmp_dir.path().join("jcr_root");
    debug!("creating jcr_root dir: {}", jcr_root_dir_path.display());
    create_dir_all(jcr_root_dir_path)?;
    Ok(())
}

fn mk_vault_dir(tmp_dir: &TempDir) -> Result<()> {
    debug!("creating vault dir: {}", vault_path(tmp_dir).display());
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
    let filter_path = format!("{}/filter.xml", vault_path(&tmp_dir).display());
    let mut filter_file = File::create(&filter_path)?;
    let filter_content = filter_content(content_path);
    debug!(
        "writing content {} to filter {}",
        filter_content, filter_path
    );
    filter_file.write_all(filter_content.as_bytes())?;
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
    let prop_path = format!("{}/properties.xml", vault_path(&tmp_dir).display());
    let mut prop_file = File::create(&prop_path)?;
    let properties_content = properties_content(&pkg);
    debug!(
        "writing content {} to properties file {}",
        &properties_content, prop_path
    );
    prop_file.write_all(properties_content.as_bytes())?;
    Ok(())
}

fn properties_content(pkg: &Pkg) -> String {
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
}

fn zip_pkg(tmp_dir: &TempDir) -> Result<()> {
    let initial_dir = env::current_dir()?;

    debug!(
        "switching dir from {} to {}",
        &initial_dir.display(),
        &tmp_dir.path().display()
    );
    env::set_current_dir(tmp_dir)?;

    let writer = File::create(tmp_dir.path().join("pkg.zip"))?;
    let mut zip = ZipWriter::new(writer);
    let options = FileOptions::default();

    for path in vec!["./jcr_root", "./META-INF"].iter() {
        let walkdir = WalkDir::new(path);
        let mut buffer = Vec::new();
        debug!("zipping {}", path);
        for entry in &mut walkdir.into_iter().flat_map(|e| e.ok()) {
            let path = entry.path();
            if path.is_file() {
                debug!("{} is a file", path.display());
                zip.start_file_from_path(path, options)?;
                let mut f = File::open(path)?;
                f.read_to_end(&mut buffer)?;
                zip.write_all(&*buffer)?;
                buffer.clear();
            } else {
                debug!("{} is a dir", path.display());
                zip.add_directory_from_path(Path::new(path), options)?;
            }
        }
    }

    zip.finish()?;

    debug!("switching back to {}", &initial_dir.display());
    env::set_current_dir(initial_dir)?;
    Ok(())
}

fn upload_pkg(cfg: &Cfg, tmp_dir: &TempDir) -> Result<()> {
    let form = multipart::Form::new().file("package", tmp_dir.path().join("pkg.zip"))?;
    let client = Client::new();
    let resp = client
        .post(&format!(
            "{}/crx/packmgr/service/.json?cmd=upload",
            cfg.instance.addr
        ))
        .header(
            "Authorization",
            format!(
                "Basic {}",
                encode(format!("{}:{}", cfg.instance.user, cfg.instance.pass))
            ),
        )
        .multipart(form)
        .send()?;
    debug!("upload pkg response: {:#?}", resp);
    Ok(())
}

fn build_pkg(cfg: &Cfg, pkg: &Pkg) -> Result<()> {
    let client = Client::new();
    let resp = client
        .post(&format!(
            "{}/crx/packmgr/service/script.html/etc/packages/{}?cmd=build",
            cfg.instance.addr,
            pkg.path(),
        ))
        .header(
            "Authorization",
            format!(
                "Basic {}",
                encode(format!("{}:{}", cfg.instance.user, cfg.instance.pass))
            ),
        )
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

fn unzip_pkg(tmp_dir: &TempDir) -> Result<()> {
    let res_zip_path = tmp_dir.path().join("res.zip");
    debug!("unzipping {}", res_zip_path.display());
    let mut archive = ZipArchive::new(File::open(res_zip_path)?)?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let outpath = file.sanitized_name();

        let outpath = tmp_dir.path().join(outpath);

        if file.is_dir() {
            debug!("extracting dir {}", outpath.display());
            create_dir_all(&outpath)?;
        } else {
            debug!("extracting file {}", outpath.display());
            if let Some(p) = outpath.parent() {
                if !p.exists() {
                    create_dir_all(&p)?;
                }
            }
            let mut outfile = File::create(&outpath)?;
            io::copy(&mut file, &mut outfile)?;
        }
    }

    Ok(())
}

fn cleanup_files(_tmp_dir: &TempDir) -> Result<()> {
    Ok(())
}

fn copy_files() -> Result<()> {
    Ok(())
}

fn init() -> Result<()> {
    debug!("initializing config file ./.je");
    let cfg = Cfg::default();
    let mut file = File::create(".je")?;
    file.write_all(toml::to_string(&cfg)?.as_bytes())?;
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    use anyhow::Result;
    use std::env;
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
}
