use elph_agent::write_json_file;
use serde_json::{Map, Value};

use super::InitError;
use super::paths::Paths;

pub struct TrustStore;

impl TrustStore {
    pub fn ensure(paths: &Paths) -> Result<(), InitError> {
        let path = paths.trust_path();
        if path.exists() {
            return Ok(());
        }

        let empty = Value::Object(Map::new());
        write_json_file(&path, &empty)?;
        Ok(())
    }
}
