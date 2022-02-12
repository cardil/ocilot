use ocilot_core::error::Error;
use ocilot_core::oci;
use ocilot_core::oci::Image;

#[derive(Debug)]
pub struct HomeBased {}

impl oci::Cache for HomeBased {
  fn list(&self) -> Result<Vec<Box<dyn Image>>, Error> {
    todo!()
  }
}
