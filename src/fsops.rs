use crate::cfg::Cfg;
use crate::path::Path;
use anyhow::Result;
use log::{debug, info};
use std::fs::{self, File, OpenOptions};
use std::io::prelude::*;
use std::io::BufReader;
use std::path::Path as OsPath;
use tempfile::TempDir;
use walkdir::{DirEntry, WalkDir};

pub(crate) fn cleanup_files(cfg: &Cfg, tmp_dir: &TempDir) -> Result<()> {
    info!("cleaning files from unwanted properties");
    for entry in WalkDir::new(tmp_dir.path().join("jcr_root"))
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| is_xml_file(e))
    {
        debug!("cleaning file {}", entry.path().display());
        let mut file = File::open(entry.path())?;
        let reader = BufReader::new(&mut file);
        let lines: Vec<_> = reader
            .lines()
            .map(|l| l.expect("could not read line"))
            .filter_map(|l| allowed_prop(l, &cfg.ignore_properties))
            .collect();

        let mut file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(entry.path())?;
        debug!("writing cleaned lines back to file");
        for line in lines {
            file.write_all(line.as_bytes())?;
        }
    }

    Ok(())
}

fn is_xml_file(e: &DirEntry) -> bool {
    is_file(e) && is_xml(e)
}

fn is_file(e: &DirEntry) -> bool {
    let is_file = e.path().is_file();
    debug!(
        "{} {} a file",
        e.path().display(),
        if is_file { "is" } else { "is not" }
    );
    is_file
}

fn is_xml(e: &DirEntry) -> bool {
    let is_xml = e.path().ends_with(".content.xml");
    debug!(
        "{} {} xml file",
        e.path().display(),
        if is_xml { "is" } else { "is not" }
    );
    is_xml
}

fn allowed_prop<S: Into<String>>(line: S, ignore_properties: &[String]) -> Option<String> {
    let line = line.into();
    let mut result = true;
    for ignore_prop in ignore_properties {
        debug!("checking if {} contains {}", line, ignore_prop);
        if line.contains(ignore_prop) {
            debug!("line {} contains not allowed property, removing", line);
            result = false;
            break;
        }
    }
    if result {
        Some(format!("{}\n", line))
    } else {
        None
    }
}

pub(crate) fn mv_files_back(tmp_dir: &TempDir, path: &Path) -> Result<()> {
    let from = tmp_dir.path().join(path.from_root());
    info!("moving files from {} to {}", from.display(), path.full());
    list_files(&from);
    if path.is_dir() {
        fs::remove_dir_all(path.full())?;
    }
    fs::rename(from, path.full())?;
    Ok(())
}

fn list_files<P: AsRef<OsPath>>(path: P) {
    debug!("files under {}:", path.as_ref().display());
    for entry in WalkDir::new(path).into_iter().filter_map(Result::ok) {
        debug!("\t- {}", entry.path().display());
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_allowed_prop() {
        // given
        let ignore_properties = &["cq:lastModified".to_string(), "testProperty".to_string()];
        let test_cases = &[
            ("cq:lastModified", None),
            ("testProperty", None),
            ("cq:lastModifiedBy", None),
            ("    cq:lastModifiedBy", None),
            ("cq:lastReplicated", Some("cq:lastReplicated\n".to_string())),
            (
                "    cq:lastReplicated",
                Some("    cq:lastReplicated\n".to_string()),
            ),
        ];

        for test_case in test_cases {
            // when
            let result = allowed_prop(test_case.0, ignore_properties);

            // then
            assert_eq!(result, test_case.1);
        }
    }
}
