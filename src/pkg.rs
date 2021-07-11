use anyhow::Result;
use log::{debug, info};
use std::env;
use std::fs::{create_dir_all, File};
use std::io;
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use tempfile::TempDir;
use walkdir::WalkDir;
use zip::write::FileOptions;
use zip::ZipArchive;
use zip::ZipWriter;

trait TempDirExt {
    fn rel_path<A: AsRef<Path>>(&self, rel: A) -> PathBuf;
}

impl TempDirExt for TempDir {
    fn rel_path<A: AsRef<Path>>(&self, rel: A) -> PathBuf {
        self.path().join(rel.as_ref())
    }
}

pub(crate) fn zip_pkg(tmp_dir: &TempDir) -> Result<()> {
    let initial_dir = env::current_dir()?;
    info!(
        "zipping pkg under tmp directory {}",
        tmp_dir.path().display()
    );

    debug!(
        "switching dir from {} to {}",
        &initial_dir.display(),
        &tmp_dir.path().display()
    );
    env::set_current_dir(tmp_dir)?;

    let writer = File::create(tmp_dir.rel_path("pkg.zip"))?;
    let mut zip = ZipWriter::new(writer);
    let options = FileOptions::default();

    for path in &["jcr_root", "META-INF"] {
        let walkdir = WalkDir::new(path);
        let mut buffer = Vec::new();
        debug!("zipping {}", path);
        for entry in &mut walkdir.into_iter().filter_map(Result::ok) {
            let path = entry.path();
            if path.is_file() {
                debug!("{} is a file", path.display());
                zip.start_file(path.display().to_string(), options)?;
                let mut f = File::open(path)?;
                f.read_to_end(&mut buffer)?;
                zip.write_all(&*buffer)?;
                buffer.clear();
            } else {
                debug!("{} is a dir", path.display());
                zip.add_directory(path.display().to_string(), options)?;
            }
        }
    }

    zip.finish()?;

    debug!("switching back to {}", &initial_dir.display());
    env::set_current_dir(initial_dir)?;
    Ok(())
}

pub(crate) fn unzip_pkg(tmp_dir: &TempDir) -> Result<()> {
    let res_zip_path = tmp_dir.rel_path("res.zip");
    info!("unzipping {}", res_zip_path.display());
    let mut archive = ZipArchive::new(File::open(res_zip_path)?)?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let outpath = file.mangled_name();

        let outpath = tmp_dir.rel_path(outpath);

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

#[cfg(test)]
mod test {
    use super::*;
    use anyhow::Result;
    use maplit::hashset;
    use std::collections::HashSet;
    use std::fs::{create_dir, create_dir_all, rename, File};
    use std::path::{Path, PathBuf};
    use tempfile::TempDir;

    #[test]
    fn test_zip_pkg() -> Result<()> {
        let _ = pretty_env_logger::try_init();
        // given
        let tmp_dir = TempDir::new()?;
        create_dir(tmp_dir.rel_path("jcr_root"))?;
        File::create(jcr_root_file(&tmp_dir, "file1"))?;
        File::create(jcr_root_file(&tmp_dir, "file2"))?;
        File::create(jcr_root_file(&tmp_dir, "file3"))?;

        create_dir_all(tmp_dir.rel_path("META-INF/vault"))?;
        File::create(vault_file(&tmp_dir, "properties.xml"))?;
        File::create(vault_file(&tmp_dir, "filter.xml"))?;

        // when
        zip_pkg(&tmp_dir)?;

        // then
        assert!(Path::new(&tmp_dir.rel_path("pkg.zip")).exists());
        let archive_files = archive_files_list(&tmp_dir.rel_path("pkg.zip"))?;

        assert_eq!(
            archive_files,
            hashset! {
                "jcr_root/".into(),
                "jcr_root/file2".into(),
                "jcr_root/file1".into(),
                "jcr_root/file3".into(),
                "META-INF/".into(),
                "META-INF/vault/".into(),
                "META-INF/vault/properties.xml".into(),
                "META-INF/vault/filter.xml".into()
            }
        );
        Ok(())
    }

    fn jcr_root_file<S: Into<String>>(dir: &TempDir, name: S) -> PathBuf {
        let path = format!("jcr_root/{}", name.into());
        Path::new(&dir.rel_path(path)).to_path_buf()
    }

    fn vault_file<S: Into<String>>(dir: &TempDir, name: S) -> PathBuf {
        let path = format!("META-INF/vault/{}", name.into());
        Path::new(&dir.rel_path(path)).to_path_buf()
    }

    fn archive_files_list<A: AsRef<Path>>(path: A) -> Result<HashSet<String>> {
        let mut archive = ZipArchive::new(File::open(path)?)?;

        let mut archive_files = HashSet::new();
        for i in 0..archive.len() {
            let file = archive.by_index(i)?;
            archive_files.insert(file.name().to_string());
        }
        Ok(archive_files)
    }

    #[test]
    fn test_unzip_pkg() -> Result<()> {
        let _ = pretty_env_logger::try_init();
        // given
        let tmp_dir = TempDir::new()?;
        create_dir(tmp_dir.rel_path("jcr_root"))?;
        File::create(jcr_root_file(&tmp_dir, "file1"))?;
        File::create(jcr_root_file(&tmp_dir, "file2"))?;
        File::create(jcr_root_file(&tmp_dir, "file3"))?;

        create_dir_all(tmp_dir.rel_path("META-INF/vault"))?;
        File::create(vault_file(&tmp_dir, "properties.xml"))?;
        File::create(vault_file(&tmp_dir, "filter.xml"))?;

        zip(&tmp_dir)?;

        let target_dir = TempDir::new()?;
        rename(
            Path::new(&tmp_dir.rel_path("pkg.zip")),
            Path::new(&target_dir.rel_path("res.zip")),
        )?;

        // when
        unzip_pkg(&target_dir)?;

        // then
        assert!(jcr_root_file(&target_dir, "file1").exists());
        assert!(jcr_root_file(&target_dir, "file2").exists());
        assert!(jcr_root_file(&target_dir, "file2").exists());

        assert!(vault_file(&target_dir, "properties.xml").exists());
        assert!(vault_file(&target_dir, "filter.xml").exists());
        Ok(())
    }

    fn zip(tmp_dir: &TempDir) -> Result<()> {
        let initial_dir = env::current_dir()?;
        env::set_current_dir(tmp_dir)?;

        let writer = File::create(tmp_dir.rel_path("pkg.zip"))?;
        let mut zip = ZipWriter::new(writer);
        let options = FileOptions::default();

        for path in &["jcr_root", "META-INF"] {
            let walkdir = WalkDir::new(path);
            let mut buffer = Vec::new();
            debug!("zipping {}", path);
            for entry in &mut walkdir.into_iter().filter_map(Result::ok) {
                let path = entry.path();
                if path.is_file() {
                    debug!("{} is a file", path.display());
                    zip.start_file(path.display().to_string(), options)?;
                    let mut f = File::open(path)?;
                    f.read_to_end(&mut buffer)?;
                    zip.write_all(&*buffer)?;
                    buffer.clear();
                } else {
                    debug!("{} is a dir", path.display());
                    zip.add_directory(path.display().to_string(), options)?;
                }
            }
        }
        zip.finish()?;
        env::set_current_dir(initial_dir)?;
        Ok(())
    }
}
