use oci_distribution as oci;
use std::fmt::Formatter;
pub mod cache;
pub mod config;
pub mod registry;

#[derive(PartialEq, Eq, Debug, Clone)]
pub(crate) struct ImageName {
  pub(crate) image: String,
  pub(crate) tags: Vec<String>,
}

#[derive(Clone)]
pub(crate) struct OciImage {
  pub(crate) data: oci::client::ImageData,
  pub(crate) name: ImageName,
}

impl std::fmt::Debug for OciImage {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("OciImage")
      .field("name", &self.name)
      .field("data(digest)", &self.data.digest())
      .finish_non_exhaustive()
  }
}

pub(crate) fn bare_digest(digest: String) -> String {
  digest
    .strip_prefix("sha256:")
    .unwrap_or(digest.as_str())
    .to_string()
}
