use error::Error;
use std::collections::HashSet;
use std::fmt::Debug;
use std::hash::{Hash, Hasher};

use tracing::instrument;

use crate::{error, fs, Arch, Artifact};

#[derive(PartialEq, Eq, Debug)]
pub struct Build {
  pub base: String,
  pub artifacts: HashSet<Artifact>,
  pub image: String,
  pub arch: HashSet<Arch>,
  pub tags: HashSet<String>,
}

#[derive(Debug)]
pub struct Command {
  pub artifact_resolver: Box<dyn fs::ArtifactResolver>,
  pub files: Box<dyn fs::Files>,
}

impl Command {
  #[instrument]
  pub fn execute(&self, b: &Build) -> Option<Error> {
    for artifact in &b.artifacts {
      let paths = self.artifact_resolver.resolve(artifact);
      if paths.is_empty() {
        return Some(Error::invalid_input("no Artifact found!"));
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
    for tag in &self.tags {
      tag.hash(state)
    }
  }
}
