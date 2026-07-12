//! Auth command output.

use crate::setup::auth::{ConfigureResult, format_auth_provider_list};

pub fn print_auth_provider_list() {
    println!("{}", format_auth_provider_list());
}

pub fn print_configure_result(result: &ConfigureResult) {
    println!(
        "\x1b[32m✓\x1b[0m Connector config {} at {}",
        result.status,
        result.config_path.display()
    );
    for step in &result.next_steps {
        println!("  - {step}");
    }
}
