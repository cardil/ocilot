use std::fmt::Debug;
use std::fs::File;
use std::io;
use std::path::PathBuf;

use ocilot_core::fs::Files;

#[derive(Debug)]
pub struct LocalFileSystem {}

impl Files for LocalFileSystem {
  fn read(&self, p: &PathBuf) -> io::Result<Box<dyn io::Read>> {
    File::options()
      .read(true)
      .write(false)
      .create(false)
      .open(p.as_path())
      .map(|f| Box::new(f) as Box<dyn io::Read>)
  }
}
