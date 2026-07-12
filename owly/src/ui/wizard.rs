//! Interactive wizard headers (onboarding, wiki brief prompts).

use std::path::Path;

use crate::runtime::constants::OWLY_DIR;
use crate::wiki::instructions::INSTRUCTIONS_FILE;

pub fn print_setup_header() {
    println!();
    println!("\x1b[36;1m>_ Owly setup\x1b[0m");
    println!("Configure your inference provider and credentials.");
    println!();
}

pub fn print_oauth_sign_in(provider_label: &str) {
    println!();
    println!("Sign in with {provider_label} (browser flow).");
}

pub fn print_oauth_signed_in() {
    println!("\x1b[32m✓\x1b[0m Signed in.");
}

pub fn print_credentials_saved(path: &Path) {
    println!();
    println!("\x1b[32m✓\x1b[0m Credentials saved to {}", path.display());
    println!();
}

pub fn print_repository_wiki_brief_header() {
    println!();
    println!("\x1b[36;1m>_ Owly wiki brief\x1b[0m");
    println!("Describe what this wiki should understand (scope, priorities, audience).");
    println!("Saved to {OWLY_DIR}/{INSTRUCTIONS_FILE} and injected into init/update prompts.");
    println!();
}

pub fn print_personal_wiki_brief_header() {
    println!();
    println!("\x1b[36;1m>_ Owly personal wiki brief\x1b[0m");
    println!("Describe what your personal wiki should track (scope, priorities, audience).");
    println!("Saved to ~/.owly/INSTRUCTIONS.md and injected into init/update prompts.");
    println!();
}
