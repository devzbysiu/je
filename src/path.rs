use std::path::Path as OsPath;

#[derive(Debug, Clone, Default)]
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

    pub(crate) fn from_root(&self) -> String {
        let path = &self.0;
        let parts: Vec<&str> = path.split("jcr_root").collect();
        assert_eq!(parts.len(), 2);
        format!("jcr_root{}", parts[1])
    }

    pub(crate) fn is_dir(&self) -> bool {
        OsPath::new(&self.0).is_dir()
    }

    pub(crate) fn parent_from_root(&self) -> String {
        let parent = OsPath::new(&self.full())
            .parent()
            .unwrap_or_else(|| OsPath::new("/"))
            .display()
            .to_string();
        let parts: Vec<&str> = parent.split("jcr_root").collect();
        assert_eq!(parts.len(), 2);
        let path: String = parts[1].into();
        format!("jcr_root{}", path)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use anyhow::Result;
    use tempfile::{NamedTempFile, TempDir};

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
    fn test_from_root_with_correct_path() {
        // given
        let path = Path::new("/home/zbychu/project/test/jcr_root/content/abc");

        // when
        let path = path.from_root();

        // then
        assert_eq!(path, "jcr_root/content/abc");
    }

    #[test]
    #[should_panic]
    fn test_from_root_with_wrong_path() {
        // given
        let path = Path::new("/home/zbychu/project/test");

        // should panic
        path.from_root();
    }

    #[test]
    fn test_is_dir_on_dir() -> Result<()> {
        // given
        let tmp_dir = TempDir::new()?;

        // then
        assert!(Path::new(tmp_dir.path().to_str().unwrap()).is_dir());
        Ok(())
    }

    #[test]
    fn test_is_dir_on_file() -> Result<()> {
        // given
        let tmp_file = NamedTempFile::new()?;

        // then
        assert!(!Path::new(tmp_file.path().to_str().unwrap()).is_dir());
        Ok(())
    }

    #[test]
    fn test_parent() {
        // given
        let full_path = OsPath::new("/home/zbychu/jcr_root/content/test");
        let path = Path::new(full_path.display().to_string());

        // when
        let path = path.parent_from_root();

        // then
        assert_eq!(path, "jcr_root/content");
    }

    #[test]
    #[should_panic]
    fn test_parent_on_root() {
        // given
        let root = OsPath::new("/");
        let path = Path::new(root.display().to_string());

        // should_panic
        path.parent_from_root();
    }
}
