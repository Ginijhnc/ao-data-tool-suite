CREATE TABLE export_manifest (
    file_key    VARCHAR(255) PRIMARY KEY,
    hash        CHAR(64) NOT NULL,
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
