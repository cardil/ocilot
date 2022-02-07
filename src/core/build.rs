use std::collections::HashSet;
use std::hash::{Hash, Hasher};
use std::option::Option;

#[derive(PartialEq, Eq, Debug)]
pub struct Build {
  pub base: String,
  pub artifacts: HashSet<Artifact>,
  pub image: String,
  pub arch: HashSet<Arch>,
  pub tags: HashSet<String>,
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

#[derive(PartialEq, Eq, Debug, Hash)]
pub struct Artifact {
  pub arch: Option<Arch>,
  pub from: String,
  pub to: String,
}

#[derive(PartialEq, Eq, Debug, Hash)]
pub enum Arch {
  Amd64,
  Arm64,
  Ppc64le,
  S390x,
}
