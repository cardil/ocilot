pub mod build;
pub mod error;
pub mod fs;
pub mod oci;

#[derive(PartialEq, Eq, Debug, Hash, Clone)]
pub struct Artifact {
  pub arch: Option<Arch>,
  pub from: String,
  pub to: Option<String>,
}

#[derive(PartialEq, Eq, Debug, Hash, Clone)]
pub enum Arch {
  Amd64,
  Arm64,
  Ppc64le,
  S390x,
}
