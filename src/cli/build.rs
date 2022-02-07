use std::collections::HashSet;
use std::io::{Error, ErrorKind};

use ocilot_core::build as core;
use regex::RegexBuilder;

pub struct Build {
  pub base: String,
  pub artifacts: HashSet<String>,
  pub image: String,
  pub arch: HashSet<String>,
  pub tags: HashSet<String>,
}

impl Build {
  pub fn to_core(&self) -> core::Build {
    let base = self.base.to_owned();
    let image = self.image.to_owned();
    let tags = self.tags
      .iter()
      .cloned()
      .collect();
    let arch = self.arch
      .iter()
      .map(|repr| arch_from_string(repr))
      .map(|res| res.unwrap())
      .collect();
    let artifacts = self.artifacts
      .iter()
      .map(|repr| artifact_from_string(repr))
      .map(|res| res.unwrap())
      .collect();
    return core::Build {
      tags,
      base,
      image,
      arch,
      artifacts,
    };
  }
}

fn invalid_format(repr: &String) -> Error {
  Error::new(
    ErrorKind::InvalidInput,
    format!("invalid format for artifact: {:?}", repr),
  )
}

fn artifact_from_string(repr: &String) -> Result<core::Artifact, Error> {
  // Ref.: https://regex101.com/r/q2qVXt/1
  let raw_re = r"^(?:(?P<arch>[^\n:]+):)?(?P<from>[^\n:]+)(?::(?P<to>[^\n:]+))?$";
  let re = RegexBuilder::new(raw_re)
    .swap_greed(true)
    .build()
    .unwrap();
  match re.captures(repr) {
    None => Result::Err(invalid_format(repr)),
    Some(cap) => {
      cap.name("from")
        .ok_or(invalid_format(repr))
        .map(|m| String::from(m.as_str()))
        .and_then(|from| {
          let to = match cap.name("to") {
            None => String::from(&from),
            Some(m) => String::from(m.as_str())
          };
          match cap.name("arch") {
            None => Option::None,
            Some(m) => Option::Some(arch_from_string(&m.as_str().to_string()))
          }.transpose().map(|arch| core::Artifact {
            arch,
            from,
            to,
          })
        })
    }
  }
}

fn arch_from_string(repr: &String) -> Result<core::Arch, Error> {
  match repr.to_lowercase().as_str() {
    "amd64" => Result::Ok(core::Arch::Amd64),
    "arm64" => Result::Ok(core::Arch::Arm64),
    "ppc64le" => Result::Ok(core::Arch::Ppc64le),
    "s390x" => Result::Ok(core::Arch::S390x),
    other => Result::Err(Error::new(
      ErrorKind::InvalidInput, format!("unknown arch: {}", other),
    ))
  }
}

#[cfg(test)]
mod tests {
  use std::collections::HashSet;
  use std::io::{Error, ErrorKind};

  use ocilot_core::build as core;

  use crate::cli::build as cli;

  #[test]
  fn arch_from_string() {
    let cases = vec![
      ("amd64", Result::Ok(core::Arch::Amd64)),
      ("arm64", Result::Ok(core::Arch::Arm64)),
      ("ppc64le", Result::Ok(core::Arch::Ppc64le)),
      ("s390x", Result::Ok(core::Arch::S390x)),
      ("invalid", Result::Err(Error::new(
        ErrorKind::InvalidInput, "unknown arch: invalid"),
      )),
    ];

    for case in cases {
      let repr = String::from(case.0);
      let want = case.1;
      let got = cli::arch_from_string(&repr);

      assert_eq!(got.is_ok(), want.is_ok());
      if want.is_ok() {
        assert_eq!(got.ok(), want.ok());
      } else {
        let got_err = got.err().unwrap();
        let want_err = want.err().unwrap();
        assert_eq!(got_err.kind(), want_err.kind());
        assert_eq!(got_err.to_string(), want_err.to_string());
      }
    }
  }

  #[test]
  fn artifact_from_string() {
    let input = String::from("amd64:target/acme-linux-amd64:/usr/bin/acme:foo");
    let res = cli::artifact_from_string(&input);
    assert_eq!(res.is_err(), true);
    let err = res.err().unwrap();
    assert_eq!(err.kind(), ErrorKind::InvalidInput);
    assert_eq!(err.to_string(),
               concat!("invalid format for artifact: ",
               "\"amd64:target/acme-linux-amd64:/usr/bin/acme:foo\""));
  }

  #[test]
  fn to_core() {
    let base = "registry.access.redhat.com/ubi8/ubi";
    let image = "quay.io/localhost/example";
    let input = cli::Build {
      base: base.to_string(),
      image: image.to_string(),
      artifacts: HashSet::from([
        "relative/file.txt".to_string(),
        "/absolute/file.txt".to_string(),
        "file.txt:/usr/lib/renamed.txt".to_string(),
        "target/*.jar:/usr/lib/app".to_string(),
        "amd64:target/acme-linux-amd64:/usr/bin/acme".to_string(),
        "arm64:target/acme-linux-arm64:/usr/bin/acme".to_string(),
      ]),
      arch: HashSet::from([
        "amd64".to_string(),
        "arm64".to_string()
      ]),
      tags: HashSet::from([
        "latest".to_string(),
        "v1".to_string(),
        "v1.1".to_string()
      ]),
    };
    let got = input.to_core();
    let want = core::Build {
      base: base.to_string(),
      image: image.to_string(),
      artifacts: HashSet::from([
        core::Artifact {
          arch: None,
          from: "/absolute/file.txt".to_string(),
          to: "/absolute/file.txt".to_string(),
        },
        core::Artifact {
          arch: None,
          from: "relative/file.txt".to_string(),
          to: "relative/file.txt".to_string(),
        },
        core::Artifact {
          arch: None,
          from: "file.txt".to_string(),
          to: "/usr/lib/renamed.txt".to_string(),
        },
        core::Artifact {
          arch: None,
          from: "target/*.jar".to_string(),
          to: "/usr/lib/app".to_string(),
        },
        core::Artifact {
          arch: Some(core::Arch::Amd64),
          from: "target/acme-linux-amd64".to_string(),
          to: "/usr/bin/acme".to_string(),
        },
        core::Artifact {
          arch: Some(core::Arch::Arm64),
          from: "target/acme-linux-arm64".to_string(),
          to: "/usr/bin/acme".to_string(),
        }
      ]),
      arch: HashSet::from([
        core::Arch::Amd64,
        core::Arch::Arm64
      ]),
      tags: HashSet::from([
        "latest".to_string(),
        "v1".to_string(),
        "v1.1".to_string()
      ]),
    };
    assert_eq!(got, want);
  }
}
