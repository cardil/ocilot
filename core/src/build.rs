use std::collections::HashSet;
use std::fmt::Debug;
use std::hash::{Hash, Hasher};
use std::path::PathBuf;
use std::time::SystemTime;

use error::Error;
use tracing::{instrument, warn};

use crate::oci::Image;
use crate::{error, fs, oci, Arch, Artifact};

#[derive(PartialEq, Eq, Debug)]
pub struct Build {
  pub base: String,
  pub artifacts: HashSet<Artifact>,
  pub image: ImageName,
  pub arch: HashSet<Arch>,
}

#[derive(PartialEq, Eq, Debug)]
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

impl Payload {
  fn modification(&self, files: &Box<dyn fs::Files>) -> SystemTime {
    let mut newest = SystemTime::UNIX_EPOCH;
    for part in &self.parts {
      let mt = files
        .modified(&part.from)
        .expect("couldn't resolve part mod time");
      if mt > newest {
        newest = mt;
      }
    }
    if newest == SystemTime::UNIX_EPOCH {
      panic!("possible bug. no modification date for any artifact was found");
    }
    newest
  }
}

#[derive(PartialEq, Eq, Debug)]
struct Part {
  arch: Option<Arch>,
  from: PathBuf,
  to: Option<String>,
}

impl Command {
  #[instrument(ret, level = "trace")]
  pub fn execute(&self, b: &Build) -> error::Result<Box<dyn Image>> {
    let payload = self.build_payload(b)?;
    let already_built = self.lookup_built(&payload, &b.image);
    if already_built.is_some() {
      return Ok(already_built.unwrap());
    }

    Err(error::Error::invalid_input("AAAAA"))
  }

  #[instrument(ret, level = "trace")]
  fn build_payload(&self, b: &Build) -> error::Result<Payload> {
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
  fn lookup_built(&self, payload: &Payload, im: &ImageName) -> Option<Box<dyn oci::Image>> {
    let result = self.oci.cache.list();
    let images = result.unwrap_or_else(|err| {
      warn!(cause = ?err, "Couldn't list local cache");
      Vec::default()
    });
    for image in images {
      if image.name() == *im {
        if image.created() >= payload.modification(&self.fs.files) {
          return Some(image);
        }
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
