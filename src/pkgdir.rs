use crate::cfg::Bundle;
use crate::path::Path;
use anyhow::Result;
use log::{debug, info};
use std::convert::Into;
use std::fs::{create_dir_all, remove_dir_all, File};
use std::io::prelude::*;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use tempfile::TempDir;

#[derive(Debug)]
pub(crate) struct Pkg {
    name: String,
    version: String,
    group: String,
}

impl Pkg {
    pub(crate) fn path(&self) -> String {
        format!("{}/{}-{}.zip", self.group, self.name, self.version)
    }
}

impl Default for Pkg {
    fn default() -> Self {
        Self {
            name: "je-pkg".into(),
            version: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("failed to count secs from EPOCH")
                .as_secs()
                .to_string(),
            group: "je".into(),
        }
    }
}

pub(crate) fn mksimple(path: &Path, pkg: &Pkg) -> Result<TempDir> {
    debug!("creating pkg dir");
    let tmp_dir = TempDir::new()?;
    mk_jcr_root_dir(&tmp_dir)?;
    mk_vault_dir(&tmp_dir)?;
    write_filter_content(&tmp_dir, &[path.content()])?;
    write_properties_content(&tmp_dir, pkg)?;
    Ok(tmp_dir)
}

fn mk_jcr_root_dir(tmp_dir: &TempDir) -> Result<()> {
    let jcr_root_dir_path = tmp_dir.path().join("jcr_root");
    debug!("creating jcr_root dir: {}", jcr_root_dir_path.display());
    create_dir_all(jcr_root_dir_path)?;
    Ok(())
}

fn mk_vault_dir(tmp_dir: &TempDir) -> Result<()> {
    debug!("creating vault dir: {}", vault_path(tmp_dir).display());
    create_dir_all(vault_path(tmp_dir))?;
    Ok(())
}

fn vault_path(tmp_dir: &TempDir) -> PathBuf {
    tmp_dir.path().join("META-INF/vault")
}

fn write_filter_content(tmp_dir: &TempDir, content_paths: &[String]) -> Result<()> {
    let filter_path = format!("{}/filter.xml", vault_path(tmp_dir).display());
    let mut filter_file = File::create(&filter_path)?;
    let filter_content = filter_content(content_paths);
    debug!(
        "writing content\n{}\nto filter {}",
        filter_content, filter_path
    );
    filter_file.write_all(filter_content.as_bytes())?;
    Ok(())
}

fn filter_content(paths: &[String]) -> String {
    let filter_prefix = r#"<?xml version="1.0" encoding="UTF-8"?>
<workspaceFilter version="1.0">"#;
    let filter_postfix = "</workspaceFilter>";
    format!(
        r#"{}
    {}
{}
"#,
        filter_prefix,
        write_filters(paths),
        filter_postfix,
    )
}

fn write_filters(paths: &[String]) -> String {
    paths
        .iter()
        .map(normalize)
        .map(create_filter)
        .collect::<Vec<String>>()
        .join("\n    ")
}

fn normalize<S: Into<String>>(path: S) -> String {
    path.into()
        .replace("_jcr_", "jcr:")
        .replace("_rep_", "rep:")
        .replace("_oak_", "oak:")
        .replace("_sling_", "sling:")
        .replace("_granite_", "granite:")
        .replace("_cq_", "cq:")
        .replace("_dam_", "dam:")
        .replace("_exif_", "exif:")
        .replace("_social_", "social:")
        .replace(".content.xml", "")
        .replace(".xml", "")
        // windows
        .replace('\\', "/")
}

fn create_filter<S: Into<String>>(path: S) -> String {
    format!(r#"<filter root="{}"/>"#, path.into())
}

fn write_properties_content(tmp_dir: &TempDir, pkg: &Pkg) -> Result<()> {
    let prop_path = format!("{}/properties.xml", vault_path(tmp_dir).display());
    let mut prop_file = File::create(&prop_path)?;
    let properties_content = properties_content(pkg);
    debug!(
        "writing content\n{}\nto properties file {}",
        &properties_content, prop_path
    );
    prop_file.write_all(properties_content.as_bytes())?;
    Ok(())
}

fn properties_content(pkg: &Pkg) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="no"?>
<!DOCTYPE properties SYSTEM "http://java.sun.com/dtd/properties.dtd">
<properties>
    <entry key="name">{}</entry>
    <entry key="version">{}</entry>
    <entry key="group">{}</entry>
</properties>"#,
        pkg.name, pkg.version, pkg.group
    )
}

pub(crate) fn mkbundle(bundle: &Bundle, pkg: &Pkg) -> Result<TempDir> {
    debug!("creating pkg dir");
    let tmp_dir = TempDir::new()?;
    mk_jcr_root_dir(&tmp_dir)?;
    mk_vault_dir(&tmp_dir)?;
    write_filter_content(&tmp_dir, bundle.paths())?;
    write_properties_content(&tmp_dir, pkg)?;
    Ok(tmp_dir)
}

pub(crate) fn clean(tmp_dir: &TempDir) -> Result<()> {
    info!("cleaning tmp dir: {}", tmp_dir.path().display());
    remove_dir_all(tmp_dir)?;
    create_dir_all(tmp_dir)?;
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    use anyhow::Result;
    use std::fs::create_dir_all;
    use std::fs::read_to_string;
    use std::path::Path as OsPath;
    use tempfile::TempDir;

    #[test]
    fn test_path_with_correct_pkg() {
        // given
        let pkg = Pkg {
            name: "pkg-name".into(),
            version: "1.0.0".into(),
            group: "group-name".into(),
        };

        // when
        let path = pkg.path();

        // then
        assert_eq!(path, "group-name/pkg-name-1.0.0.zip");
    }

    #[test]
    fn test_path_with_incorrect_pkg() {
        // given
        let pkg = Pkg {
            name: String::new(),
            version: String::new(),
            group: "group-name".into(),
        };

        // when
        let path = pkg.path();

        // then
        assert_eq!(path, "group-name/-.zip");
    }

    #[test]
    fn test_mk_jcr_root_dir() -> Result<()> {
        // given
        let tmp_dir = TempDir::new()?;

        // when
        mk_jcr_root_dir(&tmp_dir)?;

        // then
        assert!(OsPath::new(&format!("{}/jcr_root", tmp_dir.path().display())).exists());
        Ok(())
    }

    #[test]
    fn test_mk_vault_dir() -> Result<()> {
        // given
        let tmp_dir = TempDir::new()?;

        // when
        mk_vault_dir(&tmp_dir)?;

        // then
        assert!(OsPath::new(&format!("{}/META-INF/vault", tmp_dir.path().display())).exists());
        Ok(())
    }

    #[test]
    fn test_write_filter_content() -> Result<()> {
        // given
        let tmp_dir = TempDir::new()?;
        create_dir_all(format!("{}/META-INF/vault", tmp_dir.path().display()))?;

        // when
        write_filter_content(&tmp_dir, &["/content/path".into()])?;

        // then
        assert!(OsPath::new(&format!(
            "{}/META-INF/vault/filter.xml",
            tmp_dir.path().display()
        ))
        .exists());
        let filter_contents = read_to_string(format!(
            "{}/META-INF/vault/filter.xml",
            tmp_dir.path().display()
        ))?;
        assert_eq!(
            filter_contents,
            r#"<?xml version="1.0" encoding="UTF-8"?>
<workspaceFilter version="1.0">
    <filter root="/content/path"/>
</workspaceFilter>
"#,
        );
        Ok(())
    }

    #[test]
    fn test_write_properties_content() -> Result<()> {
        // given
        let tmp_dir = TempDir::new()?;
        create_dir_all(format!("{}/META-INF/vault", tmp_dir.path().display()))?;
        let pkg = Pkg::default();

        // when
        write_properties_content(&tmp_dir, &pkg)?;

        // then
        assert!(OsPath::new(&format!(
            "{}/META-INF/vault/properties.xml",
            tmp_dir.path().display()
        ))
        .exists());
        let properties_contents = read_to_string(format!(
            "{}/META-INF/vault/properties.xml",
            tmp_dir.path().display()
        ))?;
        assert_eq!(
            properties_contents,
            format!(
                r#"<?xml version="1.0" encoding="UTF-8" standalone="no"?>
<!DOCTYPE properties SYSTEM "http://java.sun.com/dtd/properties.dtd">
<properties>
    <entry key="name">{}</entry>
    <entry key="version">{}</entry>
    <entry key="group">{}</entry>
</properties>"#,
                pkg.name, pkg.version, pkg.group
            ),
        );
        Ok(())
    }

    #[test]
    fn test_mk_simple_dir() -> Result<()> {
        // given
        let file_path = Path::new("/home/user/project/jcr_root/content/client");
        let pkg = Pkg::default();

        // when
        let tmp_dir_path = mksimple(&file_path, &pkg)?;

        // then
        assert!(OsPath::new(&format!("{}/jcr_root", tmp_dir_path.path().display())).exists());
        assert!(OsPath::new(&format!(
            "{}/META-INF/vault/filter.xml",
            tmp_dir_path.path().display()
        ))
        .exists());
        let filter_contents = read_to_string(format!(
            "{}/META-INF/vault/filter.xml",
            tmp_dir_path.path().display()
        ))?;
        assert_eq!(
            filter_contents,
            r#"<?xml version="1.0" encoding="UTF-8"?>
<workspaceFilter version="1.0">
    <filter root="/content/client"/>
</workspaceFilter>
"#,
        );
        assert!(OsPath::new(&format!(
            "{}/META-INF/vault/properties.xml",
            tmp_dir_path.path().display()
        ))
        .exists());

        Ok(())
    }

    #[test]
    fn test_normalize() {
        // given
        let test_cases = vec![
            ("/content/_jcr_content", "/content/jcr:content"),
            ("/content/_rep_policy", "/content/rep:policy"),
            ("/content/_oak_root", "/content/oak:root"),
            ("/content/_sling_order", "/content/sling:order"),
            ("/content/_granite_var", "/content/granite:var"),
            ("/content/_cq_dialog", "/content/cq:dialog"),
            ("/content/_dam_asset", "/content/dam:asset"),
            ("/content/_exif_fi", "/content/exif:fi"),
            ("/content/_social_media", "/content/social:media"),
            (
                "/content/_jcr_content/.content.xml",
                "/content/jcr:content/",
            ),
            ("/content/_jcr_content.xml", "/content/jcr:content"),
            // Windows
            ("\\content\\_jcr_content", "/content/jcr:content"),
            ("\\content\\_rep_policy", "/content/rep:policy"),
            ("\\content\\_oak_root", "/content/oak:root"),
            ("\\content\\_sling_order", "/content/sling:order"),
            ("\\content\\_granite_var", "/content/granite:var"),
            ("\\content\\_cq_dialog", "/content/cq:dialog"),
            ("\\content\\_dam_asset", "/content/dam:asset"),
            ("\\content\\_exif_fi", "/content/exif:fi"),
            ("\\content\\_social_media", "/content/social:media"),
            (
                "\\content\\_jcr_content\\.content.xml",
                "/content/jcr:content/",
            ),
            ("\\content\\_jcr_content.xml", "/content/jcr:content"),
        ];

        // then
        for (input, expected) in test_cases {
            assert_eq!(normalize(input), expected);
        }
    }

    #[test]
    fn test_mk_bundle() -> Result<()> {
        // given
        let bundle = Bundle::new(
            "NOT-IMPORTANT",
            vec!["/content/client", "/content/dam/test"],
        );
        let pkg = Pkg::default();

        // when
        let tmp_dir_path = mkbundle(&bundle, &pkg)?;

        // then
        assert!(OsPath::new(&format!("{}/jcr_root", tmp_dir_path.path().display())).exists());
        assert!(OsPath::new(&format!(
            "{}/META-INF/vault/filter.xml",
            tmp_dir_path.path().display()
        ))
        .exists());
        let filter_contents = read_to_string(format!(
            "{}/META-INF/vault/filter.xml",
            tmp_dir_path.path().display()
        ))?;
        assert_eq!(
            filter_contents,
            r#"<?xml version="1.0" encoding="UTF-8"?>
<workspaceFilter version="1.0">
    <filter root="/content/client"/>
    <filter root="/content/dam/test"/>
</workspaceFilter>
"#,
        );
        assert!(OsPath::new(&format!(
            "{}/META-INF/vault/properties.xml",
            tmp_dir_path.path().display()
        ))
        .exists());

        Ok(())
    }

    #[test]
    fn test_clean_when_dir_already_empty() -> Result<()> {
        // given
        let tmp_dir = TempDir::new()?;

        // when
        clean(&tmp_dir)?;

        // then
        assert!(tmp_dir.path().read_dir()?.next().is_none());

        Ok(())
    }

    #[test]
    fn test_clean_when_has_files() -> Result<()> {
        // given
        let tmp_dir = TempDir::new()?;
        File::create(tmp_dir.path().join("test-file1"))?;
        File::create(tmp_dir.path().join("test-file2"))?;
        File::create(tmp_dir.path().join("test-file3"))?;
        File::create(tmp_dir.path().join("test-file4"))?;

        // when
        clean(&tmp_dir)?;

        // then
        assert!(tmp_dir.path().read_dir()?.next().is_none());

        Ok(())
    }
}
