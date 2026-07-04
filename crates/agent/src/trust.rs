use serde_json::{Map, Value};

use crate::appdir::Paths;

pub struct TrustStore;

impl TrustStore {
    pub fn ensure(paths: &Paths) -> crate::init::Result<()> {
        let path = paths.trust_path();
        if path.exists() {
            return Ok(());
        }

        let empty = Value::Object(Map::new());
        crate::init::write_json_file(&path, &empty)
    }
}
