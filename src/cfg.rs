use anyhow::Result;
use log::debug;
use serde_derive::{Deserialize, Serialize};
use std::env;
use std::fs::read_to_string;
use std::path::Path;

#[derive(Debug, Default, Serialize, Deserialize)]
pub(crate) struct Cfg {
    pub(crate) ignore_properties: Vec<String>,
    pub(crate) instance: Instance,
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
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct Instance {
    pub(crate) addr: String,
    pub(crate) user: String,
    pub(crate) pass: String,
}

impl Default for Instance {
    fn default() -> Self {
        Self {
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
    fn test_load_when_config_exists() -> Result<()> {
        // given
        let initial_dir = env::current_dir()?;
        let tmp_dir = TempDir::new()?;
        env::set_current_dir(tmp_dir.path())?;
        let mut cfg_file = File::create(".je")?;
        cfg_file.write_all(
            r#"ignore_properties = ["prop1", "prop2"]

[instance]
addr = "http://localhost:4502"
user = "user1"
pass = "pass1"
"#
            .as_bytes(),
        )?;

        // when
        let cfg = Cfg::load()?;

        // then
        assert_eq!(cfg.ignore_properties, vec!["prop1", "prop2"]);
        assert_eq!(cfg.instance.addr, "http://localhost:4502");
        assert_eq!(cfg.instance.user, "user1");
        assert_eq!(cfg.instance.pass, "pass1");

        env::set_current_dir(initial_dir)?;
        Ok(())
    }

    #[test]
    fn test_load_when_config_is_not_available() -> Result<()> {
        // when
        let cfg = Cfg::load()?;

        // then
        assert_eq!(cfg.ignore_properties, Vec::<String>::new());
        assert_eq!(cfg.instance.addr, "http://localhost:4502");
        assert_eq!(cfg.instance.user, "admin");
        assert_eq!(cfg.instance.pass, "admin");

        Ok(())
    }
}
