//! Short session and entry ID generation (unprefixed Kalid).

use std::cell::RefCell;
use std::collections::HashMap;
use std::thread;
use std::time::Duration;

use kalid::Kalid;

use crate::session::types::SessionTreeEntry;

thread_local! {
    static LAST_KALID: RefCell<Option<String>> = const { RefCell::new(None) };
}

/// K-sortable ID string (16-char Kalid, no prefix).
pub fn create_kalid() -> String {
    next_unique_kalid()
}

pub fn generate_entry_id(by_id: &HashMap<String, SessionTreeEntry>) -> String {
    for _ in 0..100 {
        let id = next_unique_kalid();
        if !by_id.contains_key(&id) {
            return id;
        }
        thread::sleep(Duration::from_millis(1));
    }
    next_unique_kalid()
}

pub fn generate_session_id() -> String {
    next_unique_kalid()
}

/// Returns true when `id` is a valid unprefixed Kalid string.
pub fn is_valid_kalid(id: &str) -> bool {
    id.len() == 16 && Kalid::parse(id).is_ok()
}

fn next_unique_kalid() -> String {
    for _ in 0..100 {
        let id = kalid::generate_kalid();
        let duplicate = LAST_KALID.with(|cell| {
            let mut last = cell.borrow_mut();
            if last.as_deref() == Some(id.as_str()) {
                true
            } else {
                *last = Some(id.clone());
                false
            }
        });
        if !duplicate {
            return id;
        }
        thread::sleep(Duration::from_millis(1));
    }
    kalid::generate_kalid()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_kalid_is_16_chars_without_prefix() {
        let id = create_kalid();
        assert_eq!(id.len(), 16);
        assert!(!id.contains('_'));
        assert!(is_valid_kalid(&id));
    }

    #[test]
    fn generate_session_id_is_valid_kalid() {
        assert!(is_valid_kalid(&generate_session_id()));
    }

    #[test]
    fn rapid_create_kalid_produces_distinct_ids() {
        let ids: std::collections::HashSet<String> = (0..8).map(|_| create_kalid()).collect();
        assert_eq!(ids.len(), 8);
    }
}
