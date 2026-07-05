//! Default home-directory files scaffolded on first run.
//!
//! Each type writes a minimal placeholder file when missing so `elph` and
//! `eclaw` can bootstrap their config/data trees before app-specific setup.

mod bundled;
mod trust;
mod version;

pub use bundled::BundledManifest;
pub use trust::TrustStore;
pub use version::VersionFile;
