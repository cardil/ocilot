use ocilot_core::error::Result;
use ocilot_core::oci;
use ocilot_core::oci::Image;
use tracing::instrument;

#[derive(Debug)]
pub struct Rest {}

impl oci::Registry for Rest {
  #[instrument(ret, level = "trace")]
  fn fetch(&self, _: String) -> Result<Box<dyn Image>> {
    todo!()
  }
}
