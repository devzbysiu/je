use anyhow::Result;
use getset::Getters;
use log::debug;
use serde_derive::{Deserialize, Serialize};
use std::convert::Into;
use std::env;
use std::fs::read_to_string;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct Cfg {
    pub(crate) ignore_properties: Vec<String>,

    #[serde(rename = "profile")]
    pub(crate) profiles: Vec<Instance>,

    #[serde(rename = "bundle")]
    pub(crate) bundles: Option<Vec<Bundle>>,
}

impl Cfg {
    pub(crate) fn load() -> Result<Cfg> {
        debug!("loading config from {:?}.je", env::current_dir());
        if Path::new(".je").exists() {
            Ok(toml::from_str(&read_to_string(".je")?)?)
        } else {
            debug!(".je config doesn't exists, loading default");
            Ok(Cfg::default())
        }
    }

    pub(crate) fn instance(&self, profile: Option<&String>) -> Instance {
        let default_instance = Instance::default();
        let profiles = self.profiles.clone();
        match profile {
            Some(name) => profiles
                .into_iter()
                .find(|p| p.name == *name)
                .unwrap_or(default_instance),
            None => profiles.into_iter().next().unwrap_or(default_instance),
        }
    }

    pub(crate) fn bundle(&self, bundle: Option<&str>) -> Bundle {
        if self.bundles.is_none() {
            return Bundle::default();
        }
        let bundles = self.bundles.clone().unwrap(); // can unwrap because it was checked earlier
        match bundle {
            Some(name) => bundles
                .into_iter()
                .find(|p| p.name == *name)
                .unwrap_or(Bundle::default()),
            None => Bundle::default(),
        }
    }
}

impl Default for Cfg {
    fn default() -> Self {
        Self {
            profiles: vec![Instance::new(
                "author",
                "http://localhost:4502",
                "admin",
                "admin",
            )],
            ignore_properties: vec![],
            bundles: None,
        }
    }
}

#[derive(Getters, Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
#[getset(get = "pub")]
pub(crate) struct Instance {
    name: String,
    addr: String,
    user: String,
    pass: String,
}

impl Instance {
    fn new<S: Into<String>>(name: S, addr: S, user: S, pass: S) -> Self {
        Self {
            name: name.into(),
            addr: addr.into(),
            user: user.into(),
            pass: pass.into(),
        }
    }
}

impl Default for Instance {
    fn default() -> Self {
        Self {
            name: "author".into(),
            addr: "http://localhost:4502".into(),
            user: "admin".into(),
            pass: "admin".into(),
        }
    }
}

#[derive(Getters, Default, Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
#[getset(get = "pub")]
pub(crate) struct Bundle {
    name: String,
    files: Vec<String>,
}

impl Bundle {
    #[allow(dead_code)] // TODO: remove this
    pub(crate) fn new<S: Into<String>>(name: S, files: Vec<S>) -> Self {
        Self {
            name: name.into(),
            files: files.into_iter().map(Into::into).collect(),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::env;
    use std::fs::File;
    use std::io::prelude::*;
    use std::path::PathBuf;
    use tempfile::TempDir;

    #[test]
    fn test_cfg_default() {
        // given
        let expected_cfg = Cfg {
            ignore_properties: vec![],
            profiles: vec![Instance::new(
                "author",
                "http://localhost:4502",
                "admin",
                "admin",
            )],
            bundles: None,
        };

        // when
        let default_cfg = Cfg::default();

        // then
        assert_eq!(default_cfg, expected_cfg);
    }

    #[test]
    fn test_load_when_config_exists() -> Result<()> {
        // given
        let test_config = TestConfig::new()?;
        test_config.write_all(
            r#"ignore_properties = ["prop1", "prop2"]

[[profile]]
name = "author"
addr = "http://localhost:4502"
user = "user1"
pass = "pass1"
"#,
        )?;

        let expected_profiles = vec![Instance::new(
            "author",
            "http://localhost:4502",
            "user1",
            "pass1",
        )];

        // when
        let cfg = Cfg::load()?;

        // then
        assert_eq!(cfg.ignore_properties, vec!["prop1", "prop2"]);
        assert_eq!(cfg.profiles, expected_profiles);
        Ok(())
    }

    #[test]
    fn test_load_when_config_is_not_available() -> Result<()> {
        // when
        let cfg = Cfg::load()?;

        let expected_profiles = vec![Instance::new(
            "author",
            "http://localhost:4502",
            "admin",
            "admin",
        )];

        // then
        assert_eq!(cfg.ignore_properties, Vec::<String>::new());
        assert_eq!(cfg.profiles, expected_profiles);

        Ok(())
    }

    #[test]
    fn test_instance_with_existing_profile() -> Result<()> {
        // given
        let test_config = TestConfig::new()?;
        test_config.write_all(
            r#"ignore_properties = ["prop1", "prop2"]

[[profile]]
name = "author"
addr = "http://localhost:4502"
user = "user1"
pass = "pass1"
"#,
        )?;
        let expected_instance = Instance::new("author", "http://localhost:4502", "user1", "pass1");

        // when
        let cfg = Cfg::load()?;
        let instance = cfg.instance(Some(&String::from("author")));

        // then
        assert_eq!(instance, expected_instance);
        Ok(())
    }

    #[test]
    fn test_instance_with_not_existing_profile() -> Result<()> {
        // given
        let test_config = TestConfig::new()?;
        test_config.write_all(
            r#"ignore_properties = ["prop1", "prop2"]

[[profile]]
name = "author"
addr = "http://localhost:4502"
user = "user1"
pass = "pass1"
"#,
        )?;
        let default_instance = Instance::new("author", "http://localhost:4502", "admin", "admin");

        // when
        let cfg = Cfg::load()?;
        let instance = cfg.instance(Some(&String::from("not-existing")));

        // then
        assert_eq!(instance, default_instance);
        Ok(())
    }

    #[test]
    fn test_instance_when_no_profile_was_selected() -> Result<()> {
        // given
        let test_config = TestConfig::new()?;
        test_config.write_all(
            r#"ignore_properties = ["prop1", "prop2"]

[[profile]]
name = "publish"
addr = "http://localhost:4503"
user = "user2"
pass = "pass2"

[[profile]]
name = "author"
addr = "http://localhost:4502"
user = "user1"
pass = "pass1"

"#,
        )?;
        let first_instance = Instance::new("publish", "http://localhost:4503", "user2", "pass2");

        // when
        let cfg = Cfg::load()?;
        let instance = cfg.instance(None);

        // then
        assert_eq!(instance, first_instance);
        Ok(())
    }

    #[test]
    fn test_bundles_when_bundle_defined() -> Result<()> {
        // given
        let test_config = TestConfig::new()?;
        test_config.write_all(
            r#"ignore_properties = ["prop1", "prop2"]

[[profile]]
name = "author"
addr = "http://localhost:4502"
user = "user1"
pass = "pass1"

[[bundle]]
name = "simple"
files = ["file1", "file2"]

"#,
        )?;
        let expected_bundle = Bundle::new("simple", vec!["file1", "file2"]);

        // when
        let cfg = Cfg::load()?;
        let bundle = cfg.bundle(Some("simple"));

        // then
        assert_eq!(expected_bundle, bundle);
        Ok(())
    }

    #[test]
    fn test_bundles_when_multiple_bundles_defined() -> Result<()> {
        // given
        let test_config = TestConfig::new()?;
        test_config.write_all(
            r#"ignore_properties = ["prop1", "prop2"]

[[profile]]
name = "author"
addr = "http://localhost:4502"
user = "user1"
pass = "pass1"

[[bundle]]
name = "simple"
files = ["file1", "file2"]

[[bundle]]
name = "other"
files = ["file3", "file4"]

"#,
        )?;
        let expected_simple_bundle = Bundle::new("simple", vec!["file1", "file2"]);
        let expected_other_bundle = Bundle::new("other", vec!["file3", "file4"]);

        // when
        let cfg = Cfg::load()?;
        let simple_bundle = cfg.bundle(Some("simple"));
        let other_bundle = cfg.bundle(Some("other"));

        // then
        assert_eq!(expected_simple_bundle, simple_bundle);
        assert_eq!(expected_other_bundle, other_bundle);
        Ok(())
    }

    #[test]
    fn test_bundles_when_no_bundle() -> Result<()> {
        // given
        let test_config = TestConfig::new()?;
        test_config.write_all(
            r#"ignore_properties = ["prop1", "prop2"]

[[profile]]
name = "author"
addr = "http://localhost:4502"
user = "user1"
pass = "pass1"

"#,
        )?;
        let expected_bundle = Bundle::default();

        // when
        let cfg = Cfg::load()?;
        let bundle = cfg.bundle(Some("not-existing"));

        // then
        assert_eq!(expected_bundle, bundle);
        Ok(())
    }

    #[test]
    #[should_panic]
    fn test_bundles_when_bundle_broken() {
        // given
        let test_config = TestConfig::new().unwrap();
        test_config
            .write_all(
                r#"ignore_properties = ["prop1", "prop2"]

[[profile]]
name = "author"
addr = "http://localhost:4502"
user = "user1"
pass = "pass1"

[[bundle]]
files = ["file3", "file4"]

"#,
            )
            .unwrap();

        // when
        let _ = Cfg::load().unwrap(); // should panic
    }

    struct TestConfig {
        initial_dir: PathBuf,
        tmp_dir: TempDir,
    }

    impl TestConfig {
        fn new() -> Result<Self> {
            Ok(TestConfig {
                initial_dir: env::current_dir()?,
                tmp_dir: TempDir::new()?,
            })
        }

        fn write_all<S: Into<String>>(&self, content: S) -> Result<()> {
            env::set_current_dir(self.tmp_dir.path())?;
            let mut cfg_file = File::create(".je")?;
            cfg_file.write_all(content.into().as_bytes())?;
            Ok(())
        }
    }

    impl Drop for TestConfig {
        fn drop(&mut self) {
            env::set_current_dir(self.initial_dir.clone())
                .expect("failed to change to an initial dir");
        }
    }
}
