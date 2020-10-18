use crate::cfg::{Cfg, Instance};
use crate::cmd::Opt;
use crate::path::Path;

pub(crate) struct GetArgs {
    common: Common,
    ignore_properties: Vec<String>,
}

struct Common {
    path: Path,
    instance: Instance,
    debug: bool,
}

impl GetArgs {
    pub(crate) fn new<S: Into<String>>(path: S, cfg: Cfg, opt: &Opt) -> Self {
        Self {
            common: Common {
                path: Path::new(path),
                instance: cfg.instance(opt.profile.as_ref()),
                debug: opt.debug,
            },
            ignore_properties: cfg.ignore_properties,
        }
    }

    pub(crate) fn path(&self) -> &Path {
        &self.common.path
    }

    pub(crate) fn instance(&self) -> &Instance {
        &self.common.instance
    }

    pub(crate) fn debug(&self) -> bool {
        self.common.debug
    }

    pub(crate) fn ignore_properties(&self) -> &[String] {
        &self.ignore_properties
    }
}

pub(crate) struct PutArgs {
    common: Common,
}

impl PutArgs {
    pub(crate) fn new<S: Into<String>>(path: S, cfg: &Cfg, opt: &Opt) -> Self {
        Self {
            common: Common {
                path: Path::new(path),
                instance: cfg.instance(opt.profile.as_ref()),
                debug: opt.debug,
            },
        }
    }

    pub(crate) fn path(&self) -> &Path {
        &self.common.path
    }

    pub(crate) fn instance(&self) -> &Instance {
        &self.common.instance
    }

    pub(crate) fn debug(&self) -> bool {
        self.common.debug
    }
}
