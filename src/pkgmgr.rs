use crate::cfg::Instance;
use crate::pkgdir;
use anyhow::Result;
use base64::encode;
use log::{debug, info};
use reqwest::blocking::multipart;
use reqwest::blocking::Client;
use std::fs::File;
use std::io::prelude::*;
use tempfile::TempDir;

pub(crate) fn upload_pkg(instance: &Instance, tmp_dir: &TempDir) -> Result<()> {
    info!("uploading pkg to instance: {}", instance.addr());
    let form = multipart::Form::new().file("package", tmp_dir.path().join("pkg.zip"))?;
    let client = Client::new();
    let resp = client
        .post(&format!(
            "{}/crx/packmgr/service/.json?cmd=upload",
            instance.addr()
        ))
        .header(
            "Authorization",
            format!(
                "Basic {}",
                encode(format!("{}:{}", instance.user(), instance.pass()))
            ),
        )
        .multipart(form)
        .send()?;
    debug!("upload pkg response: {:#?}", resp);
    Ok(())
}

pub(crate) fn build_pkg(instance: &Instance, pkg: &pkgdir::Pkg) -> Result<()> {
    info!(
        "building pkg with path {} on instance {}",
        pkg.path(),
        instance.addr()
    );
    let client = Client::new();
    let resp = client
        .post(&format!(
            "{}/crx/packmgr/service/.json/etc/packages/{}?cmd=build",
            instance.addr(),
            pkg.path(),
        ))
        .header(
            "Authorization",
            format!(
                "Basic {}",
                encode(format!("{}:{}", instance.user(), instance.pass()))
            ),
        )
        .send()?;
    debug!("build pkg response: {:#?}", resp);
    Ok(())
}

pub(crate) fn download_pkg(
    instance: &Instance,
    tmp_dir: &TempDir,
    pkg: &pkgdir::Pkg,
) -> Result<()> {
    info!("downloading pkg");
    let client = Client::new();
    let resp = client
        .get(&format!("{}/etc/packages/{}", instance.addr(), pkg.path(),))
        .header("Authorization", format!("Basic {}", encode("admin:admin")))
        .send()?;
    debug!("download pkg response: {:#?}", resp);
    let mut pkg_file = File::create(tmp_dir.path().join("res.zip"))?;
    pkg_file.write_all(&resp.bytes()?)?;
    Ok(())
}

pub(crate) fn install_pkg(instance: &Instance, pkg: &pkgdir::Pkg) -> Result<()> {
    info!(
        "installing pkg {} on instance: {}",
        pkg.path(),
        instance.addr()
    );
    let client = Client::new();
    let resp = client
        .post(&format!(
            "{}/crx/packmgr/service/.json/etc/packages/{}?cmd=install",
            instance.addr(),
            pkg.path()
        ))
        .header(
            "Authorization",
            format!(
                "Basic {}",
                encode(format!("{}:{}", instance.user(), instance.pass()))
            ),
        )
        .send()?;
    debug!("install pkg response: {:#?}", resp);
    Ok(())
}

pub(crate) fn delete_pkg(debug: bool, instance: &Instance, pkg: &pkgdir::Pkg) -> Result<()> {
    if debug {
        info!("package deletion omitted because of passed flag");
    } else {
        info!(
            "deleting pkg {} on instance: {}",
            pkg.path(),
            instance.addr()
        );
        let client = Client::new();
        let resp = client
            .post(&format!(
                "{}/crx/packmgr/service/.json/etc/packages/{}?cmd=delete",
                instance.addr(),
                pkg.path()
            ))
            .header(
                "Authorization",
                format!(
                    "Basic {}",
                    encode(format!("{}:{}", instance.user(), instance.pass()))
                ),
            )
            .send()?;
        debug!("delete pkg response: {:#?}", resp);
    }
    Ok(())
}
