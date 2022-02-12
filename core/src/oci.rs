use crate::build::ImageName;
use crate::{error, Arch};
use std::collections::HashSet;
use std::fmt::Debug;
use std::io;

pub trait Registry: Debug {
  fn fetch(&self, image: String) -> Result<Box<dyn Image>, error::Error>;
}

pub trait Cache: Debug {
  fn list(&self) -> Result<Vec<Box<dyn Image>>, error::Error>;
}

pub trait Image {
  fn digest(&self) -> String;
  fn tags(&self) -> Vec<String>;
  fn build_upon(&self, archs: HashSet<Arch>) -> dyn Construction;
}

pub trait Construction {
  fn add(&self, files: Vec<Input>) -> dyn Construction;
  fn build(&self, named: ImageName) -> dyn Image;
}

pub struct Input {
  pub arch: Option<Arch>,
  pub from: Box<dyn io::Read>,
  pub to: Option<String>,
}
