use crate::{cache, ImageName, OciImage};
use oci_distribution as oci;
use oci_distribution::Reference;
use ocilot_core as core;
use ocilot_core::error::{Error, Result};
use ocilot_core::oci::{Config, Construction, Image, Input, Registry};
use ocilot_core::Arch;
use std::collections::HashSet;
use std::future::Future;
use std::time::SystemTime;
use tokio::runtime::{Handle, Runtime};
use tracing::{instrument, warn};

#[derive(Debug)]
pub struct Rest {
  pub config: Box<dyn Config>,
}

impl Registry for Rest {
  #[instrument(ret, level = "trace")]
  fn fetch(&self, image_spec: &String) -> Result<Box<dyn Image>> {
    let config = oci::client::ClientConfig::default();
    let mut cli = oci::client::Client::new(config);
    let imageref = oci::Reference::try_from(image_spec.to_string())
      .map_err(|err| Error::invalid_input_from(Box::from(err)))?;
    let auth = oci::secrets::RegistryAuth::Anonymous;
    let accpected_media_types =
      Vec::from(["application/vnd.docker.image.rootfs.diff.tar.gzip"]);
    let image_data =
      block_on(cli.pull(&imageref, &auth, accpected_media_types))
        .map_err(|err| Error::Unexpected(Box::from(err)))?;
    let image = OciImage {
      data: image_data,
      name: imageref_to_imagename(imageref),
    };
    cache::persist_image(self.config.workdir()?, &image)?;
    Ok(Box::new(image) as Box<dyn Image>)
  }
}

fn block_on<F: Future>(future: F) -> F::Output {
  let (handle, _rt) = get_runtime_handle();
  handle.block_on(future)
}

fn get_runtime_handle() -> (Handle, Option<Runtime>) {
  match Handle::try_current() {
    Ok(h) => (h, None),
    Err(_) => {
      let rt = Runtime::new().unwrap();
      (rt.handle().clone(), Some(rt))
    }
  }
}

#[instrument(ret, level = "trace")]
fn imageref_to_imagename(imageref: Reference) -> ImageName {
  let mut registry = imageref.registry();
  const DOCKER_REGISTRY: &str = "docker.io";
  if registry == "" {
    registry = DOCKER_REGISTRY;
  }
  let mut repo = imageref.repository().to_string();
  if !repo.contains('/') && registry == DOCKER_REGISTRY {
    repo = format!("library/{}", repo);
  }
  let tags = match imageref.tag() {
    None => Vec::from(["latest".to_string()]),
    Some(tag) => Vec::from([tag.to_string()]),
  };
  ImageName {
    image: format!("{}/{}", registry, repo),
    tags,
  }
}

impl Image for OciImage {
  fn digest(&self) -> String {
    crate::bare_digest(self.data.digest())
  }

  fn name(&self) -> core::build::ImageName {
    core::build::ImageName {
      image: self.name.image.to_string(),
      tags: HashSet::from_iter(self.name.tags.iter().map(|t| t.to_string())),
    }
  }

  fn created(&self) -> SystemTime {
    warn!("Not yet implemented!");
    SystemTime::UNIX_EPOCH
  }

  fn construct_new(&self, _: &HashSet<Arch>) -> Box<dyn Construction> {
    Box::new(OciImageConstruction {
      digest: self.digest(),
    })
  }
}

#[derive(Debug)]
struct OciImageConstruction {
  digest: String,
}

impl Construction for OciImageConstruction {
  #[instrument(ret, level = "trace")]
  fn add(&self, _: Vec<Input>) {
    warn!("Not yet implemented!");
  }

  #[instrument(ret, level = "trace")]
  fn build(
    &self,
    name: &ocilot_core::build::ImageName,
  ) -> Result<Box<dyn Image>> {
    warn!("Not yet implemented!");
    Ok(Box::from(OciImage {
      data: oci_distribution::client::ImageData {
        layers: vec![],
        digest: Some(self.digest.to_string()),
        config: oci_distribution::client::Config {
          data: vec![],
          media_type: "Not yet implemented!".to_string(),
        },
        manifest: None,
      },
      name: ImageName {
        image: name.image.to_string(),
        tags: name.tags.iter().map(|t| t.to_string()).collect(),
      },
    }) as Box<dyn Image>)
  }
}
