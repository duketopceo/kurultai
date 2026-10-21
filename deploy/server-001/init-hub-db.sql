-- init-hub-db.sql — one-time hub DB bootstrap (hub-adoption U1, R11/KTD8).
--
-- Apply ONCE as the kurultai-hub-db superuser, BEFORE the first migrate, so
-- kurultai_migrate ends up owning the hub tables (RLS ALTER requires table
-- ownership):
--
--   psql "$KURULTAI_HUB_SUPERUSER_DSN" \
--     -v app_password="$KURULTAI_HUB_DB_PASSWORD" \
--     -v migrate_password="$KURULTAI_HUB_MIGRATE_DB_PASSWORD" \
--     -f deploy/server-001/init-hub-db.sql
--
-- Safe to re-run: role creation is wrapped in DO blocks, ALTER ROLE always
-- re-asserts attributes + password, and GRANT/ALTER DEFAULT PRIVILEGES are
-- idempotent.
--
-- Roles (KTD8):
--   kurultai_migrate — LOGIN BYPASSRLS. Schema migrations + `kurultai hub …`
--     CLI via KURULTAI_MIGRATE_DATABASE_URL. Owns hub objects → can DDL/RLS.
--   kurultai_app     — LOGIN NOINHERIT, non-owner, DML-only grants. Runtime
--     DATABASE_URL for the kurultai-hub service (with
--     KURULTAI_PG_SKIP_MIGRATE=1). FORCE RLS applies on every row.

\set ON_ERROR_STOP on

DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'kurultai_migrate') THEN
        CREATE ROLE kurultai_migrate;
    END IF;
    IF NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'kurultai_app') THEN
        CREATE ROLE kurultai_app;
    END IF;
END
$$;

-- Re-assert attributes + rotate passwords on every run (idempotent).
ALTER ROLE kurultai_migrate WITH LOGIN BYPASSRLS PASSWORD :'migrate_password';
ALTER ROLE kurultai_app     WITH LOGIN NOINHERIT PASSWORD :'app_password';

-- CONNECT/TEMP come from PUBLIC by default; stated explicitly so the file is
-- self-contained if defaults are ever tightened. Runs inside the target DB.
DO $$
BEGIN
    EXECUTE format(
        'GRANT CONNECT, TEMPORARY ON DATABASE %I TO kurultai_app, kurultai_migrate',
        current_database()
    );
END
$$;

GRANT USAGE ON SCHEMA public TO kurultai_app, kurultai_migrate;
-- The migrate role creates every hub object (tables, indexes, RLS policies).
GRANT CREATE ON SCHEMA public TO kurultai_migrate;

-- Runtime DML for the app role on every table the hub touches. ALL TABLES
-- covers whatever exists at run time; the ALTER DEFAULT PRIVILEGES below
-- cover tables created later (e.g. hub_device_flows in U3).
GRANT SELECT, INSERT, UPDATE, DELETE ON ALL TABLES IN SCHEMA public TO kurultai_app;
GRANT USAGE, SELECT ON ALL SEQUENCES IN SCHEMA public TO kurultai_app;

-- The migrate role keeps full rights on anything it didn't create (e.g.
-- tables that already existed when this file first ran).
GRANT ALL PRIVILEGES ON ALL TABLES IN SCHEMA public TO kurultai_migrate;
GRANT ALL PRIVILEGES ON ALL SEQUENCES IN SCHEMA public TO kurultai_migrate;

-- Objects created later by the migrate role auto-grant DML to the app role —
-- a new table can never silently strand the runtime DSN.
ALTER DEFAULT PRIVILEGES FOR ROLE kurultai_migrate IN SCHEMA public
    GRANT SELECT, INSERT, UPDATE, DELETE ON TABLES TO kurultai_app;
ALTER DEFAULT PRIVILEGES FOR ROLE kurultai_migrate IN SCHEMA public
    GRANT USAGE, SELECT ON SEQUENCES TO kurultai_app;

-- Same coverage when migrations run as the role executing this file (dev/CI
-- superuser). In prod the migrate role is the only creator; harmless extra.
ALTER DEFAULT PRIVILEGES IN SCHEMA public
    GRANT SELECT, INSERT, UPDATE, DELETE ON TABLES TO kurultai_app;
ALTER DEFAULT PRIVILEGES IN SCHEMA public
    GRANT USAGE, SELECT ON SEQUENCES TO kurultai_app;
