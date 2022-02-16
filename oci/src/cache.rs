use ocilot_core::error::{Error, Result};
use ocilot_core::oci;
use ocilot_core::oci::{Config, Image};
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use tracing::{debug, instrument, warn};

#[derive(Debug)]
pub struct HomeBased {
  pub config: Box<dyn Config>,
}

impl oci::Cache for HomeBased {
  #[instrument(ret, level = "trace")]
  fn list(&self) -> Result<Vec<Box<dyn Image>>> {
    warn!("Not yet implemented");
    Ok(Vec::new())
  }
}

#[instrument(ret, level = "trace")]
pub(crate) fn persist_image(
  workdir: PathBuf,
  image: &crate::OciImage,
) -> Result<()> {
  let digest = image.digest();
  let (prefix, rest) = digest.split_at(3);
  let imdir = workdir.join("images").join(prefix).join(rest);
  fs::create_dir_all(&imdir)?;

  for layer in &image.data.layers {
    let digest = crate::bare_digest(layer.sha256_digest());
    let new_layer_path = imdir.join(&digest);
    let mut new_layer_file = File::create(&new_layer_path)?;
    new_layer_file.write_all(&(layer.data).as_slice())?;
    new_layer_file.flush()?;
    debug!(layer = ?digest, "Layer cached");
  }
  let manifest_path = imdir.join("manifest.json");
  let manifest = image
    .data
    .manifest
    .as_ref()
    .ok_or(Error::Bug("no image manifest".to_string()))?;
  let manifest_cnt = serde_json::to_string(manifest)
    .map_err(|err| Error::Unexpected(Box::from(err)))?;
  let mut manifest_file = File::create(&manifest_path)?;
  manifest_file.write_all(manifest_cnt.as_bytes())?;
  manifest_file.flush()?;
  debug!(manifest = ?manifest_path, "Manifest cached");
  let config_path =
    imdir.join(crate::bare_digest(manifest.config.digest.to_string()));
  let mut config_file = File::create(&config_path)?;
  config_file.write_all(image.data.config.data.as_slice())?;
  config_file.flush()?;
  debug!(config = ?config_path, "Config cached");
  let mut version_file = File::create(imdir.join("version"))?;
  version_file.write_all("Directory Transport Version: 1.1\n".as_bytes())?;
  version_file.flush()?;
  debug!(image = ?digest, "Image cached");
  Ok(())
}
