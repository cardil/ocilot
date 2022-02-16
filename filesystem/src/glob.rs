use std::io;
use std::path::{Path, PathBuf};
use tracing::instrument;

use ocilot_core as core;
use ocilot_core::error::{Error, Result};
use wax;
use wax::{Glob, GlobError};

#[derive(Debug)]
pub struct ArtifactResolver {}

impl core::fs::ArtifactResolver for ArtifactResolver {
  #[instrument(ret, level = "trace")]
  fn resolve(&self, art: &core::Artifact) -> Result<Vec<PathBuf>> {
    let from = art.from.as_str();
    let path = Path::new(from);
    if path.exists() && path.is_file() {
      return Ok(vec![path.to_path_buf()]);
    }

    match wax::Glob::new(from) {
      Ok(glob) => match resolve_glob(glob) {
        Ok(paths) => Ok(paths),
        Err(err) => Err(Error::Unexpected(Box::new(err))),
      },
      Err(gerr) => match gerr {
        GlobError::Parse(perr) => Err(parse_error_as_core(perr)),
        GlobError::Rule(rerr) => Err(rule_error_as_core(rerr)),
        GlobError::Walk(werr) => Err(Error::Unexpected(Box::new(werr))),
        _ => panic!("can't get here"),
      },
    }
  }
}

fn parse_error_as_core(err: wax::ParseError) -> Error {
  Error::InvalidInput {
    message: format!("{}", err.expression()),
    cause: Some(Box::new(err.into_owned()) as Box<dyn std::error::Error>),
  }
}

fn rule_error_as_core(err: wax::RuleError) -> Error {
  Error::InvalidInput {
    message: format!("{}", err.expression()),
    cause: Some(Box::new(err.into_owned()) as Box<dyn std::error::Error>),
  }
}

fn resolve_glob(glob: Glob) -> io::Result<Vec<PathBuf>> {
  let cwd = std::env::current_dir()?;
  let mut entries = vec![];
  for maybe_entry in glob.walk(cwd, usize::MAX) {
    let entry = maybe_entry?;
    let path = entry.path();
    if path.is_file() {
      entries.push(path.to_path_buf());
    }
  }
  Ok(entries)
}
