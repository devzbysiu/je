use anyhow::Result;
use std::env;
use std::fs::{read_to_string, File};
use std::io::prelude::*;
use std::path::PathBuf;
use tempfile::TempDir;

pub(crate) struct TestConfig {
    initial_dir: PathBuf,
    tmp_dir: TempDir,
}

impl TestConfig {
    pub(crate) fn new() -> Result<Self> {
        Ok(TestConfig {
            initial_dir: env::current_dir()?,
            tmp_dir: TempDir::new()?,
        })
    }

    pub(crate) fn write_all<S: Into<String>>(&self, content: S) -> Result<()> {
        env::set_current_dir(self.tmp_dir.path())?;
        let mut cfg_file = File::create(".je")?;
        cfg_file.write_all(content.into().as_bytes())?;
        Ok(())
    }

    pub(crate) fn read_all(&self) -> Result<String> {
        env::set_current_dir(self.tmp_dir.path())?;
        Ok(read_to_string(".je")?)
    }
}

impl Drop for TestConfig {
    fn drop(&mut self) {
        env::set_current_dir(self.initial_dir.clone()).expect("failed to change to an initial dir");
    }
}
