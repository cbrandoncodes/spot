use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let database = manager.get_connection();

        for statement in [
            r#"
                CREATE TABLE gym_organizations (
                    id UUID PRIMARY KEY DEFAULT pg_catalog.gen_random_uuid(),
                    slug TEXT NOT NULL UNIQUE,
                    name TEXT NOT NULL,
                    description TEXT NULL,
                    logo_url TEXT NULL,
                    website_url TEXT NULL,
                    verification_status TEXT NOT NULL DEFAULT 'unverified',
                    verified_at TIMESTAMPTZ NULL,
                    visibility TEXT NOT NULL DEFAULT 'public',
                    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                    CONSTRAINT gym_organizations_slug_format CHECK (
                        slug ~ '^[a-z0-9]+(-[a-z0-9]+)*$' AND char_length(slug) <= 80
                    ),
                    CONSTRAINT gym_organizations_name_length CHECK (char_length(name) BETWEEN 1 AND 120),
                    CONSTRAINT gym_organizations_description_length CHECK (
                        description IS NULL OR char_length(description) <= 1200
                    ),
                    CONSTRAINT gym_organizations_verification_values CHECK (
                        verification_status IN ('unverified', 'pending', 'verified', 'suspended')
                    ),
                    CONSTRAINT gym_organizations_verification_pair CHECK (
                        verification_status <> 'verified' OR verified_at IS NOT NULL
                    ),
                    CONSTRAINT gym_organizations_visibility_values CHECK (
                        visibility IN ('public', 'unlisted', 'private')
                    )
                )
            "#,
            r#"
                CREATE TABLE gym_branches (
                    id UUID PRIMARY KEY DEFAULT pg_catalog.gen_random_uuid(),
                    organization_id UUID NOT NULL REFERENCES gym_organizations(id) ON DELETE CASCADE,
                    slug TEXT NOT NULL,
                    name TEXT NOT NULL,
                    description TEXT NULL,
                    address_line_1 TEXT NOT NULL,
                    address_line_2 TEXT NULL,
                    area TEXT NULL,
                    city TEXT NOT NULL,
                    district TEXT NULL,
                    region TEXT NULL,
                    country_code VARCHAR(2) NOT NULL,
                    postal_code TEXT NULL,
                    latitude DOUBLE PRECISION NOT NULL,
                    longitude DOUBLE PRECISION NOT NULL,
                    timezone TEXT NOT NULL,
                    phone TEXT NULL,
                    email TEXT NULL,
                    status TEXT NOT NULL DEFAULT 'active',
                    visibility TEXT NOT NULL DEFAULT 'public',
                    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                    CONSTRAINT gym_branches_org_slug_unique UNIQUE (organization_id, slug),
                    CONSTRAINT gym_branches_slug_format CHECK (
                        slug ~ '^[a-z0-9]+(-[a-z0-9]+)*$' AND char_length(slug) <= 80
                    ),
                    CONSTRAINT gym_branches_name_length CHECK (char_length(name) BETWEEN 1 AND 120),
                    CONSTRAINT gym_branches_description_length CHECK (
                        description IS NULL OR char_length(description) <= 1200
                    ),
                    CONSTRAINT gym_branches_country_format CHECK (country_code ~ '^[A-Z]{2}$'),
                    CONSTRAINT gym_branches_latitude_range CHECK (latitude BETWEEN -90 AND 90),
                    CONSTRAINT gym_branches_longitude_range CHECK (longitude BETWEEN -180 AND 180),
                    CONSTRAINT gym_branches_status_values CHECK (
                        status IN ('active', 'temporarily_closed', 'permanently_closed')
                    ),
                    CONSTRAINT gym_branches_visibility_values CHECK (
                        visibility IN ('public', 'unlisted', 'private')
                    )
                )
            "#,
            r#"
                CREATE INDEX gym_branches_location_idx
                ON gym_branches USING GIST (ll_to_earth(latitude, longitude))
            "#,
            r#"
                CREATE INDEX gym_branches_name_search_idx
                ON gym_branches USING GIN (lower(name) gin_trgm_ops)
            "#,
            r#"
                CREATE INDEX gym_branches_area_search_idx
                ON gym_branches USING GIN (
                    lower(coalesce(area, '') || ' ' || city || ' ' || coalesce(district, '') || ' ' || coalesce(region, '')) gin_trgm_ops
                )
            "#,
            r#"
                CREATE INDEX gym_organizations_name_search_idx
                ON gym_organizations USING GIN (lower(name) gin_trgm_ops)
            "#,
            r#"
                CREATE TABLE gym_branch_opening_hours (
                    branch_id UUID NOT NULL REFERENCES gym_branches(id) ON DELETE CASCADE,
                    weekday SMALLINT NOT NULL,
                    opens_at TIME NULL,
                    closes_at TIME NULL,
                    is_closed BOOLEAN NOT NULL DEFAULT FALSE,
                    PRIMARY KEY (branch_id, weekday),
                    CONSTRAINT gym_opening_hours_weekday_range CHECK (weekday BETWEEN 1 AND 7),
                    CONSTRAINT gym_opening_hours_values CHECK (
                        (is_closed AND opens_at IS NULL AND closes_at IS NULL)
                        OR (NOT is_closed AND opens_at IS NOT NULL AND closes_at IS NOT NULL)
                    )
                )
            "#,
            r#"
                CREATE TABLE gym_branch_hour_exceptions (
                    id UUID PRIMARY KEY DEFAULT pg_catalog.gen_random_uuid(),
                    branch_id UUID NOT NULL REFERENCES gym_branches(id) ON DELETE CASCADE,
                    exception_date DATE NOT NULL,
                    opens_at TIME NULL,
                    closes_at TIME NULL,
                    is_closed BOOLEAN NOT NULL DEFAULT FALSE,
                    note TEXT NULL,
                    CONSTRAINT gym_hour_exceptions_branch_date_unique UNIQUE (branch_id, exception_date),
                    CONSTRAINT gym_hour_exceptions_values CHECK (
                        (is_closed AND opens_at IS NULL AND closes_at IS NULL)
                        OR (NOT is_closed AND opens_at IS NOT NULL AND closes_at IS NOT NULL)
                    ),
                    CONSTRAINT gym_hour_exceptions_note_length CHECK (
                        note IS NULL OR char_length(note) <= 120
                    )
                )
            "#,
            r#"
                CREATE TABLE gym_media (
                    id UUID PRIMARY KEY DEFAULT pg_catalog.gen_random_uuid(),
                    branch_id UUID NOT NULL REFERENCES gym_branches(id) ON DELETE CASCADE,
                    media_type TEXT NOT NULL,
                    url TEXT NOT NULL,
                    alt_text TEXT NULL,
                    sort_order SMALLINT NOT NULL DEFAULT 0,
                    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                    CONSTRAINT gym_media_type_values CHECK (media_type IN ('cover', 'image')),
                    CONSTRAINT gym_media_url_length CHECK (char_length(url) BETWEEN 1 AND 2048),
                    CONSTRAINT gym_media_alt_length CHECK (alt_text IS NULL OR char_length(alt_text) <= 160),
                    CONSTRAINT gym_media_sort_nonnegative CHECK (sort_order >= 0)
                )
            "#,
            r#"
                CREATE TABLE gym_facility_types (
                    code TEXT PRIMARY KEY,
                    label TEXT NOT NULL,
                    CONSTRAINT gym_facility_types_code_format CHECK (
                        code ~ '^[a-z0-9]+(_[a-z0-9]+)*$' AND char_length(code) <= 64
                    ),
                    CONSTRAINT gym_facility_types_label_length CHECK (char_length(label) BETWEEN 1 AND 80)
                )
            "#,
            r#"
                INSERT INTO gym_facility_types (code, label) VALUES
                    ('accessibility', 'Accessible facilities'),
                    ('air_conditioning', 'Air conditioning'),
                    ('changing_rooms', 'Changing rooms'),
                    ('child_care', 'Child care'),
                    ('group_classes', 'Group classes'),
                    ('lockers', 'Lockers'),
                    ('open_24_hours', 'Open 24 hours'),
                    ('parking', 'Parking'),
                    ('personal_training', 'Personal training'),
                    ('pool', 'Swimming pool'),
                    ('sauna', 'Sauna'),
                    ('showers', 'Showers'),
                    ('wifi', 'Wi-Fi')
            "#,
            r#"
                CREATE TABLE gym_branch_facilities (
                    branch_id UUID NOT NULL REFERENCES gym_branches(id) ON DELETE CASCADE,
                    facility_code TEXT NOT NULL REFERENCES gym_facility_types(code) ON DELETE RESTRICT,
                    details TEXT NULL,
                    PRIMARY KEY (branch_id, facility_code),
                    CONSTRAINT gym_branch_facilities_details_length CHECK (
                        details IS NULL OR char_length(details) <= 240
                    )
                )
            "#,
            r#"
                CREATE TABLE gym_equipment_types (
                    code TEXT PRIMARY KEY,
                    label TEXT NOT NULL,
                    CONSTRAINT gym_equipment_types_code_format CHECK (
                        code ~ '^[a-z0-9]+(_[a-z0-9]+)*$' AND char_length(code) <= 64
                    ),
                    CONSTRAINT gym_equipment_types_label_length CHECK (char_length(label) BETWEEN 1 AND 80)
                )
            "#,
            r#"
                INSERT INTO gym_equipment_types (code, label) VALUES
                    ('adjustable_bench', 'Adjustable bench'),
                    ('barbell', 'Barbell'),
                    ('bodyweight', 'Bodyweight area'),
                    ('cable_station', 'Cable station'),
                    ('cardio_machines', 'Cardio machines'),
                    ('dumbbells', 'Dumbbells'),
                    ('kettlebells', 'Kettlebells'),
                    ('lat_pulldown', 'Lat pulldown'),
                    ('leg_press', 'Leg press'),
                    ('pull_up_bar', 'Pull-up bar'),
                    ('resistance_bands', 'Resistance bands'),
                    ('smith_machine', 'Smith machine'),
                    ('squat_rack', 'Squat rack'),
                    ('weight_plates', 'Weight plates')
            "#,
            r#"
                CREATE TABLE gym_branch_equipment (
                    branch_id UUID NOT NULL REFERENCES gym_branches(id) ON DELETE CASCADE,
                    equipment_code TEXT NOT NULL REFERENCES gym_equipment_types(code) ON DELETE RESTRICT,
                    quantity INTEGER NULL,
                    details TEXT NULL,
                    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                    PRIMARY KEY (branch_id, equipment_code),
                    CONSTRAINT gym_branch_equipment_quantity_positive CHECK (quantity IS NULL OR quantity > 0),
                    CONSTRAINT gym_branch_equipment_details_length CHECK (
                        details IS NULL OR char_length(details) <= 240
                    )
                )
            "#,
            r#"
                CREATE TABLE gym_membership_plans (
                    id UUID PRIMARY KEY DEFAULT pg_catalog.gen_random_uuid(),
                    branch_id UUID NOT NULL REFERENCES gym_branches(id) ON DELETE CASCADE,
                    name TEXT NOT NULL,
                    description TEXT NULL,
                    price_minor BIGINT NULL,
                    joining_fee_minor BIGINT NULL,
                    currency VARCHAR(3) NULL,
                    billing_period TEXT NOT NULL,
                    is_active BOOLEAN NOT NULL DEFAULT TRUE,
                    sort_order SMALLINT NOT NULL DEFAULT 0,
                    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                    CONSTRAINT gym_membership_plans_name_length CHECK (char_length(name) BETWEEN 1 AND 100),
                    CONSTRAINT gym_membership_plans_description_length CHECK (
                        description IS NULL OR char_length(description) <= 600
                    ),
                    CONSTRAINT gym_membership_plans_price_nonnegative CHECK (
                        price_minor IS NULL OR price_minor >= 0
                    ),
                    CONSTRAINT gym_membership_plans_joining_fee_nonnegative CHECK (
                        joining_fee_minor IS NULL OR joining_fee_minor >= 0
                    ),
                    CONSTRAINT gym_membership_plans_currency_pair CHECK (
                        (price_minor IS NULL AND joining_fee_minor IS NULL AND currency IS NULL)
                        OR (currency ~ '^[A-Z]{3}$')
                    ),
                    CONSTRAINT gym_membership_plans_period_values CHECK (
                        billing_period IN ('day', 'week', 'month', 'quarter', 'year', 'one_time', 'contact_gym')
                    ),
                    CONSTRAINT gym_membership_plans_sort_nonnegative CHECK (sort_order >= 0)
                )
            "#,
            r#"
                CREATE TABLE gym_links (
                    id UUID PRIMARY KEY DEFAULT pg_catalog.gen_random_uuid(),
                    branch_id UUID NOT NULL REFERENCES gym_branches(id) ON DELETE CASCADE,
                    link_type TEXT NOT NULL,
                    value TEXT NOT NULL,
                    sort_order SMALLINT NOT NULL DEFAULT 0,
                    CONSTRAINT gym_links_type_values CHECK (
                        link_type IN ('website', 'instagram', 'facebook', 'tiktok', 'x', 'whatsapp', 'email', 'phone')
                    ),
                    CONSTRAINT gym_links_value_length CHECK (char_length(value) BETWEEN 1 AND 2048),
                    CONSTRAINT gym_links_sort_nonnegative CHECK (sort_order >= 0)
                )
            "#,
            r#"
                CREATE TABLE official_gym_groups (
                    id UUID PRIMARY KEY DEFAULT pg_catalog.gen_random_uuid(),
                    branch_id UUID NOT NULL UNIQUE REFERENCES gym_branches(id) ON DELETE CASCADE,
                    slug TEXT NOT NULL UNIQUE,
                    name TEXT NOT NULL,
                    description TEXT NULL,
                    visibility TEXT NOT NULL DEFAULT 'public',
                    join_policy TEXT NOT NULL DEFAULT 'open',
                    member_directory_visibility TEXT NOT NULL DEFAULT 'members',
                    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                    CONSTRAINT official_gym_groups_slug_format CHECK (
                        slug ~ '^[a-z0-9]+(-[a-z0-9]+)*$' AND char_length(slug) <= 100
                    ),
                    CONSTRAINT official_gym_groups_name_length CHECK (char_length(name) BETWEEN 1 AND 120),
                    CONSTRAINT official_gym_groups_description_length CHECK (
                        description IS NULL OR char_length(description) <= 1200
                    ),
                    CONSTRAINT official_gym_groups_visibility_values CHECK (
                        visibility IN ('public', 'unlisted', 'private')
                    ),
                    CONSTRAINT official_gym_groups_join_policy_values CHECK (
                        join_policy IN ('open', 'approval', 'invite_only')
                    ),
                    CONSTRAINT official_gym_groups_directory_values CHECK (
                        member_directory_visibility IN ('hidden', 'members', 'public')
                    )
                )
            "#,
            r#"
                CREATE TABLE gym_group_posts (
                    id UUID PRIMARY KEY DEFAULT pg_catalog.gen_random_uuid(),
                    group_id UUID NOT NULL REFERENCES official_gym_groups(id) ON DELETE CASCADE,
                    author_user_id UUID NULL REFERENCES auth_users(id) ON DELETE SET NULL,
                    post_type TEXT NOT NULL,
                    title TEXT NOT NULL,
                    body TEXT NOT NULL,
                    starts_at TIMESTAMPTZ NULL,
                    ends_at TIMESTAMPTZ NULL,
                    published_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                    is_pinned BOOLEAN NOT NULL DEFAULT FALSE,
                    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                    CONSTRAINT gym_group_posts_type_values CHECK (
                        post_type IN (
                            'announcement', 'maintenance', 'class_event', 'promotion',
                            'hours_change', 'new_equipment', 'membership'
                        )
                    ),
                    CONSTRAINT gym_group_posts_title_length CHECK (char_length(title) BETWEEN 1 AND 160),
                    CONSTRAINT gym_group_posts_body_length CHECK (char_length(body) BETWEEN 1 AND 5000),
                    CONSTRAINT gym_group_posts_time_order CHECK (ends_at IS NULL OR starts_at IS NULL OR ends_at > starts_at)
                )
            "#,
            r#"
                CREATE INDEX gym_group_posts_feed_idx
                ON gym_group_posts (group_id, is_pinned DESC, published_at DESC)
            "#,
        ] {
            database.execute_unprepared(statement).await?;
        }

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let database = manager.get_connection();
        for statement in [
            "DROP TABLE IF EXISTS gym_group_posts",
            "DROP TABLE IF EXISTS official_gym_groups",
            "DROP TABLE IF EXISTS gym_links",
            "DROP TABLE IF EXISTS gym_membership_plans",
            "DROP TABLE IF EXISTS gym_branch_equipment",
            "DROP TABLE IF EXISTS gym_equipment_types",
            "DROP TABLE IF EXISTS gym_branch_facilities",
            "DROP TABLE IF EXISTS gym_facility_types",
            "DROP TABLE IF EXISTS gym_media",
            "DROP TABLE IF EXISTS gym_branch_hour_exceptions",
            "DROP TABLE IF EXISTS gym_branch_opening_hours",
            "DROP TABLE IF EXISTS gym_branches",
            "DROP TABLE IF EXISTS gym_organizations",
        ] {
            database.execute_unprepared(statement).await?;
        }
        Ok(())
    }
}
