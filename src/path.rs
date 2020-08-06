use anyhow::Result;
use std::path::Path as OsPath;

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

    pub(crate) fn is_dir(&self) -> bool {
        OsPath::new(&self.0).is_dir()
    }

    pub(crate) fn parent(&self) -> Result<String> {
        Ok(OsPath::new(&self.full())
            .parent()
            .unwrap_or(OsPath::new("/"))
            .display()
            .to_string())
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

    #[test]
    fn test_is_dir_on_dir() -> Result<()> {
        // given
        let tmp_dir = TempDir::new()?;

        // then
        assert_eq!(Path::new(tmp_dir.path().to_str().unwrap()).is_dir(), true);
        Ok(())
    }

    #[test]
    fn test_is_dir_on_file() -> Result<()> {
        // given
        let tmp_file = NamedTempFile::new()?;

        // then
        assert_eq!(Path::new(tmp_file.path().to_str().unwrap()).is_dir(), false);
        Ok(())
    }

    #[test]
    fn test_parent() -> Result<()> {
        // given
        let tmp_file = NamedTempFile::new()?;
        let path = Path::new(tmp_file.path().display().to_string());

        // when
        let path = path.parent()?;

        // then
        assert_eq!(
            path,
            tmp_file.path().parent().unwrap().display().to_string()
        );
        Ok(())
    }

    #[test]
    fn test_parent_on_root() -> Result<()> {
        // given
        let root = OsPath::new("/");
        let path = Path::new(root.display().to_string());

        // when
        let path = path.parent()?;

        // then
        assert_eq!(path, "/");
        Ok(())
    }
}
