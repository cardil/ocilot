use ocilot_core as core;
use std::path::PathBuf;

#[derive(Debug)]
pub struct ArtifactResolver {}

impl core::fs::ArtifactResolver for ArtifactResolver {
  fn resolve(&self, _: &core::Artifact) -> Vec<PathBuf> {
    todo!()
  }
}
