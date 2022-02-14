use ocilot_core::error::Result;
use ocilot_core::oci;
use ocilot_core::oci::Image;
use tracing::{instrument, warn};

#[derive(Debug)]
pub struct HomeBased {}

impl oci::Cache for HomeBased {
  #[instrument(ret, level = "trace")]
  fn list(&self) -> Result<Vec<Box<dyn Image>>> {
    warn!("Not yet implemented");
    Ok(Vec::new())
  }
}
