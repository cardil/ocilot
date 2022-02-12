use ocilot_core::error::Error;
use ocilot_core::oci;
use ocilot_core::oci::Image;

#[derive(Debug)]
pub struct Rest {}

impl oci::Registry for Rest {
  fn fetch(&self, _: String) -> Result<Box<dyn Image>, Error> {
    todo!()
  }
}
