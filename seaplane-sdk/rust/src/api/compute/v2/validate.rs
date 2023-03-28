use std::result::Result as StdResult;

use crate::api::compute::error::FormationValidation;
#[cfg(doc)]
use crate::api::compute::v2::Formation;

/// Determine if a [`Formation`] name (slug) is valid or not.
///
/// The name must be URL safe following the these validation rules:
///
///  - 1-63 alphanumeric (ASCII lowercase only) characters or hyphen (0-9, a-z, and '-' )
///  - hyphens ('-') may not be repeated (i.e. '--')
///  - no more than three (3) total hyphens
///  - may not start, or end with a hyphen
pub fn validate_formation_name(name: impl AsRef<str>) -> StdResult<(), FormationValidation> {
    let name = name.as_ref();
    if name.is_empty() {
        return Err(FormationValidation::NameEmpty);
    }
    if name.len() > 63 {
        return Err(FormationValidation::NameLength);
    }
    let mut hyphen_count = 0;
    if !name.as_bytes().iter().all(|&c| {
        (c > b'/' && c < b':') || (c > b'`' && c < b'{') || {
            if c == b'-' {
                hyphen_count += 1;
                true
            } else {
                false
            }
        }
    }) {
        return Err(FormationValidation::NameInvalidChar);
    }
    if hyphen_count > 3 {
        return Err(FormationValidation::NameTooManyHyphens);
    }
    if name.contains("--") {
        return Err(FormationValidation::NameConsecutiveHyphens);
    }
    if name.ends_with('-') || name.starts_with('-') {
        return Err(FormationValidation::NameLeadingOrTrailingHyphen);
    }

    Ok(())
}
