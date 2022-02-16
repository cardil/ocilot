use std::collections::HashSet;
use std::fmt::Debug;
use std::hash::{Hash, Hasher};
use std::path::PathBuf;
use std::time::SystemTime;

use tracing::{info, instrument};

use crate::error::{Error, Result};
use crate::oci::Input;
use crate::{fs, oci, Arch, Artifact};

#[derive(PartialEq, Eq, Debug)]
pub struct Build {
  pub base: String,
  pub artifacts: HashSet<Artifact>,
  pub image: ImageName,
  pub arch: HashSet<Arch>,
}

#[derive(PartialEq, Eq, Debug, Clone)]
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

#[derive(PartialEq, Eq, Debug)]
struct Payload {
  parts: Vec<Part>,
}

#[derive(PartialEq, Eq, Debug)]
struct Part {
  arch: Option<Arch>,
  from: PathBuf,
  to: Option<String>,
}

#[derive(PartialEq, Eq, Debug)]
pub struct ImageInfo {
  pub digest: String,
}

#[derive(PartialEq, Eq, Debug)]
pub enum Built {
  Cached(ImageInfo),
  Real(ImageInfo),
}

impl Command {
  #[instrument(ret, level = "trace")]
  pub fn execute(&self, b: &Build) -> Result<Built> {
    let payload = self.construct_payload(b)?;
    let already_built = self.lookup_built(&payload, &b.image)?;
    if already_built.is_some() {
      let im = already_built.unwrap();
      return Ok(Built::Cached(ImageInfo {
        digest: im.digest(),
      }));
    }
    let base = self.oci.registry.fetch(&b.base)?;
    info!(digest = ?base.digest(), "Base image fetched");
    let inputs = self.open_payload(payload)?;
    let constr = base.construct_new(&b.arch);
    constr.add(inputs);
    let built = constr.build(&b.image)?;

    Ok(Built::Real(ImageInfo {
      digest: built.digest(),
    }))
  }

  fn open_payload(&self, payload: Payload) -> Result<Vec<Input>> {
    let mut files = Vec::new();
    for part in payload.parts {
      let input = self.fs.files.read(&part.from).map(|read| Input {
        arch: part.arch.clone(),
        from: read,
        to: part.to.clone(),
      })?;
      files.push(input);
    }
    Ok(files)
  }

  #[instrument(ret, level = "trace")]
  fn construct_payload(&self, b: &Build) -> Result<Payload> {
    let mut parts = Vec::new();
    for artifact in &b.artifacts {
      let maybe_paths = self.fs.resolver.resolve(artifact);
      match maybe_paths {
        Ok(paths) => {
          if paths.is_empty() {
            return Err(Error::invalid_input(
              format!("no artifact found: {}", artifact.from).as_str(),
            ));
          }
          paths
            .iter()
            .map(|p| Part {
              arch: artifact.arch.clone(),
              from: p.to_owned(),
              to: artifact.to.clone(),
            })
            .for_each(|p| parts.push(p))
        }
        Err(err) => return Err(err),
      };
    }
    Ok(Payload { parts })
  }

  #[instrument(ret, level = "trace")]
  fn lookup_built(
    &self,
    payload: &Payload,
    im: &ImageName,
  ) -> Result<Option<Box<dyn oci::Image>>> {
    let images = self.oci.cache.list()?;
    for image in images {
      if image.name() == *im {
        let mt = self.payload_modtime(payload)?;
        if image.created() >= mt {
          return Ok(Some(image));
        }
      }
    }
    Ok(None)
  }

  fn payload_modtime(&self, payload: &Payload) -> Result<SystemTime> {
    let mut newest = SystemTime::UNIX_EPOCH;
    for part in &payload.parts {
      let mt = self.fs.files.modified(&part.from)?;
      if mt > newest {
        newest = mt;
      }
    }
    if newest == SystemTime::UNIX_EPOCH {
      return Err(Error::Bug(
        concat!(
          "possible bug. no modification date ",
          "for any artifact was found"
        )
        .to_string(),
      ));
    }
    Ok(newest)
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
