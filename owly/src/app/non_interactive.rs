use anyhow::Result;

use crate::agent::{self, RunAgentResult};
use crate::runtime::config::Config;
use crate::runtime::session::SessionStore;
use crate::ui::{
    dry_run::{
        plan_dry_run, print_chat_completion, print_doc_exists_redirect, print_dry_run, print_skipped_completion,
        print_update_skipped,
    },
    print_chat_header, print_command_header, print_completion,
};
use crate::wiki::docs::{self, DocumentationSnapshot};
use crate::wiki::mode::WikiContext;

use super::Command;
use super::doc_run::{apply_doc_run_result, run_init_agent, run_update_agent, should_skip_update_noop};

/// Print a plan for init/update/chat without calling the LLM or writing wiki pages.
pub(super) fn run_dry_run(config: &Config, ctx: &WikiContext, command: &Command) -> Result<()> {
    let plan = plan_dry_run(ctx, command)?;
    print_dry_run(config, &plan);
    Ok(())
}

pub(super) async fn run_non_interactive(
    config: &Config,
    ctx: &WikiContext,
    command: Command,
    print_mode: bool,
    stream: bool,
    verbose: bool,
) -> Result<()> {
    match command {
        Command::Init => run_non_interactive_init(config, ctx, print_mode, stream, verbose).await,
        Command::Update => run_non_interactive_update(config, ctx, print_mode, stream, verbose).await,
        Command::Chat { message: Some(msg) } => {
            let session_anchor = ctx.agent_cwd();
            let mut session = SessionStore::open(&session_anchor).await?;
            run_non_interactive_chat(config, ctx, &msg, print_mode, stream, verbose, &mut session).await
        }
        Command::Chat { message: None } => {
            anyhow::bail!("Pass a message, --init, or --update.");
        }
    }
}

async fn run_non_interactive_init(
    config: &Config,
    ctx: &WikiContext,
    print_mode: bool,
    stream: bool,
    verbose: bool,
) -> Result<()> {
    if docs::create_snapshot(ctx)?.exists {
        print_doc_exists_redirect("init");
        return do_non_interactive_update(config, ctx, print_mode, stream, verbose).await;
    }
    do_non_interactive_init(config, ctx, print_mode, stream, verbose).await
}

async fn run_non_interactive_update(
    config: &Config,
    ctx: &WikiContext,
    print_mode: bool,
    stream: bool,
    verbose: bool,
) -> Result<()> {
    if !docs::create_snapshot(ctx)?.exists {
        print_doc_exists_redirect("update");
        return do_non_interactive_init(config, ctx, print_mode, stream, verbose).await;
    }
    do_non_interactive_update(config, ctx, print_mode, stream, verbose).await
}

async fn do_non_interactive_init(
    config: &Config,
    ctx: &WikiContext,
    print_mode: bool,
    stream: bool,
    verbose: bool,
) -> Result<()> {
    print_command_header("Init", &config.provider, &config.model_id);

    let (result, snapshot) = run_init_agent(config, ctx, None, print_mode, stream, verbose).await?;

    finish_non_interactive_doc_run(ctx, config, "init", &result, &snapshot, print_mode)
}

async fn do_non_interactive_update(
    config: &Config,
    ctx: &WikiContext,
    print_mode: bool,
    stream: bool,
    verbose: bool,
) -> Result<()> {
    if should_skip_update_noop(ctx, None, stream, verbose, print_mode) {
        print_update_skipped();
        return Ok(());
    }

    print_command_header("Update", &config.provider, &config.model_id);

    let (result, snapshot) = run_update_agent(config, ctx, None, print_mode, stream, verbose).await?;

    finish_non_interactive_doc_run(ctx, config, "update", &result, &snapshot, print_mode)
}

async fn run_non_interactive_chat(
    config: &Config,
    ctx: &WikiContext,
    message: &str,
    print_mode: bool,
    stream: bool,
    verbose: bool,
    session: &mut SessionStore,
) -> Result<()> {
    print_chat_header(&config.provider, &config.model_id);

    let (system_prompt, user_prompt) = agent::prepare_chat_command(ctx, message);
    let user_prompt = format!("{user_prompt}{}", crate::wiki::prompts::create_runtime_note(ctx));

    let result = agent::run_agent(agent::RunAgentOptions {
        command: "chat",
        system_prompt: &system_prompt,
        user_prompt: &user_prompt,
        config,
        ctx,
        print_mode,
        stream,
        verbose,
        session: Some(session),
        is_followup: false,
        docs_snapshot_before: None,
    })
    .await?;

    if !result.completion_message.is_empty() {
        print_chat_completion(&result.completion_message);
    }

    Ok(())
}

fn finish_non_interactive_doc_run(
    ctx: &WikiContext,
    config: &Config,
    command: &str,
    result: &RunAgentResult,
    before: &DocumentationSnapshot,
    print_mode: bool,
) -> Result<()> {
    if result.skipped {
        if !print_mode {
            print_skipped_completion(&result.completion_message);
        }
        return Ok(());
    }

    apply_doc_run_result(ctx, config, command, result, before)?;

    if print_mode {
        if !result.completion_message.is_empty() {
            print_skipped_completion(&result.completion_message);
        }
    } else {
        print_completion(&result.completion_message);
    }

    Ok(())
}
