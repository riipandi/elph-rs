//! Versioned SQL migrations for local Turso/SQLite databases.
//!
//! Prefer `STRICT` tables using only `INTEGER`, `REAL`, `TEXT`, and `BLOB` column types
//! (use `TEXT` for timestamps, `INTEGER` for booleans).

/// One versioned SQL migration applied to a local database.
pub struct Migration {
    pub version: i64,
    pub name: &'static str,
    pub up: &'static str,
}
