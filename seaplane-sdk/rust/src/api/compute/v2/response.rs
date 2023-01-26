use serde::{Deserialize, Serialize};
use url::Url;

use crate::api::compute::v2::models::*;

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq)]
pub struct PagedResponse<T> {
    objects: Vec<T>,
    meta: PageMetadata,
}

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq)]
pub struct PageMetadata {
    /// The total number of entries in all pages
    #[serde(default)]
    total: usize,

    /// URL to fetch the next page of entries
    #[serde(default)]
    next: Option<Url>,

    /// URL to fetch the previous page of entries
    #[serde(default)]
    prev: Option<Url>,
}

pub type GetFormationsResponse = PagedResponse<Formation>;
pub type CreateFormationResponse = Formation;
pub type GetFormationResponse = Formation;
pub type DeleteFormationResponse = ();
