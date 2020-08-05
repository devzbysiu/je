pub(crate) struct Path(String);

impl Path {
    pub(crate) fn new<S: Into<String>>(path: S) -> Self {
        Path(path.into())
    }

    pub(crate) fn content(&self) -> String {
        let path = &self.0;
        let parts: Vec<&str> = path.split("jcr_root").collect();
        assert_eq!(parts.len(), 2);
        parts[1].into()
    }

    pub(crate) fn full(&self) -> String {
        self.0.clone()
    }
}
