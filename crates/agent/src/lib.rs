//! App-agnostic agent runtime primitives shared by Elph applications.

pub mod datastore;
pub mod init;
pub mod paths;
pub mod runtime;

pub use datastore::{DatabaseSpec, DatastoreError, Migration, ensure_database, ensure_databases};
pub use init::{InitError, InitProgress, ensure_dirs, write_file_if_missing, write_json_file, write_private_file};
pub use paths::{PathResolver, ResolvedPaths};
