use std::collections::HashSet;
use std::hash::Hash;
use std::option::Option;

use wax::Glob;

#[derive(Debug)]
pub struct Build<'t> {
  pub base: String,
  pub artifacts: HashSet<Artifact<'t>>,
  pub image: String,
  pub arch: HashSet<Arch>,
  pub tags: HashSet<String>,
}

impl Eq for Build<'_> {}

impl PartialEq for Build<'_> {
  fn eq(&self, other: &Self) -> bool {
    return
      self.base == other.base &&
        self.artifacts == other.artifacts &&
        self.image == other.image &&
        self.arch == other.arch &&
        self.tags == other.tags;
  }
}

#[derive(Debug)]
pub struct Artifact<'t> {
  pub arch: Option<Arch>,
  pub from: Glob<'t>,
  pub to: String,
}

impl Artifact<'_> {
  fn repr_of_from(&self) -> String {
    return format!("{:?}", self.from);
  }
}

impl Hash for Artifact<'_> {
  fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
    self.arch.hash(state);
    self.to.hash(state);
    self.repr_of_from().hash(state);
  }
}

impl Eq for Artifact<'_> {}

impl PartialEq for Artifact<'_> {
  fn eq(&self, other: &Self) -> bool {
    return self.arch == other.arch &&
      self.to == other.to &&
      self.repr_of_from() == other.repr_of_from();
  }
}

#[derive(PartialEq, Eq, Debug, Hash)]
pub enum Arch {
  Amd64,
  Arm64,
  Ppc64le,
  S390x,
}
