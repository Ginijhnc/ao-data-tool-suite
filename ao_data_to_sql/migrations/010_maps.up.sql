CREATE TABLE IF NOT EXISTS maps (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL DEFAULT '',
    data JSONB NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_maps_name ON maps(name);
CREATE INDEX IF NOT EXISTS idx_maps_terrain ON maps((data->>'TERRENO'));
CREATE INDEX IF NOT EXISTS idx_maps_zone ON maps((data->>'ZONA'));
