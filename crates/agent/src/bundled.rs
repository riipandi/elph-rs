use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use crate::appdir::Paths;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BundledManifest {
    pub version: String,
    #[serde(default)]
    pub checksums: BTreeMap<String, String>,
}

impl BundledManifest {
    pub fn defaults(app_version: &str) -> Self {
        Self {
            version: format!("elph-{app_version}"),
            checksums: BTreeMap::new(),
        }
    }

    pub fn ensure(paths: &Paths, app_version: &str) -> crate::init::Result<()> {
        let path = paths.bundled_manifest_path();
        if path.exists() {
            return Ok(());
        }

        crate::init::write_json_file(&path, &Self::defaults(app_version))
    }
}
