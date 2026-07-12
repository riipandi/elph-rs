//! Dry-run plan rendering (no LLM, no wiki writes).

use crate::app::Command;
use crate::runtime::config::Config;
use crate::wiki::docs;
use crate::wiki::metadata;
use crate::wiki::mode::{RunMode, WikiContext};

/// Planned action for init/update/chat without invoking the agent.
#[derive(Debug)]
pub enum DryRunAction {
    InitWouldCreate,
    InitWouldDelegateToUpdate,
    UpdateWouldInitFirst,
    UpdateWouldSkipNoop,
    UpdateWouldRefresh,
    Chat { message_preview: String },
}

/// Collect what would happen for a dry-run invocation.
pub fn plan_dry_run(ctx: &WikiContext, command: &Command) -> anyhow::Result<DryRunPlan> {
    let wiki_root = ctx.wiki_root();
    let action = match command {
        Command::Init => {
            if wiki_root.exists() && docs::create_snapshot(ctx)?.exists {
                DryRunAction::InitWouldDelegateToUpdate
            } else {
                DryRunAction::InitWouldCreate
            }
        }
        Command::Update => {
            if !wiki_root.exists() || !docs::create_snapshot(ctx)?.exists {
                DryRunAction::UpdateWouldInitFirst
            } else if metadata::is_update_noop_ctx(ctx) {
                DryRunAction::UpdateWouldSkipNoop
            } else {
                DryRunAction::UpdateWouldRefresh
            }
        }
        Command::Chat { message } => DryRunAction::Chat {
            message_preview: message.as_deref().unwrap_or("(interactive)").to_string(),
        },
    };
    Ok(DryRunPlan {
        mode: ctx.mode,
        wiki_root,
        action,
    })
}

#[derive(Debug)]
pub struct DryRunPlan {
    pub mode: RunMode,
    pub wiki_root: std::path::PathBuf,
    pub action: DryRunAction,
}

pub fn print_dry_run(config: &Config, plan: &DryRunPlan) {
    println!("Owly dry-run (no LLM, no wiki writes)");
    println!("  mode:     {}", plan.mode.as_str());
    println!("  provider: {}", config.provider);
    println!("  model:    {}", config.model_id);
    println!("  wiki:     {}", plan.wiki_root.display());
    match &plan.action {
        DryRunAction::InitWouldCreate => {
            println!("  action:   init → would create wiki via elph-agent");
            if plan.mode == RunMode::Code {
                println!(
                    "  after:    refresh AGENTS.md/CLAUDE.md (OWLY:START/END) + optional .github/workflows/owly-update.yml"
                );
            }
        }
        DryRunAction::InitWouldDelegateToUpdate => {
            println!("  action:   init → would delegate to update (wiki already exists)");
            if plan.mode == RunMode::Code {
                println!(
                    "  after:    refresh AGENTS.md/CLAUDE.md (OWLY:START/END) + optional .github/workflows/owly-update.yml"
                );
            }
        }
        DryRunAction::UpdateWouldInitFirst => {
            println!("  action:   update → would init first (wiki missing)");
            if plan.mode == RunMode::Code {
                println!("  after:    update .last-update.json only if wiki content changes; sync agent guidance");
            } else {
                println!("  after:    update ~/.owly/wiki/.last-update.json only if wiki content changes");
            }
        }
        DryRunAction::UpdateWouldSkipNoop => {
            println!("  action:   update → would skip (no-op: no relevant changes since last update)");
            if plan.mode == RunMode::Code {
                println!("  after:    update .last-update.json only if wiki content changes; sync agent guidance");
            } else {
                println!("  after:    update ~/.owly/wiki/.last-update.json only if wiki content changes");
            }
        }
        DryRunAction::UpdateWouldRefresh => {
            println!("  action:   update → would run surgical doc refresh via elph-agent");
            if plan.mode == RunMode::Code {
                println!("  after:    update .last-update.json only if wiki content changes; sync agent guidance");
            } else {
                println!("  after:    update ~/.owly/wiki/.last-update.json only if wiki content changes");
            }
        }
        DryRunAction::Chat { message_preview } => {
            println!("  action:   chat → would send message via elph-agent");
            println!("  message:  {message_preview}");
        }
    }
}

pub fn print_doc_exists_redirect(action: &str) {
    match action {
        "init" => {
            println!("Documentation already exists. Updating...");
            println!();
        }
        "update" => {
            println!("No documentation found. Initializing...");
            println!();
        }
        _ => {}
    }
}

pub fn print_update_skipped() {
    println!("No changes detected. Skipping.");
}

pub fn print_skipped_completion(message: &str) {
    println!("{message}");
}

pub fn print_chat_completion(message: &str) {
    print!("{message}");
    if !message.ends_with('\n') {
        println!();
    }
}
