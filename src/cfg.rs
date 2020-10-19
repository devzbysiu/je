use anyhow::Result;
use log::debug;
use serde_derive::{Deserialize, Serialize};
use std::env;
use std::fs::read_to_string;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct Cfg {
    pub(crate) ignore_properties: Vec<String>,

    #[serde(rename = "profile")]
    pub(crate) profiles: Vec<Instance>,
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
        let default_instance = Instance {
            name: "author".to_string(),
            addr: "http://localhost:4502".to_string(),
            user: "admin".to_string(),
            pass: "admin".to_string(),
        };
        let profiles = self.profiles.clone();
        match profile {
            Some(name) => profiles
                .into_iter()
                .find(|p| p.name == *name)
                .unwrap_or(default_instance),
            None => profiles.into_iter().next().unwrap_or(default_instance),
        }
    }
}

impl Default for Cfg {
    fn default() -> Self {
        Self {
            profiles: vec![Instance {
                name: "author".into(),
                addr: "http://localhost:4502".into(),
                user: "admin".into(),
                pass: "admin".into(),
            }],
            ignore_properties: vec![],
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub(crate) struct Instance {
    pub(crate) name: String,
    pub(crate) addr: String,
    pub(crate) user: String,
    pub(crate) pass: String,
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

#[cfg(test)]
mod test {
    use super::*;
    use std::env;
    use std::fs::File;
    use std::io::prelude::*;
    use tempfile::TempDir;

    #[test]
    fn test_default() -> Result<()> {
        // given
        let expected_instance = Instance {
            name: "author".into(),
            addr: "http://localhost:4502".into(),
            user: "admin".into(),
            pass: "admin".into(),
        };

        // when
        let default_instance = Instance::default();

        // then
        assert_eq!(default_instance, expected_instance);
        Ok(())
    }

    #[test]
    fn test_load_when_config_exists() -> Result<()> {
        // given
        let initial_dir = env::current_dir()?;
        let tmp_dir = TempDir::new()?;
        env::set_current_dir(tmp_dir.path())?;
        let mut cfg_file = File::create(".je")?;
        cfg_file.write_all(
            r#"ignore_properties = ["prop1", "prop2"]

[[profile]]
name = "author"
addr = "http://localhost:4502"
user = "user1"
pass = "pass1"
"#
            .as_bytes(),
        )?;

        let expected_profiles = vec![Instance {
            name: "author".into(),
            addr: "http://localhost:4502".into(),
            user: "user1".into(),
            pass: "pass1".into(),
        }];

        // when
        let cfg = Cfg::load()?;

        // then
        assert_eq!(cfg.ignore_properties, vec!["prop1", "prop2"]);
        assert_eq!(cfg.profiles, expected_profiles);

        env::set_current_dir(initial_dir)?;
        Ok(())
    }

    #[test]
    fn test_load_when_config_is_not_available() -> Result<()> {
        // when
        let cfg = Cfg::load()?;

        let expected_profiles = vec![Instance {
            name: "author".into(),
            addr: "http://localhost:4502".into(),
            user: "admin".into(),
            pass: "admin".into(),
        }];

        // then
        assert_eq!(cfg.ignore_properties, Vec::<String>::new());
        assert_eq!(cfg.profiles, expected_profiles);

        Ok(())
    }

    #[test]
    fn test_instance_with_existing_profile() -> Result<()> {
        // given
        let initial_dir = env::current_dir()?;
        let tmp_dir = TempDir::new()?;
        env::set_current_dir(tmp_dir.path())?;
        let mut cfg_file = File::create(".je")?;
        cfg_file.write_all(
            r#"ignore_properties = ["prop1", "prop2"]

[[profile]]
name = "author"
addr = "http://localhost:4502"
user = "user1"
pass = "pass1"
"#
            .as_bytes(),
        )?;
        let expected_instance = Instance {
            name: "author".into(),
            addr: "http://localhost:4502".into(),
            user: "user1".into(),
            pass: "pass1".into(),
        };

        // when
        let cfg = Cfg::load()?;
        let instance = cfg.instance(Some(&String::from("author")));

        // then
        assert_eq!(instance, expected_instance);

        env::set_current_dir(initial_dir)?;
        Ok(())
    }

    #[test]
    fn test_instance_with_not_existing_profile() -> Result<()> {
        // given
        let initial_dir = env::current_dir()?;
        let tmp_dir = TempDir::new()?;
        env::set_current_dir(tmp_dir.path())?;
        let mut cfg_file = File::create(".je")?;
        cfg_file.write_all(
            r#"ignore_properties = ["prop1", "prop2"]

[[profile]]
name = "author"
addr = "http://localhost:4502"
user = "user1"
pass = "pass1"
"#
            .as_bytes(),
        )?;
        let default_instance = Instance {
            name: "author".into(),
            addr: "http://localhost:4502".into(),
            user: "admin".into(),
            pass: "admin".into(),
        };

        // when
        let cfg = Cfg::load()?;
        let instance = cfg.instance(Some(&String::from("not-existing")));

        // then
        assert_eq!(instance, default_instance);

        env::set_current_dir(initial_dir)?;
        Ok(())
    }

    #[test]
    fn test_instance_when_no_profile_was_selected() -> Result<()> {
        // given
        let initial_dir = env::current_dir()?;
        let tmp_dir = TempDir::new()?;
        env::set_current_dir(tmp_dir.path())?;
        let mut cfg_file = File::create(".je")?;
        cfg_file.write_all(
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

"#
            .as_bytes(),
        )?;
        let first_instance = Instance {
            name: "publish".into(),
            addr: "http://localhost:4503".into(),
            user: "user2".into(),
            pass: "pass2".into(),
        };

        // when
        let cfg = Cfg::load()?;
        let instance = cfg.instance(None);

        // then
        assert_eq!(instance, first_instance);

        env::set_current_dir(initial_dir)?;
        Ok(())
    }
}
