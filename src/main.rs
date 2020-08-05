use crate::cfg::Cfg;
use crate::cmd::Opt;
use crate::pkg::{mk_pkg_dir, Pkg};
use anyhow::Result;
use base64::encode;
use log::debug;
use reqwest::blocking::multipart;
use reqwest::blocking::Client;
use std::env;
use std::fs::{create_dir_all, remove_dir_all, File, OpenOptions};
use std::io;
use std::io::prelude::*;
use std::path::Path;
use std::thread;
use std::time::Duration;
use structopt::StructOpt;
use tempfile::TempDir;
use walkdir::WalkDir;
use zip::write::FileOptions;
use zip::ZipArchive;
use zip::ZipWriter;

mod cfg;
mod cmd;
mod pkg;

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

    for path in &["jcr_root", "./META-INF"] {
        let walkdir = WalkDir::new(path);
        let mut buffer = Vec::new();
        debug!("zipping {}", path);
        for entry in &mut walkdir.into_iter().flat_map(Result::ok) {
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
    use std::fs::read_to_string;
    use std::path::Path;
    use tempfile::TempDir;

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
