CREATE TABLE IF NOT EXISTS spells (
    id INTEGER PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    data JSONB NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_spells_name ON spells(name);
CREATE INDEX IF NOT EXISTS idx_spells_tipo ON spells((data->>'TIPO'));
