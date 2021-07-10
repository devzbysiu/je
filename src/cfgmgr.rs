use crate::cfg::{Bundle, Cfg, IgnoreProp, IgnoreType, Instance};
use crate::cmd;
use anyhow::Result;
use log::{debug, info};
use serde_derive::{Deserialize, Serialize};
use std::env;
use std::fs::read_to_string;
use std::path::Path;

pub(crate) const CONFIG_FILE: &str = ".je";

pub(crate) const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct Version {
    #[serde(rename = "version")]
    value: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
struct Pre030Cfg {
    pub(crate) ignore_properties: Vec<String>,

    #[serde(rename = "profile")]
    pub(crate) profiles: Vec<Instance>,

    #[serde(rename = "bundle")]
    pub(crate) bundles: Option<Vec<Bundle>>,
}

pub(crate) fn handle_cfg_load() -> Result<Cfg> {
    debug!("loading config: {:?}{}", env::current_dir(), CONFIG_FILE);
    if Path::new(CONFIG_FILE).exists() {
        let config_content = read_to_string(CONFIG_FILE)?;
        debug!("read config: {}", config_content);
        let version: Version = toml::from_str(&config_content)?;
        debug!("version: {:#?}", version);
        if version.value.is_none() {
            // old, not versioned configuration
            let cfg: Pre030Cfg = toml::from_str(&read_to_string(CONFIG_FILE)?)?;
            reinit_config_with_current_version(cfg)
        } else {
            // new configuration, for now no transformation needed
            Ok(toml::from_str::<Cfg>(&read_to_string(CONFIG_FILE)?)?)
        }
    } else {
        debug!(".je config doesn't exists, loading default");
        Ok(Cfg::default())
    }
}

fn reinit_config_with_current_version(cfg: Pre030Cfg) -> Result<Cfg> {
    debug!("adjusting configuration to a newer version");
    let res = Cfg {
        version: Some(CURRENT_VERSION.to_string()),
        ignore_properties: adjust_ignore_props(cfg.ignore_properties),
        profiles: cfg.profiles,
        ..Cfg::default()
    };
    cmd::init(&res)?;
    Ok(res)
}

fn adjust_ignore_props(props: Vec<String>) -> Vec<IgnoreProp> {
    let mut res = Vec::new();
    for prop in props {
        res.push(IgnoreProp {
            ignore_type: IgnoreType::Contains,
            value: prop,
        });
    }
    res
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::cfg::{Bundle, IgnoreProp, Instance};
    use anyhow::Result;
    use std::env;
    use std::fs::File;
    use std::io::prelude::*;
    use std::path::PathBuf;
    use tempfile::TempDir;

    #[test]
    fn test_handle_cfg_load_when_config_not_exists() -> Result<()> {
        let _ = pretty_env_logger::try_init();
        // given
        let expected_profiles = vec![Instance::new(
            "author",
            "http://localhost:4502",
            "admin",
            "admin",
        )];
        let expected_ignore_props = vec![];
        let expected_version = Some("0.3.0".into());

        // when
        let cfg = handle_cfg_load()?;

        // then
        assert_eq!(cfg.version, expected_version);
        assert_eq!(cfg.ignore_properties, expected_ignore_props);
        assert_eq!(cfg.profiles, expected_profiles);

        Ok(())
    }

    #[test]
    fn test_handle_cfg_load_when_config_exists_but_is_from_pre_030_version() -> Result<()> {
        let _ = pretty_env_logger::try_init();
        // given
        let test_config = TestConfig::new()?;
        test_config.write_all(
            r#"ignore_properties = ["jcr:created", "jcr:createdBy"]

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
        let expected_ignore_props = vec![
            IgnoreProp {
                ignore_type: IgnoreType::Contains,
                value: "jcr:created".to_string(),
            },
            IgnoreProp {
                ignore_type: IgnoreType::Contains,
                value: "jcr:createdBy".to_string(),
            },
        ];
        let expected_version = Some("0.3.0".into());

        // when
        let cfg = handle_cfg_load()?;

        // then
        assert_eq!(cfg.version, expected_version);
        assert_eq!(cfg.ignore_properties, expected_ignore_props);
        assert_eq!(cfg.profiles, expected_profiles);

        Ok(())
    }

    #[test]
    fn test_handle_cfg_load_when_config_is_not_available() -> Result<()> {
        let _ = pretty_env_logger::try_init();
        // when
        let cfg = handle_cfg_load()?;

        let expected_profiles = vec![Instance::new(
            "author",
            "http://localhost:4502",
            "admin",
            "admin",
        )];

        // then
        assert_eq!(cfg.ignore_properties, Vec::<IgnoreProp>::new());
        assert_eq!(cfg.profiles, expected_profiles);

        Ok(())
    }

    #[test]
    fn test_instance_with_existing_profile() -> Result<()> {
        let _ = pretty_env_logger::try_init();
        // given
        let test_config = TestConfig::new()?;
        test_config.write_all(
            r#"version = "0.3.0"
               ignore_properties = [{type = "contains", value = "prop1"},
                                    {type = "contains", value = "prop2"}]

               [[profile]]
               name = "author"
               addr = "http://localhost:4502"
               user = "user1"
               pass = "pass1"
            "#,
        )?;
        let expected_instance = Instance::new("author", "http://localhost:4502", "user1", "pass1");

        // when
        let cfg = handle_cfg_load()?;
        let instance = cfg.instance(Some(&String::from("author")));

        // then
        assert_eq!(instance, expected_instance);
        Ok(())
    }

    #[test]
    fn test_instance_with_not_existing_profile() -> Result<()> {
        let _ = pretty_env_logger::try_init();
        // given
        let test_config = TestConfig::new()?;
        test_config.write_all(
            r#"version = "0.3.0"
               ignore_properties = [{type = "contains", value = "prop1"},
                                    {type = "contains", value = "prop2"}]

               [[profile]]
               name = "author"
               addr = "http://localhost:4502"
               user = "user1"
               pass = "pass1"
            "#,
        )?;
        let default_instance = Instance::new("author", "http://localhost:4502", "admin", "admin");

        // when
        let cfg = handle_cfg_load()?;
        let instance = cfg.instance(Some(&String::from("not-existing")));

        // then
        assert_eq!(instance, default_instance);
        Ok(())
    }

    #[test]
    fn test_instance_when_no_profile_was_selected() -> Result<()> {
        let _ = pretty_env_logger::try_init();
        // given
        let test_config = TestConfig::new()?;
        test_config.write_all(
            r#"version = "0.3.0"
               ignore_properties = [{type = "contains", value = "prop1"},
                                    {type = "contains", value = "prop2"}]

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
        let cfg = handle_cfg_load()?;
        let instance = cfg.instance(None);

        // then
        assert_eq!(instance, first_instance);
        Ok(())
    }

    #[test]
    fn test_bundles_when_bundle_defined() -> Result<()> {
        let _ = pretty_env_logger::try_init();
        // given
        let test_config = TestConfig::new()?;
        test_config.write_all(
            r#"version = "0.3.0"
               ignore_properties = [{type = "contains", value = "prop1"},
                                    {type = "contains", value = "prop2"}]

               [[profile]]
               name = "author"
               addr = "http://localhost:4502"
               user = "user1"
               pass = "pass1"

               [[bundle]]
               name = "simple"
               paths = ["file1", "file2"]
            "#,
        )?;
        let expected_bundle = Bundle::new("simple", vec!["file1", "file2"]);

        // when
        let cfg = handle_cfg_load();
        debug!("result: {:?}", cfg);
        let cfg = handle_cfg_load()?;
        let bundle = cfg.bundle(Some("simple"));

        // // then
        assert_eq!(expected_bundle, bundle);
        Ok(())
    }

    #[test]
    fn test_bundles_when_multiple_bundles_defined() -> Result<()> {
        let _ = pretty_env_logger::try_init();
        // given
        let test_config = TestConfig::new()?;
        test_config.write_all(
            r#"version = "0.3.0"
               ignore_properties = [{type = "contains", value = "prop1"},
                                    {type = "contains", value = "prop2"}]

               [[profile]]
               name = "author"
               addr = "http://localhost:4502"
               user = "user1"
               pass = "pass1"

               [[bundle]]
               name = "simple"
               paths = ["file1", "file2"]

               [[bundle]]
               name = "other"
               paths = ["file3", "file4"]
            "#,
        )?;
        let expected_simple_bundle = Bundle::new("simple", vec!["file1", "file2"]);
        let expected_other_bundle = Bundle::new("other", vec!["file3", "file4"]);

        // when
        let cfg = handle_cfg_load()?;
        let simple_bundle = cfg.bundle(Some("simple"));
        let other_bundle = cfg.bundle(Some("other"));

        // then
        assert_eq!(expected_simple_bundle, simple_bundle);
        assert_eq!(expected_other_bundle, other_bundle);
        Ok(())
    }

    #[test]
    fn test_bundles_when_no_bundle() -> Result<()> {
        let _ = pretty_env_logger::try_init();
        // given
        let test_config = TestConfig::new()?;
        test_config.write_all(
            r#"version = "0.3.0"
               ignore_properties = [{type = "contains", value = "prop1"},
                                    {type = "contains", value = "prop2"}]

               [[profile]]
               name = "author"
               addr = "http://localhost:4502"
               user = "user1"
               pass = "pass1"
            "#,
        )?;
        let expected_bundle = Bundle::default();

        // when
        let cfg = handle_cfg_load()?;
        let bundle = cfg.bundle(Some("not-existing"));

        // then
        assert_eq!(expected_bundle, bundle);
        Ok(())
    }

    #[test]
    #[should_panic]
    fn test_bundles_when_bundle_broken() {
        let _ = pretty_env_logger::try_init();
        // given
        let test_config = TestConfig::new().unwrap();
        test_config
            .write_all(
                r#"version = "0.3.0"
                   ignore_properties = [{type = "contains", value = "prop1"},
                                        {type = "contains", value = "prop2"}]

                   [[profile]]
                   name = "author"
                   addr = "http://localhost:4502"
                   user = "user1"
                   pass = "pass1"

                   [[bundle]]
                   paths = ["file3", "file4"]
                "#,
            )
            .unwrap();

        // when
        let _not_important = handle_cfg_load().unwrap(); // should panic
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
