//! Credential diagnostics display.

use anyhow::Result;

use crate::runtime::credentials::{get_credential_diagnostics, load_env};
use crate::setup::auth;

pub fn print_credentials_diagnostics() -> Result<()> {
    let _ = load_env();
    let rows = get_credential_diagnostics()?;
    println!("Owly credential diagnostics (~/.owly/.env + process env)\n");
    for row in rows {
        let flag = if row.set { "set  " } else { "unset" };
        println!("  [{flag}] {:<28} {}", row.key, row.display);
    }
    println!("\n{}", auth::format_auth_provider_list());
    Ok(())
}
