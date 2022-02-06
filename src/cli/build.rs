use std::collections::HashSet;
use wax::Glob;
use crate::core::build as core;

pub struct Build {
  pub base: String,
  pub artifacts: HashSet<String>,
  pub image: String,
  pub arch: HashSet<String>,
  pub tags: HashSet<String>,
}

impl Build {
  pub fn to_core(&self) -> core::Build {
    return core::Build {
      base: self.base.to_owned(),
      image: self.image.to_owned(),
      artifacts: HashSet::from([
        core::Artifact{
          arch: None,
          from: Glob::new("/absolute/file.txt").unwrap(),
          to: "/absolute/file.txt".to_string()
        },
        core::Artifact{
          arch: None,
          from: Glob::new("relative/file.txt").unwrap(),
          to: "relative/file.txt".to_string()
        },
        core::Artifact{
          arch: None,
          from: Glob::new("file.txt").unwrap(),
          to: "/usr/lib/renamed.txt".to_string()
        },
        core::Artifact{
          arch: None,
          from: Glob::new("target/*.jar").unwrap(),
          to: "/usr/lib/app".to_string()
        },
        core::Artifact{
          arch: Some(core::Arch::Amd64),
          from: Glob::new("target/acme-linux-amd64").unwrap(),
          to: "/usr/bin/acme".to_string()
        },
        core::Artifact{
          arch: Some(core::Arch::Arm64),
          from: Glob::new("target/acme-linux-arm64").unwrap(),
          to: "/usr/bin/acme".to_string()
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
      ])
    };
  }
}

#[cfg(test)]
mod tests {
  use std::collections::HashSet;
  use wax::Glob;
  use crate::cli::build as cli;
  use crate::core::build as core;

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
      ])
    };
    let got = input.to_core();
    let want = core::Build{
      base: base.to_string(),
      image: image.to_string(),
      artifacts: HashSet::from([
        core::Artifact{
          arch: None,
          from: Glob::new("/absolute/file.txt").unwrap(),
          to: "/absolute/file.txt".to_string()
        },
        core::Artifact{
          arch: None,
          from: Glob::new("relative/file.txt").unwrap(),
          to: "relative/file.txt".to_string()
        },
        core::Artifact{
          arch: None,
          from: Glob::new("file.txt").unwrap(),
          to: "/usr/lib/renamed.txt".to_string()
        },
        core::Artifact{
          arch: None,
          from: Glob::new("target/*.jar").unwrap(),
          to: "/usr/lib/app".to_string()
        },
        core::Artifact{
          arch: Some(core::Arch::Amd64),
          from: Glob::new("target/acme-linux-amd64").unwrap(),
          to: "/usr/bin/acme".to_string()
        },
        core::Artifact{
          arch: Some(core::Arch::Arm64),
          from: Glob::new("target/acme-linux-arm64").unwrap(),
          to: "/usr/bin/acme".to_string()
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
      ])
    };
    assert_eq!(got, want);
  }
}
