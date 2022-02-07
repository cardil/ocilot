use io::{Error, ErrorKind};
use std::collections::HashSet;
use std::hash::Hash;
use std::io;
use std::option::Option;

use regex::RegexBuilder;
use wax::Glob;

#[derive(Debug)]
pub struct Build<'t> {
  pub base: String,
  pub artifacts: HashSet<Artifact<'t>>,
  pub image: String,
  pub arch: HashSet<Arch>,
  pub tags: HashSet<String>,
}

impl Eq for Build<'_> {}

impl PartialEq for Build<'_> {
  fn eq(&self, other: &Self) -> bool {
    return
      self.base == other.base &&
        self.artifacts == other.artifacts &&
        self.image == other.image &&
        self.arch == other.arch &&
        self.tags == other.tags;
  }
}

#[derive(Debug)]
pub struct Artifact<'t> {
  pub arch: Option<Arch>,
  pub from: Glob<'t>,
  pub to: String,
}

impl Artifact<'_> {
  pub fn from_string(repr: &'static String) -> Result<Artifact, Error> {
    // Ref.: https://regex101.com/r/q2qVXt/1
    let raw_re = r"^(?:(?P<arch>[^\n:]+):)?(?P<from>[^\n:]+)(?::(?P<to>[^\n:]+))?$";
    let re = RegexBuilder::new(raw_re)
      .swap_greed(true)
      .build()
      .unwrap();
    let empty = Artifact {
      arch: None,
      from: Glob::new("").unwrap(),
      to: "".to_string(),
    };
    match re.captures(repr.as_str()) {
      None => Result::Err(Error::new(
        ErrorKind::InvalidInput, repr.as_str(),
      )),
      Some(cap) => {
        let from_result = cap.name("from").ok_or(Error::new(
          ErrorKind::InvalidInput, repr.as_str(),
        )).map(|m| m.as_str());
        if from_result.is_err() {
          return from_result.map(|_| empty);
        }
        let from = from_result.unwrap();
        let to = match cap.name("to") {
          None => from,
          Some(m) => m.as_str()
        };
        let arch = match cap.name("arch") {
          None => Option::None,
          Some(m) => Option::Some(Arch::from_string(&m.as_str().to_string()))
        }.transpose();
        if arch.is_err() {
          return arch.map(|_| empty);
        }

        Glob::new(from)
          .map_err(|err| Error::new(ErrorKind::InvalidInput, err))
          .map(|glob| Artifact {
            arch: arch.unwrap(),
            from: glob,
            to: to.to_string(),
          })
      }
    }
  }

  fn repr_of_from(&self) -> String {
    return format!("{:?}", self.from);
  }
}

impl Hash for Artifact<'_> {
  fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
    self.arch.hash(state);
    self.to.hash(state);
    self.repr_of_from().hash(state);
  }
}

impl Eq for Artifact<'_> {}

impl PartialEq for Artifact<'_> {
  fn eq(&self, other: &Self) -> bool {
    return self.arch == other.arch &&
      self.to == other.to &&
      self.repr_of_from() == other.repr_of_from();
  }
}

#[derive(PartialEq, Eq, Debug, Hash)]
pub enum Arch {
  Amd64,
  Arm64,
  Ppc64le,
  S390x,
}

impl Arch {
  pub fn from_string(repr: &String) -> Result<Arch, Error> {
    match repr.to_lowercase().as_str() {
      "amd64" => Result::Ok(Arch::Amd64),
      "arm64" => Result::Ok(Arch::Arm64),
      "ppc64le" => Result::Ok(Arch::Ppc64le),
      "s390x" => Result::Ok(Arch::S390x),
      other => Result::Err(Error::new(
        ErrorKind::InvalidInput, format!("unknown arch {}", other),
      ))
    }
  }
}
