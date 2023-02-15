use seaplane::{
    api::compute::v2::FlightId,
    rexports::container_image_ref::{ImageReference, ImageReferenceError},
};

use crate::{
    error::{CliError, Result},
    ops::NameId,
};

pub type FlightNameId = NameId<FlightId>;

/// Allows eliding `registry` but otherwise just proxies parsing to ImageReference
pub fn str_to_image_ref(registry: &str, image_str: &str) -> Result<ImageReference> {
    match image_str.parse::<ImageReference>() {
        Ok(ir) => Ok(ir),
        Err(ImageReferenceError::ErrDomainInvalidFormat(_)) => {
            let ir: ImageReference = format!("{registry}/{image_str}").parse()?;
            Ok(ir)
        }
        Err(e) => Err(CliError::from(e)),
    }
}
