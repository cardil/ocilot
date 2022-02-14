use crate::{build, error, Arch};
use std::collections::HashSet;
use std::fmt::Debug;
use std::{io, time};

pub trait Registry: Debug {
  fn fetch(&self, image: String) -> error::Result<Box<dyn Image>>;
}

pub trait Cache: Debug {
  fn list(&self) -> error::Result<Vec<Box<dyn Image>>>;
}

pub trait Image: Debug {
  fn digest(&self) -> String;
  fn name(&self) -> build::ImageName;
  fn created(&self) -> time::SystemTime;
  fn construct_new(&self, archs: HashSet<Arch>) -> dyn Construction;
}

pub trait Construction {
  fn add(&self, files: Vec<Input>) -> dyn Construction;
  fn build(&self, named: build::ImageName) -> dyn Image;
}

pub struct Input {
  pub arch: Option<Arch>,
  pub from: Box<dyn io::Read>,
  pub to: Option<String>,
}
