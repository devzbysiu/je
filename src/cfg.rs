use crate::cfgmgr::VERSION;
use getset::Getters;
use serde_derive::{Deserialize, Serialize};
use std::convert::Into;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct Cfg {
    pub(crate) version: Option<String>,

    pub(crate) ignore_properties: Vec<IgnoreProp>,

    #[serde(rename = "profile")]
    pub(crate) profiles: Vec<Instance>,

    #[serde(rename = "bundle")]
    pub(crate) bundles: Option<Vec<Bundle>>,
}

impl Cfg {
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
                .unwrap_or_default(),
            None => Bundle::default(),
        }
    }
}

impl Default for Cfg {
    fn default() -> Self {
        Self {
            version: Some(VERSION.to_string()),
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
    pub(crate) fn new<S: Into<String>>(name: S, addr: S, user: S, pass: S) -> Self {
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
    paths: Vec<String>,
}

impl Bundle {
    #[allow(dead_code)] // TODO: remove this
    pub(crate) fn new<S: Into<String>>(name: S, files: Vec<S>) -> Self {
        Self {
            name: name.into(),
            paths: files.into_iter().map(Into::into).collect(),
        }
    }
}

#[derive(Getters, Default, Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub(crate) struct IgnoreProp {
    #[serde(rename = "type")]
    pub(crate) ignore_type: IgnoreType,

    pub(crate) value: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub(crate) enum IgnoreType {
    Contains,
    Regex,
}

impl Default for IgnoreType {
    fn default() -> Self {
        IgnoreType::Contains
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_cfg_default() {
        // given
        let expected_cfg = Cfg {
            version: Some("0.3.0".to_string()),
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
}
