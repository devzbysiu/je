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
