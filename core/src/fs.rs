use crate::{error, Artifact};
use std::fmt::Debug;
use std::io;
use std::path::PathBuf;

pub trait ArtifactResolver: Debug {
  fn resolve(&self, ar: &Artifact) -> Result<Vec<PathBuf>, error::Error>;
}

pub trait Files: Debug {
  fn read(&self, p: &PathBuf) -> io::Result<Box<dyn io::Read>>;
}
