CREATE TABLE IF NOT EXISTS blacksmith_weapons (
    id INTEGER PRIMARY KEY,
    data JSONB NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_blacksmith_weapons_obj_index ON blacksmith_weapons((data->>'OBJ_INDEX'));
