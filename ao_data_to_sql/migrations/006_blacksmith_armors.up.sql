CREATE TABLE IF NOT EXISTS blacksmith_armors (
    id INTEGER PRIMARY KEY,
    data JSONB NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_blacksmith_armors_obj_index ON blacksmith_armors((data->>'OBJ_INDEX'));
