//! Connector ingestion progress display.

use crate::connectors::ConnectorIngestResult;

pub fn print_ingest_header(connector_id: &str, instance_id: &str) {
    println!("\n>_ Owly ingest {connector_id} ({instance_id})");
}

pub fn print_pull_result(pull: &ConnectorIngestResult) {
    println!("  pull: {}", pull.message);
    for warning in &pull.warnings {
        println!("  warn: {warning}");
    }
}

pub fn print_wiki_skipped() {
    println!("  wiki: skipped (no new evidence)");
}

pub fn print_wiki_update_complete() {
    println!("  wiki: update complete");
}

pub fn print_wiki_completion_message(message: &str) {
    println!("{message}");
}
