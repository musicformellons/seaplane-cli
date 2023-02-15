use seaplane::api::compute::v2::validate_formation_name;

/// Current Rules:
///
///  - 1-63 alphanumeric (only ASCII lowercase) characters or hyphen (0-9, a-z, and '-')
///  - hyphens ('-') may not be repeated (i.e. '--')
///  - no more than three (3) total hyphens
///  - no consecutive hyphens
///  - no trailing hyphen
pub fn validate_name(name: &str) -> Result<String, String> {
    validate_formation_name(name).map_err(|e| e.to_string())?;
    Ok(name.into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_names() {
        assert!(validate_name("").is_err());
        assert!(validate_name("imwaaaaaytoolongforanythingthatshouldbeanameimwaaaaaytoolongforanythingthatshouldbeaname").is_err());
        assert!(validate_name("no-ending-hyphen-").is_err());
        assert!(validate_name("no-special-chars!").is_err());
        assert!(validate_name("noUperCase").is_err());
        assert!(validate_name("too-many-hyphens-in-here").is_err());
    }
}
