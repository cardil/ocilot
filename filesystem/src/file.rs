use std::fmt::Debug;
use std::fs::File;
use std::path::PathBuf;
use std::time::SystemTime;
use std::{fs, io};

use ocilot_core::fs::Files;
use tracing::instrument;

#[derive(Debug)]
pub struct LocalFileSystem {}

impl Files for LocalFileSystem {
  #[instrument(level = "trace")]
  fn read(&self, p: &PathBuf) -> io::Result<Box<dyn io::Read>> {
    File::options()
      .read(true)
      .write(false)
      .create(false)
      .open(p.as_path())
      .map(|f| Box::new(f) as Box<dyn io::Read>)
  }

  #[instrument(ret, level = "trace")]
  fn modified(&self, p: &PathBuf) -> io::Result<SystemTime> {
    let md = fs::metadata(p)?;
    md.modified()
  }
}
