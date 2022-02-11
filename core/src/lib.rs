pub mod build;
pub mod error;
pub mod fs;

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
