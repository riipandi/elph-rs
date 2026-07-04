use turso::Connection;

use super::DatastoreError;

pub struct Migration {
    pub version: i64,
    pub name: &'static str,
    pub up: &'static str,
}

pub async fn run(conn: &Connection, migrations: &[Migration]) -> Result<(), DatastoreError> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS app_migrations (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            version INTEGER NOT NULL,
            name TEXT NOT NULL,
            applied_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
        )",
        (),
    )
    .await?;

    conn.execute(
        "CREATE UNIQUE INDEX IF NOT EXISTS idx_app_migrations_version
         ON app_migrations(version)",
        (),
    )
    .await?;

    let mut rows = conn
        .query("SELECT COALESCE(MAX(version), 0) FROM app_migrations", ())
        .await?;

    let current_version = if let Some(row) = rows.next().await? {
        row.get::<i64>(0)?
    } else {
        0
    };

    for migration in migrations {
        if migration.version <= current_version {
            continue;
        }

        for statement in split_sql(migration.up) {
            conn.execute(statement, ()).await?;
        }

        conn.execute(
            "INSERT INTO app_migrations (version, name) VALUES (?, ?)",
            (migration.version, migration.name),
        )
        .await?;
    }

    Ok(())
}

pub fn metadata_migrations() -> &'static [Migration] {
    &[
        Migration {
            version: 1,
            name: "create_sessions_table",
            up: "CREATE TABLE IF NOT EXISTS sessions (
                    id TEXT PRIMARY KEY,
                    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
                    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
                    work_dir TEXT,
                    provider_id TEXT,
                    model_id TEXT,
                    agent_mode TEXT DEFAULT 'build',
                    system_prompt TEXT,
                    metadata TEXT
                );
                CREATE INDEX IF NOT EXISTS idx_sessions_created_at ON sessions(created_at);
                CREATE INDEX IF NOT EXISTS idx_sessions_work_dir ON sessions(work_dir);",
        },
        Migration {
            version: 2,
            name: "create_messages_table",
            up: "CREATE TABLE IF NOT EXISTS messages (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    session_id TEXT NOT NULL,
                    role TEXT NOT NULL,
                    content TEXT,
                    tool_call_id TEXT,
                    tool_calls TEXT,
                    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
                    FOREIGN KEY (session_id) REFERENCES sessions(id) ON DELETE CASCADE
                );
                CREATE INDEX IF NOT EXISTS idx_messages_session_id ON messages(session_id);
                CREATE INDEX IF NOT EXISTS idx_messages_created_at ON messages(created_at);",
        },
        Migration {
            version: 3,
            name: "create_todos_table",
            up: "CREATE TABLE IF NOT EXISTS todos (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    session_id TEXT NOT NULL,
                    content TEXT NOT NULL,
                    completed BOOLEAN NOT NULL DEFAULT 0,
                    position INTEGER NOT NULL DEFAULT 0,
                    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
                    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
                    FOREIGN KEY (session_id) REFERENCES sessions(id) ON DELETE CASCADE
                );
                CREATE INDEX IF NOT EXISTS idx_todos_session_id ON todos(session_id);
                CREATE INDEX IF NOT EXISTS idx_todos_position ON todos(session_id, position);",
        },
        Migration {
            version: 4,
            name: "create_goals_table",
            up: "CREATE TABLE IF NOT EXISTS goals (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    session_id TEXT NOT NULL,
                    objective TEXT NOT NULL,
                    completion_criterion TEXT,
                    status TEXT NOT NULL DEFAULT 'active',
                    turns_used INTEGER NOT NULL DEFAULT 0,
                    tokens_used INTEGER NOT NULL DEFAULT 0,
                    wall_clock_ms INTEGER NOT NULL DEFAULT 0,
                    wall_clock_budget_ms INTEGER NOT NULL DEFAULT 0,
                    turn_budget INTEGER NOT NULL DEFAULT 0,
                    token_budget INTEGER NOT NULL DEFAULT 0,
                    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
                    completed_at DATETIME,
                    FOREIGN KEY (session_id) REFERENCES sessions(id) ON DELETE CASCADE
                );
                CREATE INDEX IF NOT EXISTS idx_goals_session_id ON goals(session_id);
                CREATE INDEX IF NOT EXISTS idx_goals_status ON goals(status);",
        },
        Migration {
            version: 5,
            name: "create_skill_cache_table",
            up: "CREATE TABLE IF NOT EXISTS skill_cache (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    skill_name TEXT NOT NULL,
                    skill_hash TEXT NOT NULL,
                    content TEXT NOT NULL,
                    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
                    expires_at DATETIME,
                    UNIQUE(skill_name, skill_hash)
                );
                CREATE INDEX IF NOT EXISTS idx_skill_cache_name ON skill_cache(skill_name);
                CREATE INDEX IF NOT EXISTS idx_skill_cache_expires ON skill_cache(expires_at);",
        },
    ]
}

pub fn memory_migrations() -> &'static [Migration] {
    &[Migration {
        version: 1,
        name: "create_memories_table",
        up: "CREATE TABLE IF NOT EXISTS memories (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                workspace_id TEXT NOT NULL,
                category TEXT NOT NULL,
                content TEXT NOT NULL,
                metadata TEXT,
                created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
            );
            CREATE INDEX IF NOT EXISTS idx_memories_workspace_id ON memories(workspace_id);
            CREATE INDEX IF NOT EXISTS idx_memories_category ON memories(category);",
    }]
}

fn split_sql(sql: &str) -> Vec<&str> {
    sql.split(';')
        .map(str::trim)
        .filter(|statement| !statement.is_empty())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_sql_handles_multiple_statements() {
        let parts = split_sql("CREATE TABLE a (id INT); CREATE INDEX idx ON a(id);");
        assert_eq!(parts.len(), 2);
    }
}
