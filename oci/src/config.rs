use ocilot_core::oci;
use std::path::PathBuf;

#[derive(Debug)]
pub struct Config {
  pub workdir: PathBuf,
}

impl oci::Config for Config {
  fn workdir(&self) -> ocilot_core::error::Result<PathBuf> {
    Ok(self.workdir.clone())
  }
}
