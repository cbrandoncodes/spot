use std::collections::HashSet;

use chrono::{DateTime, NaiveDate, NaiveTime, Utc};
use serde::Serialize;
use thiserror::Error;
use uuid::Uuid;

pub const FACILITY_OPTIONS: &[(&str, &str)] = &[
    ("accessibility", "Accessible facilities"),
    ("air_conditioning", "Air conditioning"),
    ("changing_rooms", "Changing rooms"),
    ("child_care", "Child care"),
    ("group_classes", "Group classes"),
    ("lockers", "Lockers"),
    ("open_24_hours", "Open 24 hours"),
    ("parking", "Parking"),
    ("personal_training", "Personal training"),
    ("pool", "Swimming pool"),
    ("sauna", "Sauna"),
    ("showers", "Showers"),
    ("wifi", "Wi-Fi"),
];

pub const EQUIPMENT_OPTIONS: &[(&str, &str)] = &[
    ("adjustable_bench", "Adjustable bench"),
    ("barbell", "Barbell"),
    ("bodyweight", "Bodyweight area"),
    ("cable_station", "Cable station"),
    ("cardio_machines", "Cardio machines"),
    ("dumbbells", "Dumbbells"),
    ("kettlebells", "Kettlebells"),
    ("lat_pulldown", "Lat pulldown"),
    ("leg_press", "Leg press"),
    ("pull_up_bar", "Pull-up bar"),
    ("resistance_bands", "Resistance bands"),
    ("smith_machine", "Smith machine"),
    ("squat_rack", "Squat rack"),
    ("weight_plates", "Weight plates"),
];

#[derive(Clone, Debug)]
pub struct GymSearch {
    pub query: Option<String>,
    pub area: Option<String>,
    pub location: Option<SearchLocation>,
    pub facilities: Vec<String>,
    pub equipment: Vec<String>,
    pub limit: u64,
    pub offset: u64,
}

impl GymSearch {
    pub fn validate(mut self) -> Result<Self, ValidationError> {
        self.query = normalize_optional("query", self.query, 100)?;
        self.area = normalize_optional("area", self.area, 100)?;
        if let Some(location) = &self.location {
            location.validate()?;
        }
        validate_codes("facilities", &self.facilities, FACILITY_OPTIONS, 10)?;
        validate_codes("equipment", &self.equipment, EQUIPMENT_OPTIONS, 14)?;
        if !(1..=50).contains(&self.limit) {
            return Err(ValidationError::new("limit", "must be between 1 and 50"));
        }
        if self.offset > 500 {
            return Err(ValidationError::new("offset", "must not exceed 500"));
        }
        Ok(self)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct SearchLocation {
    pub latitude: f64,
    pub longitude: f64,
    pub radius_meters: u32,
}

impl SearchLocation {
    fn validate(&self) -> Result<(), ValidationError> {
        if !self.latitude.is_finite() || !(-90.0..=90.0).contains(&self.latitude) {
            return Err(ValidationError::new(
                "location.latitude",
                "must be between -90 and 90",
            ));
        }
        if !self.longitude.is_finite() || !(-180.0..=180.0).contains(&self.longitude) {
            return Err(ValidationError::new(
                "location.longitude",
                "must be between -180 and 180",
            ));
        }
        if !(500..=50_000).contains(&self.radius_meters) {
            return Err(ValidationError::new(
                "location.radiusMeters",
                "must be between 500 and 50000",
            ));
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GymSearchPage {
    pub items: Vec<GymBranchSummary>,
    pub next_offset: Option<u64>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GymBranchSummary {
    pub id: Uuid,
    pub organization: GymOrganizationSummary,
    pub name: String,
    pub area: Option<String>,
    pub city: String,
    pub district: Option<String>,
    pub country_code: String,
    pub latitude: f64,
    pub longitude: f64,
    pub status: String,
    pub distance_meters: Option<u32>,
    pub cover_image_url: Option<String>,
    pub facilities: Vec<CatalogItem>,
    pub equipment: Vec<CatalogItem>,
    pub official_group: Option<OfficialGroupSummary>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GymOrganizationSummary {
    pub id: Uuid,
    pub slug: String,
    pub name: String,
    pub logo_url: Option<String>,
    pub is_verified: bool,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GymOrganizationProfile {
    pub id: Uuid,
    pub slug: String,
    pub name: String,
    pub description: Option<String>,
    pub logo_url: Option<String>,
    pub website_url: Option<String>,
    pub is_verified: bool,
    pub branches: Vec<GymBranchSummary>,
}

#[derive(Clone, Debug, serde::Deserialize, Serialize)]
pub struct CatalogItem {
    pub code: String,
    pub label: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OfficialGroupSummary {
    pub id: Uuid,
    pub slug: String,
    pub name: String,
    pub visibility: String,
    pub join_policy: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GymBranchProfile {
    pub summary: GymBranchSummary,
    pub description: Option<String>,
    pub address: GymAddress,
    pub timezone: String,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub opening_hours: Vec<OpeningHours>,
    pub hour_exceptions: Vec<OpeningHoursException>,
    pub media: Vec<GymMedia>,
    pub facility_details: Vec<BranchFacility>,
    pub equipment_inventory: Vec<BranchEquipment>,
    pub membership_plans: Vec<MembershipPlan>,
    pub links: Vec<GymLink>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GymAddress {
    pub line_1: String,
    pub line_2: Option<String>,
    pub area: Option<String>,
    pub city: String,
    pub district: Option<String>,
    pub region: Option<String>,
    pub country_code: String,
    pub postal_code: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OpeningHours {
    pub weekday: i16,
    pub opens_at: Option<NaiveTime>,
    pub closes_at: Option<NaiveTime>,
    pub is_closed: bool,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OpeningHoursException {
    pub date: NaiveDate,
    pub opens_at: Option<NaiveTime>,
    pub closes_at: Option<NaiveTime>,
    pub is_closed: bool,
    pub note: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GymMedia {
    pub id: Uuid,
    pub media_type: String,
    pub url: String,
    pub alt_text: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BranchFacility {
    pub code: String,
    pub label: String,
    pub details: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BranchEquipment {
    pub code: String,
    pub label: String,
    pub quantity: Option<i32>,
    pub details: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MembershipPlan {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub price_minor: Option<i64>,
    pub joining_fee_minor: Option<i64>,
    pub currency: Option<String>,
    pub billing_period: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GymLink {
    pub link_type: String,
    pub value: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OfficialGroupProfile {
    pub id: Uuid,
    pub slug: String,
    pub name: String,
    pub description: Option<String>,
    pub visibility: String,
    pub join_policy: String,
    pub member_directory_visibility: String,
    pub branch: GroupBranch,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GroupBranch {
    pub id: Uuid,
    pub name: String,
    pub area: Option<String>,
    pub city: String,
    pub organization_name: String,
    pub organization_verified: bool,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GroupPostPage {
    pub items: Vec<GroupPost>,
    pub next_offset: Option<u64>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GroupPost {
    pub id: Uuid,
    pub post_type: String,
    pub title: String,
    pub body: String,
    pub starts_at: Option<DateTime<Utc>>,
    pub ends_at: Option<DateTime<Utc>>,
    pub published_at: DateTime<Utc>,
    pub is_pinned: bool,
}

fn validate_codes(
    field: &'static str,
    values: &[String],
    supported: &[(&str, &str)],
    maximum: usize,
) -> Result<(), ValidationError> {
    if values.len() > maximum {
        return Err(ValidationError::new(
            field,
            format!("must contain at most {maximum} values"),
        ));
    }
    let unique = values.iter().map(String::as_str).collect::<HashSet<_>>();
    if unique.len() != values.len() {
        return Err(ValidationError::new(field, "cannot contain duplicates"));
    }
    if values
        .iter()
        .any(|value| !supported.iter().any(|(code, _)| code == value))
    {
        return Err(ValidationError::new(field, "contains an unsupported value"));
    }
    Ok(())
}

fn normalize_optional(
    field: &'static str,
    value: Option<String>,
    maximum: usize,
) -> Result<Option<String>, ValidationError> {
    let Some(value) = value else {
        return Ok(None);
    };
    let value = value.trim();
    if value.is_empty() {
        return Ok(None);
    }
    if value.chars().count() > maximum {
        return Err(ValidationError::new(
            field,
            format!("must not exceed {maximum} characters"),
        ));
    }
    Ok(Some(value.to_owned()))
}

#[derive(Clone, Debug, Error, Eq, PartialEq)]
#[error("{field} {message}")]
pub struct ValidationError {
    pub field: &'static str,
    pub message: String,
}

impl ValidationError {
    pub fn new(field: &'static str, message: impl Into<String>) -> Self {
        Self {
            field,
            message: message.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{GymSearch, SearchLocation};

    fn search() -> GymSearch {
        GymSearch {
            query: None,
            area: None,
            location: None,
            facilities: Vec::new(),
            equipment: Vec::new(),
            limit: 20,
            offset: 0,
        }
    }

    #[test]
    fn rejects_unknown_catalog_codes() {
        let mut search = search();
        search.facilities.push("made_up_facility".to_owned());
        assert_eq!(
            search
                .validate()
                .expect_err("unknown facilities must be rejected")
                .field,
            "facilities"
        );
    }

    #[test]
    fn rejects_invalid_coordinates() {
        let mut search = search();
        search.location = Some(SearchLocation {
            latitude: 91.0,
            longitude: 3.0,
            radius_meters: 10_000,
        });
        assert_eq!(
            search
                .validate()
                .expect_err("invalid coordinates must be rejected")
                .field,
            "location.latitude"
        );
    }
}
