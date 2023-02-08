use serde::{Deserialize, Serialize};
use url::Url;

use crate::api::compute::v2::models::*;

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq)]
pub struct PagedResponse<T> {
    pub objects: Vec<T>,
    pub meta: PageMetadata,
}

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq)]
pub struct PageMetadata {
    /// The total number of entries in all pages
    #[serde(default)]
    pub total: usize,

    /// URL to fetch the next page of entries
    #[serde(default)]
    pub next: Option<Url>,

    /// URL to fetch the previous page of entries
    #[serde(default)]
    pub prev: Option<Url>,
}

pub type GetFormationsResponse = PagedResponse<Formation>;
pub type CreateFormationResponse = Formation;
pub type GetFormationResponse = Formation;
pub type DeleteFormationResponse = ();
