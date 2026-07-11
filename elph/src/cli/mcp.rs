use clap::{Parser, Subcommand};

use super::help;
use elph_core::utils::path::AppPaths;

use crate::platform::mcp as mcp_runtime;
use crate::platform::{EXIT_ERROR, EXIT_SUCCESS, ExitCode, Paths, Settings, ensure_home_blocking};
use elph_agent::{McpServerConfig, clear_credentials, has_stored_credentials, probe_server_with_auth, run_oauth_flow};

#[derive(Parser, Default)]
#[command(
    name = "mcp",
    about = "Manage MCP server configurations",
    color = clap::ColorChoice::Auto
)]
pub struct McpArgs {
    #[command(subcommand)]
    pub command: Option<McpCommands>,
}

#[derive(Subcommand)]
pub enum McpCommands {
    /// List configured MCP servers
    List,
    /// Add or update an MCP server configuration
    Add {
        /// Name of the MCP server
        name: String,
        /// MCP server configuration (JSON string or file path)
        #[arg(value_name = "CONFIG")]
        config: Option<String>,
    },
    /// Remove an MCP server configuration
    Remove {
        /// Name of the MCP server to remove
        name: String,
    },
    /// Diagnose MCP server configuration and connectivity
    Doctor,
    /// Authenticate with an OAuth-enabled MCP server
    Auth {
        /// Name of the MCP server
        name: String,
        /// OAuth scopes (space-separated). Defaults to server config or empty.
        #[arg(long, value_delimiter = ' ')]
        scopes: Vec<String>,
    },
    /// Remove OAuth credentials for an MCP server
    Logout {
        /// Name of the MCP server
        name: String,
    },
}

pub fn handle(args: &McpArgs) -> ExitCode {
    let Some(cmd) = &args.command else {
        return help::print_subcommand_help::<McpArgs>();
    };

    let paths = match ensure_home_blocking(env!("CARGO_PKG_VERSION")) {
        Ok(paths) => paths,
        Err(error) => {
            eprintln!("{error}");
            return EXIT_ERROR;
        }
    };

    match cmd {
        McpCommands::List => match mcp_runtime::load_config(&paths) {
            Ok(config) => {
                if config.servers.is_empty() {
                    println!("No MCP servers configured.");
                    println!("Config: {}", paths.mcp_config_path().display());
                } else {
                    let auth_store_path = paths.auth_store_path();
                    for (name, server) in &config.servers {
                        let disabled = if server.is_disabled() { " [disabled]" } else { "" };
                        let oauth = if has_stored_credentials(&auth_store_path, name) {
                            " [oauth:authorized]"
                        } else if server.wants_oauth() {
                            " [oauth:needed]"
                        } else {
                            ""
                        };
                        println!("{name}: type={}{disabled}{oauth}", server.kind_label());
                        if let Some(url) = server.remote_url() {
                            println!("  url: {url}");
                        }
                        if let McpServerConfig::Stdio(c) = server {
                            println!("  command: {} {:?}", c.command, c.args);
                        }
                    }
                }
                EXIT_SUCCESS
            }
            Err(error) => {
                eprintln!("{error}");
                EXIT_ERROR
            }
        },
        McpCommands::Add { name, config } => {
            let Some(raw) = config else {
                help::unimplemented("MCP add — interactive config entry not yet implemented");
                return EXIT_SUCCESS;
            };
            match mcp_runtime::parse_server_config(raw) {
                Ok(server) => match mcp_runtime::upsert_server(&paths, name, server) {
                    Ok(()) => {
                        let _ = mcp_runtime::ensure_project_mcp_cache(&paths);
                        println!("Saved MCP server '{name}' to {}", paths.mcp_config_path().display());
                        EXIT_SUCCESS
                    }
                    Err(error) => {
                        eprintln!("{error}");
                        EXIT_ERROR
                    }
                },
                Err(error) => {
                    eprintln!("{error}");
                    EXIT_ERROR
                }
            }
        }
        McpCommands::Remove { name } => match mcp_runtime::remove_server(&paths, name) {
            Ok(true) => {
                let auth_store_path = paths.auth_store_path();
                let rt = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .expect("tokio runtime");
                let _ = rt.block_on(clear_credentials(&auth_store_path, name));
                println!("Removed MCP server '{name}'.");
                EXIT_SUCCESS
            }
            Ok(false) => {
                eprintln!("MCP server '{name}' not found.");
                EXIT_ERROR
            }
            Err(error) => {
                eprintln!("{error}");
                EXIT_ERROR
            }
        },
        McpCommands::Doctor => handle_doctor(&paths),
        McpCommands::Auth { name, scopes } => handle_auth(&paths, name, scopes),
        McpCommands::Logout { name } => {
            let auth_store_path = paths.auth_store_path();
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("tokio runtime");
            match rt.block_on(clear_credentials(&auth_store_path, name)) {
                Ok(true) => {
                    println!("Cleared OAuth credentials for MCP server '{name}'.");
                    EXIT_SUCCESS
                }
                Ok(false) => {
                    eprintln!("No OAuth credentials found for MCP server '{name}'.");
                    EXIT_ERROR
                }
                Err(error) => {
                    eprintln!("{error}");
                    EXIT_ERROR
                }
            }
        }
    }
}

fn handle_auth(paths: &Paths, name: &str, scopes: &[String]) -> ExitCode {
    let config = match mcp_runtime::load_config(paths) {
        Ok(c) => c,
        Err(error) => {
            eprintln!("{error}");
            return EXIT_ERROR;
        }
    };
    let Some(server) = config.servers.get(name) else {
        eprintln!("MCP server '{name}' not found. Add it first with `elph mcp add`.");
        return EXIT_ERROR;
    };
    let Some(url) = server.remote_url() else {
        eprintln!("MCP server '{name}' is stdio; OAuth applies only to http/sse servers.");
        return EXIT_ERROR;
    };

    let scope_refs: Vec<&str> = if scopes.is_empty() {
        server.oauth_scopes().iter().map(String::as_str).collect()
    } else {
        scopes.iter().map(String::as_str).collect()
    };

    let auth_store_path = paths.auth_store_path();
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("tokio runtime");

    match rt.block_on(run_oauth_flow(name, url, &auth_store_path, &scope_refs)) {
        Ok(result) => {
            println!(
                "OAuth complete for '{name}' (client_id={}). Stored at {}.",
                result.client_id,
                result.credentials_path.display()
            );
            EXIT_SUCCESS
        }
        Err(error) => {
            eprintln!("OAuth failed: {error}");
            EXIT_ERROR
        }
    }
}

fn handle_doctor(paths: &Paths) -> ExitCode {
    let settings = match Settings::load(paths) {
        Ok(s) => s,
        Err(error) => {
            eprintln!("{error}");
            return EXIT_ERROR;
        }
    };
    let _ = settings;

    let config = match mcp_runtime::load_config(paths) {
        Ok(c) => c,
        Err(error) => {
            eprintln!("{error}");
            return EXIT_ERROR;
        }
    };

    if config.servers.is_empty() {
        println!("No MCP servers configured.");
        return EXIT_SUCCESS;
    }

    let auth_store_path = paths.auth_store_path();
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("tokio runtime");

    let mut ok = true;
    for (name, server) in &config.servers {
        if server.is_disabled() {
            println!("{name}: skipped [disabled]");
            continue;
        }
        let result = rt.block_on(probe_server_with_auth(name, server, Some(&auth_store_path)));
        let status = if result.ok { "ok" } else { "fail" };
        let oauth = if has_stored_credentials(&auth_store_path, name) {
            " oauth=yes"
        } else if server.wants_oauth() {
            " oauth=missing"
        } else {
            ""
        };
        println!("{name}: {status} [{}]{oauth} — {}", result.transport, result.message);
        if !result.ok {
            ok = false;
        }
    }

    if let Ok(cache) = mcp_runtime::project_mcp_cache_dir(paths) {
        println!("Project MCP cache: {}", cache.display());
    }
    println!("OAuth credentials: {}", auth_store_path.display());

    if ok { EXIT_SUCCESS } else { EXIT_ERROR }
}
