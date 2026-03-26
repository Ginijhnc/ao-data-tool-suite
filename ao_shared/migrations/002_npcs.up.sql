CREATE TABLE IF NOT EXISTS npcs (
    id INTEGER PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    data JSONB NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_npcs_name ON npcs(name);
CREATE INDEX IF NOT EXISTS idx_npcs_npc_type ON npcs((data->>'NPCTYPE'));
