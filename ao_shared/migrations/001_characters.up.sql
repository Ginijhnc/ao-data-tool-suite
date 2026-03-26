CREATE TABLE IF NOT EXISTS characters (
    id SERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL UNIQUE,
    data JSONB NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_characters_name ON characters(name);
