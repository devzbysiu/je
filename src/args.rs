use crate::cfg::{Cfg, Instance};
use crate::cmd::Opt;
use crate::path::Path;
use getset::{CopyGetters, Getters};

#[derive(Debug, Getters, CopyGetters, Default, Clone)]
pub(crate) struct GetArgs {
    #[getset(get = "pub")]
    path: Path,
    #[getset(get = "pub")]
    instance: Instance,
    #[getset(get_copy = "pub")]
    debug: bool,
    #[getset(get = "pub")]
    ignore_properties: Vec<String>,
}

impl GetArgs {
    pub(crate) fn new<S: Into<String>>(path: S, cfg: Cfg, opt: &Opt) -> Self {
        Self {
            path: Path::new(path),
            instance: cfg.instance(opt.profile.as_ref()),
            debug: opt.debug,
            ignore_properties: cfg.ignore_properties,
        }
    }
}

#[derive(Debug, CopyGetters, Getters, Default, Clone)]
pub(crate) struct PutArgs {
    #[getset(get = "pub")]
    path: Path,
    #[getset(get = "pub")]
    instance: Instance,
    #[getset(get_copy = "pub")]
    debug: bool,
}

impl PutArgs {
    pub(crate) fn new<S: Into<String>>(path: S, cfg: &Cfg, opt: &Opt) -> Self {
        Self {
            path: Path::new(path),
            instance: cfg.instance(opt.profile.as_ref()),
            debug: opt.debug,
        }
    }
}
