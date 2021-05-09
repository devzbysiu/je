use crate::cfg::{Bundle, IgnoreProp, IgnoreType};
use crate::path::Path;
use anyhow::Result;
use log::{debug, info, warn};
use regex::Regex;
use std::fs::{self, File, OpenOptions};
use std::io::prelude::*;
use std::io::BufReader;
use std::path::Path as OsPath;
use std::path::PathBuf;
use tempfile::TempDir;
use walkdir::{DirEntry, WalkDir};

struct Entry {
    path: PathBuf,
    direntry: Option<DirEntry>,
}

impl Entry {
    fn is_xml_file(&self) -> bool {
        self.is_file() && self.is_content_xml()
    }

    fn is_file(&self) -> bool {
        let is_file = self.path().is_file();
        debug!(
            "{} {} a file",
            self.path().display(),
            if is_file { "is" } else { "is not" }
        );
        is_file
    }

    fn is_content_xml(&self) -> bool {
        let is_xml = self.path().ends_with(".content.xml");
        debug!(
            "{} {} xml file",
            self.path().display(),
            if is_xml { "is" } else { "is not" }
        );
        is_xml
    }

    fn path(&self) -> &OsPath {
        match self.direntry {
            Some(ref e) => e.path(),
            None => self.path.as_ref(),
        }
    }
}

impl From<&OsPath> for Entry {
    fn from(path: &OsPath) -> Self {
        Self {
            path: PathBuf::from(path),
            direntry: None,
        }
    }
}

impl From<DirEntry> for Entry {
    fn from(direntry: DirEntry) -> Self {
        Self {
            path: direntry.path().into(),
            direntry: Some(direntry),
        }
    }
}

pub(crate) fn cleanup_files(ignore_properties: &[IgnoreProp], tmp_dir: &TempDir) -> Result<()> {
    info!("cleaning files from unwanted properties");
    for entry in WalkDir::new(tmp_dir.path().join("jcr_root"))
        .into_iter()
        .filter_map(to_entry)
        .filter(Entry::is_xml_file)
    {
        debug!("cleaning file {}", entry.path().display());
        let mut file = File::open(entry.path())?;
        let reader = BufReader::new(&mut file);
        let lines: Vec<_> = reader
            .lines()
            .map(|l| l.expect("could not read line"))
            .filter_map(|l| allowed_prop(l, ignore_properties))
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

fn to_entry(result: std::result::Result<DirEntry, walkdir::Error>) -> Option<Entry> {
    match result.ok() {
        Some(e) => Some(Entry::from(e)),
        None => None,
    }
}

fn allowed_prop<S: Into<String>>(line: S, ignore_properties: &[IgnoreProp]) -> Option<String> {
    let line = line.into();
    let mut result = true;
    for ignore_prop in ignore_properties {
        match ignore_prop.ignore_type {
            IgnoreType::Contains => {
                debug!("checking if '{}' contains '{}'", line, ignore_prop.value);
                if line.contains(&ignore_prop.value) {
                    debug!("line '{}' contains not allowed property, removing", line);
                    result = false;
                    break;
                }
            }
            IgnoreType::Regex => {
                debug!(
                    "checking if '{}' matches regex '{}'",
                    line, ignore_prop.value
                );
                let regex = Regex::new(&ignore_prop.value);
                if let Err(e) = regex {
                    warn!(
                        "regex '{}' is incorrect, skipping: '{}'",
                        ignore_prop.value, e
                    );
                    continue;
                }
                // can unwrap because it's checked earlier
                let regex = regex.unwrap();
                if regex.is_match(&line) {
                    debug!(
                        "line '{}' matches regex '{}', removing",
                        line, ignore_prop.value
                    );
                    result = false;
                    break;
                }
            }
        }
    }
    if result {
        Some(format!("{}\n", line))
    } else {
        None
    }
}

pub(crate) fn mv_bundle_back(tmp_dir: &TempDir, bundle: &Bundle) -> Result<()> {
    for file in bundle.paths() {
        mv_files_back(tmp_dir, &Path::new(file))?;
    }
    Ok(())
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
    use std::fs::create_dir_all;
    use std::fs::{read_to_string, File};
    use tempfile::{NamedTempFile, TempDir};

    #[test]
    fn test_allowed_prop() {
        // given
        let ignore_properties = &[
            IgnoreProp {
                ignore_type: IgnoreType::Contains,
                value: "cq:lastModified".to_string(),
            },
            IgnoreProp {
                ignore_type: IgnoreType::Contains,
                value: "testProperty".to_string(),
            },
        ];
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

    #[test]
    fn test_is_file_with_file() -> Result<()> {
        // given
        let tmpfile = NamedTempFile::new()?;
        let entry = Entry::from(tmpfile.path());

        // when
        let is_file = entry.is_file();

        // then
        assert_eq!(is_file, true);

        Ok(())
    }

    #[test]
    fn test_is_file_with_directory() -> Result<()> {
        // given
        let tmpdir = TempDir::new()?;
        let entry = Entry::from(tmpdir.path());

        // when
        let is_file = entry.is_file();

        // then
        assert_eq!(is_file, false);

        Ok(())
    }

    #[test]
    fn test_is_content_xml_with_other_file() -> Result<()> {
        // given
        let tmpdir = TempDir::new()?;
        let xml_filepath = tmpdir.path().join("some-other-file.xml");
        let mut file = File::create(&xml_filepath)?;
        file.write_all(b"test_content")?;
        let entry = Entry::from(xml_filepath.as_path());

        // when
        let is_xml = entry.is_content_xml();

        // then
        assert_eq!(is_xml, false);

        Ok(())
    }

    #[test]
    fn test_is_xml_with_xml_file() -> Result<()> {
        // given
        let tmpdir = TempDir::new()?;
        let xml_filepath = tmpdir.path().join(".content.xml");
        let mut file = File::create(&xml_filepath)?;
        file.write_all(b"test_content")?;
        let entry = Entry::from(xml_filepath.as_path());

        // when
        let is_xml = entry.is_xml_file();

        // then
        assert_eq!(is_xml, true);

        Ok(())
    }

    #[test]
    fn test_mv_files_back_with_regular_file() -> Result<()> {
        // given
        let src_dir = TempDir::new()?;
        create_dir_all(src_dir.path().join("jcr_root"))?;
        File::create(src_dir.path().join("jcr_root/some-file"))?;

        let target_dir = TempDir::new()?;
        create_dir_all(target_dir.path().join("jcr_root"))?;

        let path = Path::new(
            target_dir
                .path()
                .join("jcr_root/some-file")
                .to_str()
                .unwrap(),
        );

        // when
        mv_files_back(&src_dir, &path)?;

        // then
        assert_eq!(target_dir.path().join("jcr_root/some-file").exists(), true);
        Ok(())
    }

    #[test]
    fn test_mv_files_back_with_directory() -> Result<()> {
        // given
        let src_dir = TempDir::new()?;
        create_dir_all(src_dir.path().join("jcr_root/some-dir"))?;
        File::create(src_dir.path().join("jcr_root/some-dir/some-file"))?;

        let target_dir = TempDir::new()?;
        create_dir_all(target_dir.path().join("jcr_root/some-dir"))?;

        let path = Path::new(
            target_dir
                .path()
                .join("jcr_root/some-dir")
                .to_str()
                .unwrap(),
        );

        // when
        mv_files_back(&src_dir, &path)?;

        // then
        assert_eq!(
            target_dir
                .path()
                .join("jcr_root/some-dir/some-file")
                .exists(),
            true
        );
        Ok(())
    }

    #[test]
    fn test_cleanup_files_with_type_contains() -> Result<()> {
        let _ = pretty_env_logger::try_init();
        // given
        let ignore_properties = vec![IgnoreProp {
            ignore_type: IgnoreType::Contains,
            value: "property-to-ignore".into(),
        }];
        let tmp_dir = TempDir::new()?;
        create_dir_all(tmp_dir.path().join("jcr_root"))?;
        let mut file = File::create(tmp_dir.path().join("jcr_root/.content.xml"))?;
        file.write_all(
            r#"some-property
property-to-ignore
other-property"#
                .as_bytes(),
        )?;

        // when
        cleanup_files(&ignore_properties, &tmp_dir)?;

        // then
        let content = read_to_string(tmp_dir.path().join("jcr_root/.content.xml"))?;

        assert_eq!(
            content,
            r#"some-property
other-property
"#
        );
        Ok(())
    }

    #[test]
    fn test_cleanup_files_with_type_regex() -> Result<()> {
        let _ = pretty_env_logger::try_init();
        // given
        let ignore_properties = vec![IgnoreProp {
            ignore_type: IgnoreType::Regex,
            value: ".*to.*".into(),
        }];
        let tmp_dir = TempDir::new()?;
        create_dir_all(tmp_dir.path().join("jcr_root"))?;
        let mut file = File::create(tmp_dir.path().join("jcr_root/.content.xml"))?;
        file.write_all(
            r#"some-property
property-to-ignore
other-to-property
"#
            .as_bytes(),
        )?;

        // when
        cleanup_files(&ignore_properties, &tmp_dir)?;

        // then
        let content = read_to_string(tmp_dir.path().join("jcr_root/.content.xml"))?;

        assert_eq!(
            content,
            r#"some-property
"#
        );
        Ok(())
    }

    #[test]
    fn test_cleanup_files_with_type_regex_and_complex_value() -> Result<()> {
        let _ = pretty_env_logger::try_init();
        // given
        let ignore_properties = vec![IgnoreProp {
            ignore_type: IgnoreType::Regex,
            value: r".*='\[]'".into(),
        }];
        let tmp_dir = TempDir::new()?;
        create_dir_all(tmp_dir.path().join("jcr_root"))?;
        let mut file = File::create(tmp_dir.path().join("jcr_root/.content.xml"))?;
        file.write_all(
            r#"jcr:mixinTypes='[]'
other-property
"#
            .as_bytes(),
        )?;

        // when
        cleanup_files(&ignore_properties, &tmp_dir)?;

        // then
        let content = read_to_string(tmp_dir.path().join("jcr_root/.content.xml"))?;

        assert_eq!(
            content,
            r#"other-property
"#
        );
        Ok(())
    }

    #[test]
    fn test_cleanup_files_with_type_regex_broken_value() -> Result<()> {
        let _ = pretty_env_logger::try_init();
        // given
        let ignore_properties = vec![IgnoreProp {
            ignore_type: IgnoreType::Regex,
            value: "[".into(),
        }];
        let tmp_dir = TempDir::new()?;
        create_dir_all(tmp_dir.path().join("jcr_root"))?;
        let mut file = File::create(tmp_dir.path().join("jcr_root/.content.xml"))?;
        file.write_all(
            r#"jcr:mixinTypes='[]'
other-property
"#
            .as_bytes(),
        )?;

        // when
        cleanup_files(&ignore_properties, &tmp_dir)?;

        // then
        let content = read_to_string(tmp_dir.path().join("jcr_root/.content.xml"))?;

        assert_eq!(
            content,
            r#"jcr:mixinTypes='[]'
other-property
"#
        );
        Ok(())
    }

    #[test]
    fn test_cleanup_files_without_ignoring_properties() -> Result<()> {
        let _ = pretty_env_logger::try_init();
        // given
        let ignore_properties = vec![];
        let tmp_dir = TempDir::new()?;
        create_dir_all(tmp_dir.path().join("jcr_root"))?;
        let mut file = File::create(tmp_dir.path().join("jcr_root/.content.xml"))?;
        file.write_all(
            r#"some-property
property
other-property"#
                .as_bytes(),
        )?;

        // when
        cleanup_files(&ignore_properties, &tmp_dir)?;

        // then
        let content = read_to_string(tmp_dir.path().join("jcr_root/.content.xml"))?;

        assert_eq!(
            content,
            r#"some-property
property
other-property
"#
        );
        Ok(())
    }
}
