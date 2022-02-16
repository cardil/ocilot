use crate::error::Result;
use crate::{build, Arch};
use std::collections::HashSet;
use std::fmt::Debug;
use std::path::PathBuf;
use std::{io, time};

pub trait Config: Debug {
  fn workdir(&self) -> Result<PathBuf>;
}

pub trait Registry: Debug {
  fn fetch(&self, image: &String) -> Result<Box<dyn Image>>;
}

pub trait Cache: Debug {
  fn list(&self) -> Result<Vec<Box<dyn Image>>>;
}

pub trait Image: Debug {
  fn digest(&self) -> String;
  fn name(&self) -> build::ImageName;
  fn created(&self) -> time::SystemTime;
  fn construct_new(&self, archs: &HashSet<Arch>) -> Box<dyn Construction>;
}

pub trait Construction {
  fn add(&self, files: Vec<Input>);
  fn build(&self, named: &build::ImageName) -> Result<Box<dyn Image>>;
}

pub struct Input {
  pub arch: Option<Arch>,
  pub from: Box<dyn io::Read>,
  pub to: Option<String>,
}
