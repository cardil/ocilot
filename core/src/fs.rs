use crate::{error, Artifact};
use std::fmt::Debug;
use std::io;
use std::path::PathBuf;
use std::time::SystemTime;

pub trait ArtifactResolver: Debug {
  fn resolve(&self, ar: &Artifact) -> error::Result<Vec<PathBuf>>;
}

pub trait Files: Debug {
  fn read(&self, p: &PathBuf) -> io::Result<Box<dyn io::Read>>;
  fn modified(&self, p: &PathBuf) -> io::Result<SystemTime>;
}
