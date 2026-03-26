CREATE TABLE IF NOT EXISTS objects (
    id INTEGER PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    data JSONB NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_objects_name ON objects(name);
CREATE INDEX IF NOT EXISTS idx_objects_obj_type ON objects((data->>'OBJTYPE'));
