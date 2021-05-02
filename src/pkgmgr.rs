use crate::http::Client;
use crate::pkgdir;
use anyhow::Result;
use log::{debug, info};
use std::fs::File;
use std::io::prelude::*;
use tempfile::TempDir;

pub(crate) fn upload_pkg(client: &impl Client, dir: &TempDir) -> Result<()> {
    let resp = client.post_file(
        "/crx/packmgr/service/.json?cmd=upload",
        dir.path().join("pkg.zip"),
    )?;
    debug!("upload pkg response: {:#?}", resp);
    Ok(())
}

pub(crate) fn build_pkg(client: &impl Client, pkg: &pkgdir::Pkg) -> Result<()> {
    let resp = client.post(&format!(
        "/crx/packmgr/service/.json/etc/packages/{}?cmd=build",
        pkg.path(),
    ))?;
    debug!("build pkg response: {:#?}", resp);
    Ok(())
}

pub(crate) fn download_pkg(client: &impl Client, dir: &TempDir, pkg: &pkgdir::Pkg) -> Result<()> {
    info!("downloading pkg");
    let resp = client.get(&format!("/etc/packages/{}", pkg.path(),))?;
    debug!("download pkg response: {:#?}", resp);
    let mut pkg_file = File::create(dir.path().join("res.zip"))?;
    pkg_file.write_all(&resp.bytes()?)?;
    Ok(())
}

pub(crate) fn install_pkg(client: &impl Client, pkg: &pkgdir::Pkg) -> Result<()> {
    let resp = client.post(&format!(
        "/crx/packmgr/service/.json/etc/packages/{}?cmd=install",
        pkg.path()
    ))?;
    debug!("install pkg response: {:#?}", resp);
    Ok(())
}

pub(crate) fn delete_pkg(client: &impl Client, debug: bool, pkg: &pkgdir::Pkg) -> Result<()> {
    if debug {
        info!("package deletion omitted because of passed flag");
        return Ok(());
    }
    let resp = client.post(&format!(
        "/crx/packmgr/service/.json/etc/packages/{}?cmd=delete",
        pkg.path()
    ))?;
    debug!("delete pkg response: {:#?}", resp);
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::http::Response;
    use crate::pkgdir::Pkg;
    use anyhow::Result;
    use regex::Regex;
    use std::cell::RefCell;
    use std::path::Path;

    #[test]
    fn test_upload_pkg() -> Result<()> {
        // given
        let spy = ClientSpy::new();
        let dir = TempDir::new()?;
        let pkg_path = dir.path().join("pkg.zip");

        // when
        upload_pkg(&spy, &dir)?;

        // then
        assert_eq!(
            spy.post_file_req(),
            (
                "/crx/packmgr/service/.json?cmd=upload".into(),
                to_string(&pkg_path)
            )
        );
        Ok(())
    }

    #[test]
    fn test_build_pkg() -> Result<()> {
        // given
        let spy = ClientSpy::new();
        let pkg = Pkg::default();
        let req_regex =
            Regex::new(r"/crx/packmgr/service/\.json/etc/packages/je/je-pkg-\d+\.zip\?cmd=build")?;

        // when
        build_pkg(&spy, &pkg)?;

        // then
        assert!(req_regex.is_match(&spy.post_req()));
        Ok(())
    }

    #[test]
    fn test_download_pkg() -> Result<()> {
        // given
        let spy = ClientSpy::new();
        let pkg = Pkg::default();
        let dir = TempDir::new()?;
        let req_regex = Regex::new(r"/etc/packages/je/je-pkg-\d+\.zip")?;

        // when
        download_pkg(&spy, &dir, &pkg)?;

        // then
        println!("req: {}", spy.get_req());
        assert!(req_regex.is_match(&spy.get_req()));

        Ok(())
    }

    #[test]
    fn test_install_pkg() -> Result<()> {
        // given
        let spy = ClientSpy::new();
        let pkg = Pkg::default();
        let req_regex = Regex::new(
            r"/crx/packmgr/service/\.json/etc/packages/je/je-pkg-\d+\.zip\?cmd=install",
        )?;

        // when
        install_pkg(&spy, &pkg)?;

        // then
        assert!(req_regex.is_match(&spy.post_req()));
        Ok(())
    }

    #[test]
    fn test_delete_when_deletion_turned_on_pkg() -> Result<()> {
        // given
        let spy = ClientSpy::new();
        let pkg = Pkg::default();
        let req_regex =
            Regex::new(r"/crx/packmgr/service/\.json/etc/packages/je/je-pkg-\d+\.zip\?cmd=delete")?;

        // when
        delete_pkg(&spy, false, &pkg)?;

        // then
        assert!(req_regex.is_match(&spy.post_req()));
        Ok(())
    }

    #[test]
    fn test_delete_when_deletion_skipped() -> Result<()> {
        // given
        let spy = ClientSpy::new();
        let pkg = Pkg::default();

        // when
        delete_pkg(&spy, true, &pkg)?;

        // then
        assert_eq!(spy.post_req(), "");
        Ok(())
    }

    fn to_string<A: AsRef<Path>>(path: A) -> String {
        path.as_ref().display().to_string()
    }

    struct ClientSpy {
        post_file_req: RefCell<(String, String)>,
        post_req: RefCell<String>,
        get_req: RefCell<String>,
    }

    impl ClientSpy {
        fn new() -> Self {
            Self {
                post_file_req: RefCell::new((String::new(), String::new())),
                post_req: RefCell::new(String::new()),
                get_req: RefCell::new(String::new()),
            }
        }

        fn post_file_req(&self) -> (String, String) {
            self.post_file_req.clone().take()
        }

        fn post_req(&self) -> String {
            self.post_req.clone().take()
        }

        fn get_req(&self) -> String {
            self.get_req.clone().take()
        }
    }

    impl Client for ClientSpy {
        fn post_file<S: Into<String>, A: AsRef<Path>>(
            &self,
            path: S,
            filepath: A,
        ) -> Result<Response> {
            *self.post_file_req.borrow_mut() = (path.into(), to_string(filepath));
            Ok(Response(None))
        }

        fn post<S: Into<String>>(&self, path: S) -> Result<Response> {
            *self.post_req.borrow_mut() = path.into();
            Ok(Response(None))
        }

        fn get<S: Into<String>>(&self, path: S) -> Result<Response> {
            *self.get_req.borrow_mut() = path.into();
            Ok(Response(None))
        }
    }
}
