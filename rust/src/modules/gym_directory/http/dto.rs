use serde::{Deserialize, Serialize};

use super::super::domain::{EQUIPMENT_OPTIONS, FACILITY_OPTIONS, GymSearch, SearchLocation};

#[derive(Debug, Deserialize, garde::Validate)]
#[garde(allow_unvalidated)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct SearchGymsRequest {
    #[garde(length(chars, max = 100))]
    query: Option<String>,
    #[garde(length(chars, max = 100))]
    area: Option<String>,
    #[garde(dive)]
    location: Option<SearchLocationRequest>,
    #[serde(default)]
    #[garde(length(max = 10), inner(length(bytes, max = 64)))]
    facilities: Vec<String>,
    #[serde(default)]
    #[garde(length(max = 14), inner(length(bytes, max = 64)))]
    equipment: Vec<String>,
    #[serde(default = "default_limit")]
    #[garde(range(min = 1, max = 50))]
    limit: u64,
    #[serde(default)]
    #[garde(range(max = 500))]
    offset: u64,
}

impl SearchGymsRequest {
    pub(super) fn into_domain(self) -> GymSearch {
        GymSearch {
            query: self.query,
            area: self.area,
            location: self.location.map(Into::into),
            facilities: self.facilities,
            equipment: self.equipment,
            limit: self.limit,
            offset: self.offset,
        }
    }
}

#[derive(Debug, Deserialize, garde::Validate)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct SearchLocationRequest {
    #[garde(range(min = -90.0, max = 90.0))]
    latitude: f64,
    #[garde(range(min = -180.0, max = 180.0))]
    longitude: f64,
    #[serde(default = "default_radius")]
    #[garde(range(min = 500, max = 50_000))]
    radius_meters: u32,
}

impl From<SearchLocationRequest> for SearchLocation {
    fn from(value: SearchLocationRequest) -> Self {
        Self {
            latitude: value.latitude,
            longitude: value.longitude,
            radius_meters: value.radius_meters,
        }
    }
}

#[derive(Debug, Deserialize, garde::Validate)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct GroupPostsQuery {
    #[serde(default = "default_limit")]
    #[garde(range(min = 1, max = 50))]
    pub(super) limit: u64,
    #[serde(default)]
    #[garde(range(max = 500))]
    pub(super) offset: u64,
}

const fn default_limit() -> u64 {
    20
}

const fn default_radius() -> u32 {
    10_000
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct CatalogOptionsResponse {
    facilities: Vec<CatalogOption>,
    equipment: Vec<CatalogOption>,
}

impl CatalogOptionsResponse {
    pub(super) fn new() -> Self {
        Self {
            facilities: options(FACILITY_OPTIONS),
            equipment: options(EQUIPMENT_OPTIONS),
        }
    }
}

#[derive(Serialize)]
struct CatalogOption {
    code: &'static str,
    label: &'static str,
}

fn options(values: &[(&'static str, &'static str)]) -> Vec<CatalogOption> {
    values
        .iter()
        .map(|(code, label)| CatalogOption { code, label })
        .collect()
}
