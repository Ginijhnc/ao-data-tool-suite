CREATE TABLE IF NOT EXISTS carpenter_objects (
    id INTEGER PRIMARY KEY,
    data JSONB NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_carpenter_objects_obj_index ON carpenter_objects((data->>'OBJ_INDEX'));
