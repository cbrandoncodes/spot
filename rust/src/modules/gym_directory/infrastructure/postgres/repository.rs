use std::fmt::Write as _;

use chrono::{DateTime, NaiveDate, NaiveTime, Utc};
use sea_orm::{DatabaseBackend, DatabaseConnection, DbErr, FromQueryResult, Statement, Value};
use thiserror::Error;
use uuid::Uuid;

use crate::modules::gym_directory::domain::{
    BranchEquipment, BranchFacility, GroupBranch, GroupPost, GymAddress, GymBranchProfile,
    GymBranchSummary, GymLink, GymMedia, GymOrganizationProfile, GymOrganizationSummary, GymSearch,
    MembershipPlan, OfficialGroupProfile, OfficialGroupSummary, OpeningHours,
    OpeningHoursException,
};

#[derive(Clone)]
pub struct GymDirectoryRepository {
    database: DatabaseConnection,
}

impl GymDirectoryRepository {
    pub fn new(database: DatabaseConnection) -> Self {
        Self { database }
    }

    pub async fn search(
        &self,
        search: &GymSearch,
    ) -> Result<Vec<GymBranchSummary>, RepositoryError> {
        self.search_scoped(search, None).await
    }

    async fn search_scoped(
        &self,
        search: &GymSearch,
        organization_id: Option<Uuid>,
    ) -> Result<Vec<GymBranchSummary>, RepositoryError> {
        let mut bindings = Bindings::default();
        let distance_expression = if let Some(location) = search.location {
            let latitude = bindings.push(location.latitude);
            let longitude = bindings.push(location.longitude);
            format!(
                "earth_distance(ll_to_earth({latitude}, {longitude}), ll_to_earth(b.latitude, b.longitude))"
            )
        } else {
            "NULL::double precision".to_owned()
        };

        let mut sql = format!(
            r#"
                SELECT
                    b.id,
                    b.name AS branch_name,
                    b.area,
                    b.city,
                    b.district,
                    b.country_code,
                    b.latitude,
                    b.longitude,
                    b.status,
                    o.id AS organization_id,
                    o.slug AS organization_slug,
                    o.name AS organization_name,
                    o.logo_url AS organization_logo_url,
                    (o.verification_status = 'verified') AS organization_verified,
                    {distance_expression} AS distance_meters,
                    (
                        SELECT gm.url
                        FROM gym_media gm
                        WHERE gm.branch_id = b.id
                        ORDER BY (gm.media_type = 'cover') DESC, gm.sort_order, gm.created_at
                        LIMIT 1
                    ) AS cover_image_url,
                    COALESCE((
                        SELECT jsonb_agg(
                            jsonb_build_object('code', ft.code, 'label', ft.label)
                            ORDER BY ft.label
                        )
                        FROM gym_branch_facilities bf
                        JOIN gym_facility_types ft ON ft.code = bf.facility_code
                        WHERE bf.branch_id = b.id
                    ), '[]'::jsonb) AS facilities,
                    COALESCE((
                        SELECT jsonb_agg(
                            jsonb_build_object('code', et.code, 'label', et.label)
                            ORDER BY et.label
                        )
                        FROM gym_branch_equipment be
                        JOIN gym_equipment_types et ON et.code = be.equipment_code
                        WHERE be.branch_id = b.id
                    ), '[]'::jsonb) AS equipment,
                    g.id AS group_id,
                    g.slug AS group_slug,
                    g.name AS group_name,
                    g.visibility AS group_visibility,
                    g.join_policy AS group_join_policy
                FROM gym_branches b
                JOIN gym_organizations o ON o.id = b.organization_id
                LEFT JOIN official_gym_groups g
                    ON g.branch_id = b.id AND g.visibility = 'public'
                WHERE b.visibility = 'public'
                    AND b.status <> 'permanently_closed'
                    AND o.visibility = 'public'
            "#
        );

        if let Some(query) = &search.query {
            let parameter = bindings.push(query.clone());
            let _ = write!(
                sql,
                " AND (lower(o.name) LIKE '%' || lower({parameter}) || '%' OR lower(b.name) LIKE '%' || lower({parameter}) || '%')"
            );
        }
        if let Some(area) = &search.area {
            let parameter = bindings.push(area.clone());
            let _ = write!(
                sql,
                " AND lower(coalesce(b.area, '') || ' ' || b.city || ' ' || coalesce(b.district, '') || ' ' || coalesce(b.region, '')) LIKE '%' || lower({parameter}) || '%'"
            );
        }
        if let Some(organization_id) = organization_id {
            let parameter = bindings.push(organization_id);
            let _ = write!(sql, " AND b.organization_id = {parameter}");
        }
        for facility in &search.facilities {
            let parameter = bindings.push(facility.clone());
            let _ = write!(
                sql,
                " AND EXISTS (SELECT 1 FROM gym_branch_facilities requested_facility WHERE requested_facility.branch_id = b.id AND requested_facility.facility_code = {parameter})"
            );
        }
        for equipment in &search.equipment {
            let parameter = bindings.push(equipment.clone());
            let _ = write!(
                sql,
                " AND EXISTS (SELECT 1 FROM gym_branch_equipment requested_equipment WHERE requested_equipment.branch_id = b.id AND requested_equipment.equipment_code = {parameter})"
            );
        }
        if let Some(location) = search.location {
            let latitude = bindings.push(location.latitude);
            let longitude = bindings.push(location.longitude);
            let radius = bindings.push(f64::from(location.radius_meters));
            let _ = write!(
                sql,
                " AND earth_box(ll_to_earth({latitude}, {longitude}), {radius}) @> ll_to_earth(b.latitude, b.longitude) AND earth_distance(ll_to_earth({latitude}, {longitude}), ll_to_earth(b.latitude, b.longitude)) <= {radius}"
            );
        }

        if search.location.is_some() {
            sql.push_str(
                " ORDER BY distance_meters ASC, organization_verified DESC, o.name, b.name",
            );
        } else {
            sql.push_str(" ORDER BY organization_verified DESC, o.name, b.name");
        }
        let limit = bindings.push(i64::try_from(search.limit + 1).unwrap_or(51));
        let offset = bindings.push(i64::try_from(search.offset).unwrap_or(0));
        let _ = write!(sql, " LIMIT {limit} OFFSET {offset}");

        let rows = SearchRow::find_by_statement(Statement::from_sql_and_values(
            DatabaseBackend::Postgres,
            sql,
            bindings.values,
        ))
        .all(&self.database)
        .await?;
        rows.into_iter().map(TryInto::try_into).collect()
    }

    pub async fn find_organization(
        &self,
        slug: &str,
    ) -> Result<Option<GymOrganizationProfile>, RepositoryError> {
        let Some(row) = OrganizationRow::find_by_statement(Statement::from_sql_and_values(
            DatabaseBackend::Postgres,
            r#"
                SELECT id, slug, name, description, logo_url, website_url,
                       (verification_status = 'verified') AS is_verified
                FROM gym_organizations
                WHERE slug = $1 AND visibility = 'public'
            "#,
            [slug.into()],
        ))
        .one(&self.database)
        .await?
        else {
            return Ok(None);
        };

        let mut branches = self
            .search_scoped(
                &GymSearch {
                    query: None,
                    area: None,
                    location: None,
                    facilities: Vec::new(),
                    equipment: Vec::new(),
                    limit: 50,
                    offset: 0,
                },
                Some(row.id),
            )
            .await?;
        branches.truncate(50);
        Ok(Some(GymOrganizationProfile {
            id: row.id,
            slug: row.slug,
            name: row.name,
            description: row.description,
            logo_url: row.logo_url,
            website_url: row.website_url,
            is_verified: row.is_verified,
            branches,
        }))
    }

    pub async fn find_branch(
        &self,
        branch_id: Uuid,
    ) -> Result<Option<GymBranchProfile>, RepositoryError> {
        let statement = Statement::from_sql_and_values(
            DatabaseBackend::Postgres,
            r#"
                SELECT
                    b.id,
                    b.name AS branch_name,
                    b.description,
                    b.address_line_1,
                    b.address_line_2,
                    b.area,
                    b.city,
                    b.district,
                    b.region,
                    b.country_code,
                    b.postal_code,
                    b.latitude,
                    b.longitude,
                    b.timezone,
                    b.phone,
                    b.email,
                    b.status,
                    o.id AS organization_id,
                    o.slug AS organization_slug,
                    o.name AS organization_name,
                    o.logo_url AS organization_logo_url,
                    (o.verification_status = 'verified') AS organization_verified,
                    (
                        SELECT gm.url
                        FROM gym_media gm
                        WHERE gm.branch_id = b.id
                        ORDER BY (gm.media_type = 'cover') DESC, gm.sort_order, gm.created_at
                        LIMIT 1
                    ) AS cover_image_url,
                    COALESCE((
                        SELECT jsonb_agg(jsonb_build_object('code', ft.code, 'label', ft.label) ORDER BY ft.label)
                        FROM gym_branch_facilities bf
                        JOIN gym_facility_types ft ON ft.code = bf.facility_code
                        WHERE bf.branch_id = b.id
                    ), '[]'::jsonb) AS facilities,
                    COALESCE((
                        SELECT jsonb_agg(jsonb_build_object('code', et.code, 'label', et.label) ORDER BY et.label)
                        FROM gym_branch_equipment be
                        JOIN gym_equipment_types et ON et.code = be.equipment_code
                        WHERE be.branch_id = b.id
                    ), '[]'::jsonb) AS equipment,
                    g.id AS group_id,
                    g.slug AS group_slug,
                    g.name AS group_name,
                    g.visibility AS group_visibility,
                    g.join_policy AS group_join_policy
                FROM gym_branches b
                JOIN gym_organizations o ON o.id = b.organization_id
                LEFT JOIN official_gym_groups g
                    ON g.branch_id = b.id AND g.visibility <> 'private'
                WHERE b.id = $1
                    AND b.visibility <> 'private'
                    AND o.visibility <> 'private'
            "#,
            [branch_id.into()],
        );
        let Some(base) = BranchRow::find_by_statement(statement)
            .one(&self.database)
            .await?
        else {
            return Ok(None);
        };

        let opening_hours = OpeningHoursRow::find_by_statement(statement_for(
            r#"SELECT weekday, opens_at, closes_at, is_closed
               FROM gym_branch_opening_hours WHERE branch_id = $1 ORDER BY weekday"#,
            branch_id,
        ))
        .all(&self.database)
        .await?
        .into_iter()
        .map(Into::into)
        .collect();
        let hour_exceptions = OpeningHoursExceptionRow::find_by_statement(statement_for(
            r#"SELECT exception_date, opens_at, closes_at, is_closed, note
               FROM gym_branch_hour_exceptions
               WHERE branch_id = $1 AND exception_date >= CURRENT_DATE
               ORDER BY exception_date LIMIT 60"#,
            branch_id,
        ))
        .all(&self.database)
        .await?
        .into_iter()
        .map(Into::into)
        .collect();
        let media = GymMediaRow::find_by_statement(statement_for(
            r#"SELECT id, media_type, url, alt_text FROM gym_media
               WHERE branch_id = $1 ORDER BY (media_type = 'cover') DESC, sort_order, created_at"#,
            branch_id,
        ))
        .all(&self.database)
        .await?
        .into_iter()
        .map(Into::into)
        .collect();
        let facility_details = BranchFacilityRow::find_by_statement(statement_for(
            r#"SELECT ft.code, ft.label, bf.details
               FROM gym_branch_facilities bf
               JOIN gym_facility_types ft ON ft.code = bf.facility_code
               WHERE bf.branch_id = $1 ORDER BY ft.label"#,
            branch_id,
        ))
        .all(&self.database)
        .await?
        .into_iter()
        .map(Into::into)
        .collect();
        let equipment_inventory = BranchEquipmentRow::find_by_statement(statement_for(
            r#"SELECT et.code, et.label, be.quantity, be.details
               FROM gym_branch_equipment be
               JOIN gym_equipment_types et ON et.code = be.equipment_code
               WHERE be.branch_id = $1 ORDER BY et.label"#,
            branch_id,
        ))
        .all(&self.database)
        .await?
        .into_iter()
        .map(Into::into)
        .collect();
        let membership_plans = MembershipPlanRow::find_by_statement(statement_for(
            r#"SELECT id, name, description, price_minor, joining_fee_minor, currency, billing_period
               FROM gym_membership_plans
               WHERE branch_id = $1 AND is_active
               ORDER BY sort_order, name"#,
            branch_id,
        ))
        .all(&self.database)
        .await?
        .into_iter()
        .map(Into::into)
        .collect();
        let links = GymLinkRow::find_by_statement(statement_for(
            r#"SELECT link_type, value FROM gym_links
               WHERE branch_id = $1 ORDER BY sort_order, link_type"#,
            branch_id,
        ))
        .all(&self.database)
        .await?
        .into_iter()
        .map(Into::into)
        .collect();

        Ok(Some(base.into_profile(
            opening_hours,
            hour_exceptions,
            media,
            facility_details,
            equipment_inventory,
            membership_plans,
            links,
        )?))
    }

    pub async fn find_group(
        &self,
        group_id: Uuid,
    ) -> Result<Option<OfficialGroupProfile>, RepositoryError> {
        let row = GroupRow::find_by_statement(Statement::from_sql_and_values(
            DatabaseBackend::Postgres,
            r#"
                SELECT
                    g.id, g.slug, g.name, g.description, g.visibility, g.join_policy,
                    g.member_directory_visibility, b.id AS branch_id, b.name AS branch_name,
                    b.area, b.city, o.name AS organization_name,
                    (o.verification_status = 'verified') AS organization_verified
                FROM official_gym_groups g
                JOIN gym_branches b ON b.id = g.branch_id
                JOIN gym_organizations o ON o.id = b.organization_id
                WHERE g.id = $1
                    AND g.visibility <> 'private'
                    AND b.visibility <> 'private'
                    AND o.visibility <> 'private'
            "#,
            [group_id.into()],
        ))
        .one(&self.database)
        .await?;
        Ok(row.map(Into::into))
    }

    pub async fn list_group_posts(
        &self,
        group_id: Uuid,
        limit: u64,
        offset: u64,
    ) -> Result<Vec<GroupPost>, RepositoryError> {
        let rows = GroupPostRow::find_by_statement(Statement::from_sql_and_values(
            DatabaseBackend::Postgres,
            r#"
                SELECT p.id, p.post_type, p.title, p.body, p.starts_at, p.ends_at,
                       p.published_at, p.is_pinned
                FROM gym_group_posts p
                JOIN official_gym_groups g ON g.id = p.group_id
                JOIN gym_branches b ON b.id = g.branch_id
                JOIN gym_organizations o ON o.id = b.organization_id
                WHERE p.group_id = $1
                    AND p.published_at <= CURRENT_TIMESTAMP
                    AND g.visibility <> 'private'
                    AND b.visibility <> 'private'
                    AND o.visibility <> 'private'
                ORDER BY p.is_pinned DESC, p.published_at DESC
                LIMIT $2 OFFSET $3
            "#,
            [
                group_id.into(),
                i64::try_from(limit + 1).unwrap_or(51).into(),
                i64::try_from(offset).unwrap_or(0).into(),
            ],
        ))
        .all(&self.database)
        .await?;
        Ok(rows.into_iter().map(Into::into).collect())
    }
}

#[derive(Default)]
struct Bindings {
    values: Vec<Value>,
}

impl Bindings {
    fn push(&mut self, value: impl Into<Value>) -> String {
        self.values.push(value.into());
        format!("${}", self.values.len())
    }
}

fn statement_for(sql: &str, id: Uuid) -> Statement {
    Statement::from_sql_and_values(DatabaseBackend::Postgres, sql, [id.into()])
}

#[derive(FromQueryResult)]
struct OrganizationRow {
    id: Uuid,
    slug: String,
    name: String,
    description: Option<String>,
    logo_url: Option<String>,
    website_url: Option<String>,
    is_verified: bool,
}

#[derive(FromQueryResult)]
struct SearchRow {
    id: Uuid,
    branch_name: String,
    area: Option<String>,
    city: String,
    district: Option<String>,
    country_code: String,
    latitude: f64,
    longitude: f64,
    status: String,
    organization_id: Uuid,
    organization_slug: String,
    organization_name: String,
    organization_logo_url: Option<String>,
    organization_verified: bool,
    distance_meters: Option<f64>,
    cover_image_url: Option<String>,
    facilities: serde_json::Value,
    equipment: serde_json::Value,
    group_id: Option<Uuid>,
    group_slug: Option<String>,
    group_name: Option<String>,
    group_visibility: Option<String>,
    group_join_policy: Option<String>,
}

impl TryFrom<SearchRow> for GymBranchSummary {
    type Error = RepositoryError;

    fn try_from(row: SearchRow) -> Result<Self, Self::Error> {
        let official_group = group_summary(
            row.group_id,
            row.group_slug,
            row.group_name,
            row.group_visibility,
            row.group_join_policy,
        )?;
        Ok(Self {
            id: row.id,
            organization: GymOrganizationSummary {
                id: row.organization_id,
                slug: row.organization_slug,
                name: row.organization_name,
                logo_url: row.organization_logo_url,
                is_verified: row.organization_verified,
            },
            name: row.branch_name,
            area: row.area,
            city: row.city,
            district: row.district,
            country_code: row.country_code,
            latitude: row.latitude,
            longitude: row.longitude,
            status: row.status,
            distance_meters: row.distance_meters.map(distance_to_u32),
            cover_image_url: row.cover_image_url,
            facilities: serde_json::from_value(row.facilities)?,
            equipment: serde_json::from_value(row.equipment)?,
            official_group,
        })
    }
}

#[derive(FromQueryResult)]
struct BranchRow {
    id: Uuid,
    branch_name: String,
    description: Option<String>,
    address_line_1: String,
    address_line_2: Option<String>,
    area: Option<String>,
    city: String,
    district: Option<String>,
    region: Option<String>,
    country_code: String,
    postal_code: Option<String>,
    latitude: f64,
    longitude: f64,
    timezone: String,
    phone: Option<String>,
    email: Option<String>,
    status: String,
    organization_id: Uuid,
    organization_slug: String,
    organization_name: String,
    organization_logo_url: Option<String>,
    organization_verified: bool,
    cover_image_url: Option<String>,
    facilities: serde_json::Value,
    equipment: serde_json::Value,
    group_id: Option<Uuid>,
    group_slug: Option<String>,
    group_name: Option<String>,
    group_visibility: Option<String>,
    group_join_policy: Option<String>,
}

impl BranchRow {
    #[allow(clippy::too_many_arguments)]
    fn into_profile(
        self,
        opening_hours: Vec<OpeningHours>,
        hour_exceptions: Vec<OpeningHoursException>,
        media: Vec<GymMedia>,
        facility_details: Vec<BranchFacility>,
        equipment_inventory: Vec<BranchEquipment>,
        membership_plans: Vec<MembershipPlan>,
        links: Vec<GymLink>,
    ) -> Result<GymBranchProfile, RepositoryError> {
        let summary = GymBranchSummary {
            id: self.id,
            organization: GymOrganizationSummary {
                id: self.organization_id,
                slug: self.organization_slug,
                name: self.organization_name,
                logo_url: self.organization_logo_url,
                is_verified: self.organization_verified,
            },
            name: self.branch_name,
            area: self.area.clone(),
            city: self.city.clone(),
            district: self.district.clone(),
            country_code: self.country_code.clone(),
            latitude: self.latitude,
            longitude: self.longitude,
            status: self.status,
            distance_meters: None,
            cover_image_url: self.cover_image_url,
            facilities: serde_json::from_value(self.facilities)?,
            equipment: serde_json::from_value(self.equipment)?,
            official_group: group_summary(
                self.group_id,
                self.group_slug,
                self.group_name,
                self.group_visibility,
                self.group_join_policy,
            )?,
        };
        Ok(GymBranchProfile {
            summary,
            description: self.description,
            address: GymAddress {
                line_1: self.address_line_1,
                line_2: self.address_line_2,
                area: self.area,
                city: self.city,
                district: self.district,
                region: self.region,
                country_code: self.country_code,
                postal_code: self.postal_code,
            },
            timezone: self.timezone,
            phone: self.phone,
            email: self.email,
            opening_hours,
            hour_exceptions,
            media,
            facility_details,
            equipment_inventory,
            membership_plans,
            links,
        })
    }
}

fn group_summary(
    id: Option<Uuid>,
    slug: Option<String>,
    name: Option<String>,
    visibility: Option<String>,
    join_policy: Option<String>,
) -> Result<Option<OfficialGroupSummary>, RepositoryError> {
    match (id, slug, name, visibility, join_policy) {
        (None, None, None, None, None) => Ok(None),
        (Some(id), Some(slug), Some(name), Some(visibility), Some(join_policy)) => {
            Ok(Some(OfficialGroupSummary {
                id,
                slug,
                name,
                visibility,
                join_policy,
            }))
        }
        _ => Err(RepositoryError::CorruptData(
            "official gym group fields are incomplete",
        )),
    }
}

fn distance_to_u32(value: f64) -> u32 {
    value.round().clamp(0.0, f64::from(u32::MAX)) as u32
}

#[derive(FromQueryResult)]
struct OpeningHoursRow {
    weekday: i16,
    opens_at: Option<NaiveTime>,
    closes_at: Option<NaiveTime>,
    is_closed: bool,
}

impl From<OpeningHoursRow> for OpeningHours {
    fn from(row: OpeningHoursRow) -> Self {
        Self {
            weekday: row.weekday,
            opens_at: row.opens_at,
            closes_at: row.closes_at,
            is_closed: row.is_closed,
        }
    }
}

#[derive(FromQueryResult)]
struct OpeningHoursExceptionRow {
    exception_date: NaiveDate,
    opens_at: Option<NaiveTime>,
    closes_at: Option<NaiveTime>,
    is_closed: bool,
    note: Option<String>,
}

impl From<OpeningHoursExceptionRow> for OpeningHoursException {
    fn from(row: OpeningHoursExceptionRow) -> Self {
        Self {
            date: row.exception_date,
            opens_at: row.opens_at,
            closes_at: row.closes_at,
            is_closed: row.is_closed,
            note: row.note,
        }
    }
}

#[derive(FromQueryResult)]
struct GymMediaRow {
    id: Uuid,
    media_type: String,
    url: String,
    alt_text: Option<String>,
}

impl From<GymMediaRow> for GymMedia {
    fn from(row: GymMediaRow) -> Self {
        Self {
            id: row.id,
            media_type: row.media_type,
            url: row.url,
            alt_text: row.alt_text,
        }
    }
}

#[derive(FromQueryResult)]
struct BranchFacilityRow {
    code: String,
    label: String,
    details: Option<String>,
}

impl From<BranchFacilityRow> for BranchFacility {
    fn from(row: BranchFacilityRow) -> Self {
        Self {
            code: row.code,
            label: row.label,
            details: row.details,
        }
    }
}

#[derive(FromQueryResult)]
struct BranchEquipmentRow {
    code: String,
    label: String,
    quantity: Option<i32>,
    details: Option<String>,
}

impl From<BranchEquipmentRow> for BranchEquipment {
    fn from(row: BranchEquipmentRow) -> Self {
        Self {
            code: row.code,
            label: row.label,
            quantity: row.quantity,
            details: row.details,
        }
    }
}

#[derive(FromQueryResult)]
struct MembershipPlanRow {
    id: Uuid,
    name: String,
    description: Option<String>,
    price_minor: Option<i64>,
    joining_fee_minor: Option<i64>,
    currency: Option<String>,
    billing_period: String,
}

impl From<MembershipPlanRow> for MembershipPlan {
    fn from(row: MembershipPlanRow) -> Self {
        Self {
            id: row.id,
            name: row.name,
            description: row.description,
            price_minor: row.price_minor,
            joining_fee_minor: row.joining_fee_minor,
            currency: row.currency,
            billing_period: row.billing_period,
        }
    }
}

#[derive(FromQueryResult)]
struct GymLinkRow {
    link_type: String,
    value: String,
}

impl From<GymLinkRow> for GymLink {
    fn from(row: GymLinkRow) -> Self {
        Self {
            link_type: row.link_type,
            value: row.value,
        }
    }
}

#[derive(FromQueryResult)]
struct GroupRow {
    id: Uuid,
    slug: String,
    name: String,
    description: Option<String>,
    visibility: String,
    join_policy: String,
    member_directory_visibility: String,
    branch_id: Uuid,
    branch_name: String,
    area: Option<String>,
    city: String,
    organization_name: String,
    organization_verified: bool,
}

impl From<GroupRow> for OfficialGroupProfile {
    fn from(row: GroupRow) -> Self {
        Self {
            id: row.id,
            slug: row.slug,
            name: row.name,
            description: row.description,
            visibility: row.visibility,
            join_policy: row.join_policy,
            member_directory_visibility: row.member_directory_visibility,
            branch: GroupBranch {
                id: row.branch_id,
                name: row.branch_name,
                area: row.area,
                city: row.city,
                organization_name: row.organization_name,
                organization_verified: row.organization_verified,
            },
        }
    }
}

#[derive(FromQueryResult)]
struct GroupPostRow {
    id: Uuid,
    post_type: String,
    title: String,
    body: String,
    starts_at: Option<DateTime<Utc>>,
    ends_at: Option<DateTime<Utc>>,
    published_at: DateTime<Utc>,
    is_pinned: bool,
}

impl From<GroupPostRow> for GroupPost {
    fn from(row: GroupPostRow) -> Self {
        Self {
            id: row.id,
            post_type: row.post_type,
            title: row.title,
            body: row.body,
            starts_at: row.starts_at,
            ends_at: row.ends_at,
            published_at: row.published_at,
            is_pinned: row.is_pinned,
        }
    }
}

#[derive(Debug, Error)]
pub enum RepositoryError {
    #[error("gym directory database operation failed")]
    Database(#[from] DbErr),
    #[error("gym directory data is invalid")]
    Json(#[from] serde_json::Error),
    #[error("gym directory data is inconsistent: {0}")]
    CorruptData(&'static str),
}
