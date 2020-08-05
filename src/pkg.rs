use anyhow::Result;
use log::debug;
use std::env;
use std::fs::{create_dir_all, File};
use std::io;
use std::io::prelude::*;
use std::path::Path;
use tempfile::TempDir;
use walkdir::WalkDir;
use zip::write::FileOptions;
use zip::ZipArchive;
use zip::ZipWriter;

pub(crate) fn zip_pkg(tmp_dir: &TempDir) -> Result<()> {
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

pub(crate) fn unzip_pkg(tmp_dir: &TempDir) -> Result<()> {
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

#[cfg(test)]
mod test {
    use super::*;
    use anyhow::Result;
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
}
