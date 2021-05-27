use anyhow::Result;
use assert_cmd::cmd::Command;
use log::debug;
use mock::AemMock;
use std::env;
use std::fs::create_dir_all;
use std::fs::{read_to_string, File};
use std::io::prelude::*;
use std::path::Path as OsPath;
use tempfile::TempDir;
use walkdir::WalkDir;

#[tokio::test]
async fn test_mock() -> Result<()> {
    pretty_env_logger::init();
    let tmp_dir = TempDir::new()?;

    let content_dir = tmp_dir.path().join("initial-content/jcr_root/content");
    create_dir_all(&content_dir)?;
    let mut file = File::create(content_dir.join("test-file"))?;
    file.write_all("initial-content".as_bytes())?;

    let mock = AemMock::new(&tmp_dir);
    mock.start()?;
    std::thread::sleep(std::time::Duration::from_secs(3));

    env::set_current_dir(tmp_dir.path())?;
    let mut cfg = File::create(".je")?;
    let cfg_content = r#"ignore_properties = []

    [[profile]]
    name = "author"
    addr = "http://localhost:7788"
    user = "admin"
    pass = "admin"
    "#;
    cfg.write_all(cfg_content.as_bytes())?;

    let file = read_to_string(content_dir.join("test-file"))?;
    assert_eq!(file, "initial-content");

    let output = Command::cargo_bin("je")?
        .arg("get")
        .arg(content_dir.as_path().to_string_lossy().to_string())
        .assert()
        .success();

    debug!("output: {:?}", output);
    list_files(&content_dir);

    let file = read_to_string(content_dir.join("test-file"))?;
    assert_eq!(file, "zip-content");

    Ok(())
}

fn list_files<P: AsRef<OsPath>>(path: P) {
    debug!("files under {}:", path.as_ref().display());
    for entry in WalkDir::new(path).into_iter().filter_map(Result::ok) {
        debug!("\t- {}", entry.path().display());
    }
}

#[cfg(test)]
mod mock {
    use anyhow::Result;
    use log::{debug, warn};
    use reqwest::blocking::Client;
    use rocket::form::{Form, FromForm};
    use rocket::http::Status;
    use rocket::{get, post, routes, Build, Config, Rocket, Shutdown, State};
    use std::fs::create_dir_all;
    use std::fs::{copy, read_to_string, File};
    use std::io::prelude::*;
    use std::net::{IpAddr, Ipv4Addr};
    use tempfile::TempDir;
    use tokio::runtime::Runtime;
    use walkdir::WalkDir;
    use zip::write::FileOptions;
    use zip::ZipWriter;

    pub(crate) struct AemMock<'a> {
        tmp_dir: &'a TempDir,
    }

    impl<'a> AemMock<'a> {
        pub(crate) fn new(tmp_dir: &'a TempDir) -> Self {
            Self { tmp_dir }
        }

        pub(crate) fn start(&self) -> Result<()> {
            // let tmp_dir = self.tmp_dir;

            let content_dir = self.tmp_dir.path().join("server-zip/jcr_root/content");
            create_dir_all(&content_dir)?;
            let mut file = File::create(content_dir.join("test-file"))?;
            file.write_all("zip-content".as_bytes())?;

            debug!(
                "zip file content#####################################: {}",
                read_to_string(content_dir.join("test-file"))?
            );

            let writer = File::create(self.tmp_dir.path().join("server-zip/pkg.zip"))?;
            let options = FileOptions::default();
            let mut zip = ZipWriter::new(writer);

            let walkdir = WalkDir::new(self.tmp_dir.path().join("server-zip/jcr_root"));
            let mut buffer = Vec::new();
            for entry in &mut walkdir.into_iter().flat_map(Result::ok) {
                let path = entry.path();
                let short_path = path.strip_prefix(self.tmp_dir.path().join("server-zip"))?;
                if path.is_file() {
                    zip.start_file(short_path.display().to_string(), options)?;
                    let mut f = File::open(path)?;
                    f.read_to_end(&mut buffer)?;
                    zip.write_all(&*buffer)?;
                    buffer.clear();
                } else {
                    zip.add_directory(short_path.display().to_string(), options)?;
                }
            }

            zip.finish()?;
            copy(
                self.tmp_dir.path().join("server-zip/pkg.zip"),
                "/tmp/testzup.zip",
            )?;

            let tmp_dir_path = self.tmp_dir.path().to_string_lossy().to_string();
            std::thread::spawn(|| -> Result<()> {
                let rt = Runtime::new()?;
                rt.block_on(rocket(tmp_dir_path).launch())?;
                Ok(())
            });
            Ok(())
        }
    }

    fn rocket(tmp_dir_path: String) -> Rocket<Build> {
        let config = Config {
            address: IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
            port: 7788,
            ..Config::default()
        };
        rocket::custom(config).manage(tmp_dir_path).mount(
            "/",
            routes![upload_pkg, handle_pkg, download_pkg, shutdown_endpoint],
        )
    }

    #[post("/crx/packmgr/service/.json?cmd=upload", data = "<package>")]
    fn upload_pkg(package: Form<Package>) -> Status {
        debug!("package data: {:?}", package);
        Status::Ok
    }

    #[post("/crx/packmgr/service/.json/etc/packages/je/<name>?<cmd>")]
    fn handle_pkg(name: String, cmd: String) -> Status {
        match cmd.as_str() {
            "build" => build_pkg(),
            "install" => install_pkg(),
            "delete" => delete_pkg(),
            other => {
                warn!("not supported operation: {}", other);
                Status::NotImplemented
            }
        }
    }

    #[get("/etc/packages/je/<name>")]
    fn download_pkg(name: String, path: &State<String>) -> Vec<u8> {
        debug!("file zip {}{}", path, "/server-zip/pkg.zip");
        let mut file = File::open(format!("{}{}", path, "/server-zip/pkg.zip")).unwrap();
        let mut buf: Vec<u8> = Vec::new();
        file.read_to_end(&mut buf).unwrap();
        buf
    }

    fn build_pkg() -> Status {
        debug!("building pkg");
        Status::Ok
    }

    fn install_pkg() -> Status {
        debug!("installing pkg");
        Status::Ok
    }

    fn delete_pkg() -> Status {
        debug!("deleting pkg");
        Status::Ok
    }

    #[post("/mock/shutdown")]
    fn shutdown_endpoint(shutdown: Shutdown) {
        shutdown.notify();
    }

    pub(crate) fn stop() -> Result<()> {
        let client = Client::new();
        client.post("http://127.0.0.1:7788/mock/shutdown").send()?;
        Ok(())
    }

    #[derive(FromForm, Debug)]
    struct Package {
        data: Vec<u8>,
    }
}
