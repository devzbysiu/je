use crate::http::AemClient;
use crate::pkgdir;
use anyhow::Result;
use log::{debug, info};
use std::fs::File;
use std::io::prelude::*;
use tempfile::TempDir;

pub(crate) fn upload_pkg(client: &AemClient, dir: &TempDir) -> Result<()> {
    let resp = client.post_file(
        "/crx/packmgr/service/.json?cmd=upload",
        dir.path().join("pkg.zip"),
    )?;
    debug!("upload pkg response: {:#?}", resp);
    Ok(())
}

pub(crate) fn build_pkg(client: &AemClient, pkg: &pkgdir::Pkg) -> Result<()> {
    let resp = client.post(&format!(
        "/crx/packmgr/service/.json/etc/packages/{}?cmd=build",
        pkg.path(),
    ))?;
    debug!("build pkg response: {:#?}", resp);
    Ok(())
}

pub(crate) fn download_pkg(client: &AemClient, dir: &TempDir, pkg: &pkgdir::Pkg) -> Result<()> {
    info!("downloading pkg");
    let resp = client.get(&format!("/etc/packages/{}", pkg.path(),))?;
    debug!("download pkg response: {:#?}", resp);
    let mut pkg_file = File::create(dir.path().join("res.zip"))?;
    pkg_file.write_all(&resp.bytes()?)?;
    Ok(())
}

pub(crate) fn install_pkg(client: &AemClient, pkg: &pkgdir::Pkg) -> Result<()> {
    let resp = client.post(&format!(
        "/crx/packmgr/service/.json/etc/packages/{}?cmd=install",
        pkg.path()
    ))?;
    debug!("install pkg response: {:#?}", resp);
    Ok(())
}

pub(crate) fn delete_pkg(client: &AemClient, debug: bool, pkg: &pkgdir::Pkg) -> Result<()> {
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
