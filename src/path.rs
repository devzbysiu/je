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

    pub(crate) fn with_root(&self) -> String {
        let path = &self.0;
        let parts: Vec<&str> = path.split("jcr_root").collect();
        assert_eq!(parts.len(), 2);
        format!("jcr_root{}", parts[1])
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_content_path_with_correct_path() {
        // given
        let path = Path::new("/home/zbychu/project/test/jcr_root/content/abc");

        // when
        let content_path = path.content();

        // then
        assert_eq!(content_path, "/content/abc");
    }

    #[test]
    #[should_panic]
    fn test_content_path_with_broken_path() {
        // given
        let path = Path::new("/home/zbychu/project/test/content/abc");

        // should panic
        path.content();
    }

    #[test]
    fn test_full_path() {
        // given
        let path = Path::new("/home/zbychu/project/test/jcr_root/content/abc");

        // when
        let full_path = path.full();

        // then
        assert_eq!(full_path, "/home/zbychu/project/test/jcr_root/content/abc");
    }

    #[test]
    fn test_with_root_with_correct_path() {
        // given
        let path = Path::new("/home/zbychu/project/test/jcr_root/content/abc");

        // when
        let path = path.with_root();

        // then
        assert_eq!(path, "jcr_root/content/abc");
    }

    #[test]
    #[should_panic]
    fn test_with_root_with_wrong_path() {
        // given
        let path = Path::new("/home/zbychu/project/test");

        // should panic
        path.with_root();
    }
}
