-- Search support for usernames, gyms, exercises, and programs.
CREATE EXTENSION IF NOT EXISTS pg_trgm;

-- MVP gym-distance support available on the current Railway PostgreSQL instance.
-- `earthdistance` depends on `cube` and provides indexed great-circle distance
-- calculations without requiring the unavailable PostGIS extension.
CREATE EXTENSION IF NOT EXISTS cube;
CREATE EXTENSION IF NOT EXISTS earthdistance;
