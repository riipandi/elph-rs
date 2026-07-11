use elph_agent::Migration;

pub fn metadata_migrations() -> &'static [Migration] {
    &[
        Migration {
            version: 1,
            name: "create_sessions_table",
            up: "CREATE TABLE IF NOT EXISTS sessions (
                    id TEXT PRIMARY KEY,
                    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
                    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
                    persona TEXT,
                    provider_id TEXT,
                    model_id TEXT,
                    title TEXT,
                    metadata TEXT
                ) STRICT;
                CREATE INDEX IF NOT EXISTS idx_sessions_created_at ON sessions(created_at);
                CREATE INDEX IF NOT EXISTS idx_sessions_updated_at ON sessions(updated_at);",
        },
        Migration {
            version: 2,
            name: "create_messages_table",
            up: "CREATE TABLE IF NOT EXISTS messages (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    session_id TEXT NOT NULL,
                    role TEXT NOT NULL,
                    content TEXT,
                    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
                    FOREIGN KEY (session_id) REFERENCES sessions(id) ON DELETE CASCADE
                ) STRICT;
                CREATE INDEX IF NOT EXISTS idx_messages_session_id ON messages(session_id);
                CREATE INDEX IF NOT EXISTS idx_messages_created_at ON messages(created_at);",
        },
    ]
}

pub fn memory_migrations() -> &'static [Migration] {
    &[Migration {
        version: 1,
        name: "create_memories_table",
        up: "CREATE TABLE IF NOT EXISTS memories (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                topic TEXT NOT NULL,
                content TEXT NOT NULL,
                source TEXT,
                metadata TEXT,
                created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
                updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
            ) STRICT;
            CREATE INDEX IF NOT EXISTS idx_memories_topic ON memories(topic);",
    }]
}
