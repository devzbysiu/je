use crate::cfg::Cfg;
use crate::pkgdir;
use anyhow::Result;
use base64::encode;
use log::{debug, info};
use reqwest::blocking::multipart;
use reqwest::blocking::Client;
use std::fs::File;
use std::io::prelude::*;
use tempfile::TempDir;

pub(crate) fn upload_pkg(cfg: &Cfg, tmp_dir: &TempDir) -> Result<()> {
    info!("uploading pkg to instance: {}", cfg.instance.addr);
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

pub(crate) fn build_pkg(cfg: &Cfg, pkg: &pkgdir::Pkg) -> Result<()> {
    info!(
        "building pkg with path {} on instance {}",
        pkg.path(),
        cfg.instance.addr
    );
    let client = Client::new();
    let resp = client
        .post(&format!(
            "{}/crx/packmgr/service/.json/etc/packages/{}?cmd=build",
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

pub(crate) fn download_pkg(cfg: &Cfg, tmp_dir: &TempDir, pkg: &pkgdir::Pkg) -> Result<()> {
    info!("downloading pkg");
    let client = Client::new();
    let resp = client
        .get(&format!(
            "{}/etc/packages/{}",
            cfg.instance.addr,
            pkg.path(),
        ))
        .header("Authorization", format!("Basic {}", encode("admin:admin")))
        .send()?;
    debug!("download pkg response: {:#?}", resp);
    let mut pkg_file = File::create(tmp_dir.path().join("res.zip"))?;
    pkg_file.write_all(&resp.bytes()?)?;
    Ok(())
}

pub(crate) fn install_pkg(cfg: &Cfg, pkg: &pkgdir::Pkg) -> Result<()> {
    info!(
        "installing pkg {} on instance: {}",
        pkg.path(),
        cfg.instance.addr
    );
    let client = Client::new();
    let resp = client
        .post(&format!(
            "{}/crx/packmgr/service/.json/etc/packages/{}?cmd=install",
            cfg.instance.addr,
            pkg.path()
        ))
        .header(
            "Authorization",
            format!(
                "Basic {}",
                encode(format!("{}:{}", cfg.instance.user, cfg.instance.pass))
            ),
        )
        .send()?;
    debug!("install pkg response: {:#?}", resp);
    Ok(())
}

pub(crate) fn delete_pkg(debug: bool, cfg: &Cfg, pkg: &pkgdir::Pkg) -> Result<()> {
    if debug {
        info!("package deletion omitted because of passed flag");
    } else {
        info!(
            "deleting pkg {} on instance: {}",
            pkg.path(),
            cfg.instance.addr
        );
        let client = Client::new();
        let resp = client
            .post(&format!(
                "{}/crx/packmgr/service/.json/etc/packages/{}?cmd=delete",
                cfg.instance.addr,
                pkg.path()
            ))
            .header(
                "Authorization",
                format!(
                    "Basic {}",
                    encode(format!("{}:{}", cfg.instance.user, cfg.instance.pass))
                ),
            )
            .send()?;
        debug!("delete pkg response: {:#?}", resp);
    }
    Ok(())
}
