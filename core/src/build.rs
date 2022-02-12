use std::collections::HashSet;
use std::fmt::Debug;
use std::hash::{Hash, Hasher};

use error::Error;
use tracing::instrument;

use crate::{error, fs, oci, Arch, Artifact};

#[derive(PartialEq, Eq, Debug)]
pub struct Build {
  pub base: String,
  pub artifacts: HashSet<Artifact>,
  pub image: ImageName,
  pub arch: HashSet<Arch>,
}

#[derive(PartialEq, Eq, Debug)]
pub struct ImageName {
  pub image: String,
  pub tags: HashSet<String>,
}

#[derive(Debug)]
pub struct Command {
  pub fs: FileSystem,
  pub oci: Oci,
}

#[derive(Debug)]
pub struct FileSystem {
  pub resolver: Box<dyn fs::ArtifactResolver>,
  pub files: Box<dyn fs::Files>,
}

#[derive(Debug)]
pub struct Oci {
  pub registry: Box<dyn oci::Registry>,
  pub cache: Box<dyn oci::Cache>,
}

impl Command {
  #[instrument(ret, level = "trace")]
  pub fn execute(&self, b: &Build) -> Option<Error> {
    for artifact in &b.artifacts {
      let maybe_paths = self.fs.resolver.resolve(artifact);
      if maybe_paths.is_err() {
        return maybe_paths.err();
      }
      let paths = maybe_paths.unwrap();
      if paths.is_empty() {
        return Some(Error::invalid_input(
          format!("no artifact found: {}", artifact.from).as_str(),
        ));
      }
    }
    None
  }
}

impl Hash for Build {
  fn hash<H: Hasher>(&self, state: &mut H) {
    self.base.hash(state);
    self.image.hash(state);
    for artifact in &self.artifacts {
      artifact.hash(state)
    }
    for arch in &self.arch {
      arch.hash(state)
    }
  }
}

impl Hash for ImageName {
  fn hash<H: Hasher>(&self, state: &mut H) {
    self.image.hash(state);
    for tag in &self.tags {
      tag.hash(state)
    }
  }
}
