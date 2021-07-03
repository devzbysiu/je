use crate::cfg::Cfg;
use crate::cmd;
use anyhow::Result;
use log::{debug, info};
use std::env;
use std::fs::read_to_string;
use std::path::Path;

pub(crate) const VERSION: &str = env!("CARGO_PKG_VERSION");

pub(crate) fn handle_cfg_load() -> Result<Cfg> {
    debug!("loading config from {:?}.je", env::current_dir());
    if Path::new(".je").exists() {
        let cfg: Cfg = toml::from_str(&read_to_string(".je")?)?;
        Ok(match cfg.version {
            None => reinit_config_with_current_version(cfg)?,
            Some(ref ver) if ver != VERSION => reinit_config_with_current_version(cfg)?,
            Some(ref _current_version) => {
                info!("config file with current version");
                cfg
            }
        })
    } else {
        debug!(".je config doesn't exists, loading default");
        Ok(Cfg::default())
    }
}

fn reinit_config_with_current_version(mut cfg: Cfg) -> Result<Cfg> {
    debug!("adjusting configuration to a newer version");
    cfg.version = Some(VERSION.to_string());
    cmd::init(&cfg)?;
    Ok(cfg)
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
    fn test_handle_cfg_load_when_config_exists_but_is_from_older_version() -> Result<()> {
        // given
        let test_config = TestConfig::new()?;
        test_config.write_all(
            r#"ignore_properties = []

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
    fn test_handle_cfg_load_when_config_is_not_available() -> Result<()> {
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
        // given
        let test_config = TestConfig::new()?;
        test_config.write_all(
            r#"ignore_properties = [{type = "contains", value = "prop1"}, 
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
        // given
        let test_config = TestConfig::new()?;
        test_config.write_all(
            r#"ignore_properties = [{type = "contains", value = "prop1"},
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
        // given
        let test_config = TestConfig::new()?;
        test_config.write_all(
            r#"ignore_properties = [{type = "contains", value = "prop1"},
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
        // given
        let test_config = TestConfig::new()?;
        test_config.write_all(
            r#"ignore_properties = [{type = "contains", value = "prop1"},
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
        // given
        let test_config = TestConfig::new()?;
        test_config.write_all(
            r#"ignore_properties = [{type = "contains", value = "prop1"},
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
        // given
        let test_config = TestConfig::new()?;
        test_config.write_all(
            r#"ignore_properties = [{type = "contains", value = "prop1"},
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
        // given
        let test_config = TestConfig::new().unwrap();
        test_config
            .write_all(
                r#"ignore_properties = [{type = "contains", value = "prop1"},
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
