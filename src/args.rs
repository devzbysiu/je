use crate::cfg::{Bundle, Cfg, IgnoreProp, Instance};
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
    ignore_properties: Vec<IgnoreProp>,
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

#[derive(Debug, Getters, CopyGetters, Default, Clone)]
pub(crate) struct GetBundleArgs {
    #[getset(get = "pub")]
    bundle: Bundle,

    #[getset(get = "pub")]
    instance: Instance,

    #[getset(get_copy = "pub")]
    debug: bool,

    #[getset(get = "pub")]
    ignore_properties: Vec<IgnoreProp>,
}

impl GetBundleArgs {
    pub(crate) fn new<S: Into<String>>(name: S, cfg: Cfg, opt: &Opt) -> Self {
        Self {
            bundle: cfg.bundle(Some(&name.into())),
            instance: cfg.instance(opt.profile.as_ref()),
            debug: opt.debug,
            ignore_properties: cfg.ignore_properties,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cfg::IgnoreType;

    #[test]
    fn test_get_args_creation() {
        // given
        let path = "/some/path";
        let cfg = Cfg {
            ignore_properties: vec![IgnoreProp {
                ignore_type: IgnoreType::Contains,
                value: "some value".into(),
            }],
            profiles: vec![
                Instance::new("author-1", "http://localhost:4502", "admin", "admin"),
                Instance::new("publish-1", "http://localhost:4503", "admin", "admin"),
            ],
            ..Cfg::default()
        };
        let opt = Opt {
            debug: false,
            profile: Some("author-1".into()),
            ..Opt::default()
        };
        let expected = GetArgs {
            path: Path::new("/some/path"),
            instance: Instance::new("author-1", "http://localhost:4502", "admin", "admin"),
            debug: false,
            ignore_properties: vec![IgnoreProp {
                ignore_type: crate::cfg::IgnoreType::Contains,
                value: "some value".into(),
            }],
        };

        // when
        let get_args = GetArgs::new(path, cfg, &opt);

        // then
        assert_eq!(expected.path.full(), get_args.path.full());
        assert_eq!(expected.instance, get_args.instance);
        assert_eq!(expected.debug, get_args.debug);
        assert_eq!(expected.ignore_properties, get_args.ignore_properties);
    }

    #[test]
    fn test_put_args_creation() {
        // given
        let path = "/some/path";
        let cfg = Cfg {
            profiles: vec![
                Instance::new("int-author", "http://localhost:4502", "admin", "admin"),
                Instance::new("int-publish", "http://localhost:4503", "admin", "admin"),
            ],
            ..Cfg::default()
        };
        let opt = Opt {
            debug: true,
            profile: Some("int-publish".into()),
            ..Opt::default()
        };
        let expected = PutArgs {
            path: Path::new("/some/path"),
            instance: Instance::new("int-publish", "http://localhost:4503", "admin", "admin"),
            debug: true,
        };

        // when
        let actual = PutArgs::new(path, &cfg, &opt);

        // then
        assert_eq!(expected.path.full(), actual.path.full());
        assert_eq!(expected.instance, actual.instance);
        assert_eq!(expected.debug, actual.debug);
    }

    #[test]
    fn test_get_bundle_args_creation() {
        // given
        let bundle_name = "test-bundle";
        let cfg = Cfg {
            ignore_properties: vec![IgnoreProp {
                ignore_type: IgnoreType::Contains,
                value: "other value".into(),
            }],
            bundles: Some(vec![
                Bundle::new("test-bundle", vec!["/some/file"]),
                Bundle::new("other", vec!["/different/file"]),
            ]),
            profiles: vec![
                Instance::new("prod-author", "http://localhost:4502", "admin", "admin"),
                Instance::new("prod-publish", "http://localhost:4503", "admin", "admin"),
            ],
            ..Cfg::default()
        };
        let opt = Opt {
            debug: false,
            profile: Some("prod-author".into()),
            ..Opt::default()
        };
        let expected = GetBundleArgs {
            bundle: Bundle::new("test-bundle", vec!["/some/file"]),
            instance: Instance::new("prod-author", "http://localhost:4502", "admin", "admin"),
            debug: false,
            ignore_properties: vec![IgnoreProp {
                ignore_type: crate::cfg::IgnoreType::Contains,
                value: "other value".into(),
            }],
        };

        // when
        let actual = GetBundleArgs::new(bundle_name, cfg, &opt);

        // then
        assert_eq!(expected.bundle, actual.bundle);
        assert_eq!(expected.instance, actual.instance);
        assert_eq!(expected.debug, actual.debug);
        assert_eq!(expected.ignore_properties, actual.ignore_properties);
    }
}
