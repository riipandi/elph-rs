//! Agent runtime for Elph coding assistant applications.

pub mod appdir;
pub mod bundled;
pub mod datastore;
pub mod init;
pub mod runtime;
pub mod settings;
pub mod trust;
pub mod version;

pub use appdir::Paths;
pub use init::{InitError, ensure, ensure_blocking, ensure_with_paths};
