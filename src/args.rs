use crate::cfg::Instance;
use crate::path::Path;

pub(crate) struct GetArgs {
    pub(crate) path: Path,
    pub(crate) instance: Instance,
    pub(crate) debug: bool,
    pub(crate) ignore_properties: Vec<String>,
}

pub(crate) struct PutArgs {
    pub(crate) path: Path,
    pub(crate) instance: Instance,
    pub(crate) debug: bool,
}
