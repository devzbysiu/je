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
    use anyhow::Result;
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
            spy.post_file_reqs(),
            &[(
                "/crx/packmgr/service/.json?cmd=upload".into(),
                to_string(&pkg_path)
            )]
        );
        Ok(())
    }

    fn to_string<A: AsRef<Path>>(path: A) -> String {
        path.as_ref().display().to_string()
    }

    struct ClientSpy {
        post_file_reqs: RefCell<Vec<(String, String)>>,
    }

    impl ClientSpy {
        fn new() -> Self {
            Self {
                post_file_reqs: RefCell::new(Vec::new()),
            }
        }

        fn post_file_reqs(&self) -> Vec<(String, String)> {
            self.post_file_reqs.take()
        }
    }

    impl Client for ClientSpy {
        fn post_file<S: Into<String>, A: AsRef<Path>>(
            &self,
            path: S,
            filepath: A,
        ) -> Result<Response> {
            self.post_file_reqs
                .borrow_mut()
                .push((path.into(), to_string(filepath)));
            Ok(Response(None))
        }

        fn post<S: Into<String>>(&self, path: S) -> Result<Response> {
            Ok(Response(None))
        }

        fn get<S: Into<String>>(&self, path: S) -> Result<Response> {
            Ok(Response(None))
        }
    }
}
