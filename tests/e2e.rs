use anyhow::Result;
use assert_cmd::cmd::Command;
use mock::AemMock;
use setup_fs::setup_fs;
use std::env;
use std::fs::{read_to_string, File};
use std::io::prelude::*;
use tempfile::TempDir;
use walkdir::WalkDir;
use zip::write::FileOptions;
use zip::ZipWriter;

#[tokio::test]
async fn test_mock() -> Result<()> {
    pretty_env_logger::init();
    // given
    let tmp_dir = TempDir::new()?;
    setup_fs(
        tmp_dir.path(),
        r#"
        |_initial-content
        | |_jcr_root
        |   |_content
        |     |_test-file
        |       "initial-content"
        |_server-zip
          |_jcr_root
            |_content
              |_test-file
                "zip-content"
    "#,
    )?;
    start_mock(&tmp_dir)?;
    setup_je_config(&tmp_dir)?;

    let content_dir = tmp_dir.path().join("initial-content/jcr_root/content");
    let file = read_to_string(content_dir.join("test-file"))?;
    assert_eq!(file, "initial-content");

    // when
    Command::cargo_bin("je")?
        .arg("get")
        .arg(content_dir.as_path().to_string_lossy().to_string())
        .assert()
        .success();

    // then
    assert_eq!(
        read_to_string(content_dir.join("test-file"))?,
        "zip-content"
    );

    Ok(())
}

fn start_mock(tmp_dir: &TempDir) -> Result<()> {
    let buf = create_pkg_zip(&tmp_dir)?;
    let mock = AemMock::new(buf);
    mock.start()?;
    std::thread::sleep(std::time::Duration::from_secs(3));
    Ok(())
}

fn create_pkg_zip(tmp_dir: &TempDir) -> Result<Vec<u8>> {
    let writer = File::create(tmp_dir.path().join("server-zip/pkg.zip"))?;
    let options = FileOptions::default();
    let mut zip = ZipWriter::new(writer);

    let walkdir = WalkDir::new(tmp_dir.path().join("server-zip/jcr_root"));
    let mut buffer = Vec::new();
    for entry in &mut walkdir.into_iter().flat_map(Result::ok) {
        let path = entry.path();
        let short_path = path.strip_prefix(tmp_dir.path().join("server-zip"))?;
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

    let mut file = File::open(tmp_dir.path().join("server-zip/pkg.zip")).unwrap();
    let mut buf: Vec<u8> = Vec::new();
    file.read_to_end(&mut buf)?;

    Ok(buf)
}

fn setup_je_config(tmp_dir: &TempDir) -> Result<()> {
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
    Ok(())
}

#[cfg(test)]
mod mock {
    use anyhow::Result;
    use log::{debug, warn};
    use reqwest::blocking::Client;
    use rocket::form::{Form, FromForm};
    use rocket::http::Status;
    use rocket::{get, post, routes, Build, Config, Rocket, Shutdown, State};
    use std::net::{IpAddr, Ipv4Addr};
    use tokio::runtime::Runtime;

    pub(crate) struct AemMock {
        zip: Vec<u8>,
    }

    impl AemMock {
        pub(crate) fn new(zip: Vec<u8>) -> Self {
            Self { zip }
        }

        pub(crate) fn start(&self) -> Result<()> {
            let zip = self.zip.clone();
            std::thread::spawn(|| -> Result<()> {
                let rt = Runtime::new()?;
                rt.block_on(rocket(zip).launch())?;
                Ok(())
            });
            Ok(())
        }
    }

    fn rocket(zip: Vec<u8>) -> Rocket<Build> {
        let config = Config {
            address: IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
            port: 7788,
            ..Config::default()
        };
        rocket::custom(config).manage(zip).mount(
            "/",
            routes![upload_pkg, handle_pkg, download_pkg, shutdown_endpoint],
        )
    }

    #[post("/crx/packmgr/service/.json?cmd=upload", data = "<package>")]
    fn upload_pkg(package: Form<Package>) -> Status {
        debug!("package data: {:?}", package);
        Status::Ok
    }

    #[post("/crx/packmgr/service/.json/etc/packages/je/<_name>?<cmd>")]
    fn handle_pkg(_name: String, cmd: String) -> Status {
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

    #[get("/etc/packages/je/<_name>")]
    fn download_pkg(_name: String, zip: &State<Vec<u8>>) -> Vec<u8> {
        debug!("responding with zip");
        zip.to_vec()
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
