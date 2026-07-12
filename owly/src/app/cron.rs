//! Connector schedule management (`onboarding.json`).

use anyhow::{Result, bail};
use chrono::Utc;

use crate::connectors::{ConnectorId, is_connector_id};
use crate::setup::onboarding_config::{self, OnboardingSourceScheduleConfig, save_onboarding_config};

pub type ScheduleTarget = String;

pub fn cron_list() -> Result<()> {
    let config = onboarding_config::read_onboarding_config()?;
    crate::ui::schedules::print_schedule_list(&config);
    Ok(())
}

pub fn cron_pause(target: &str) -> Result<()> {
    mutate_schedule(target, true)
}

pub fn cron_resume(target: &str) -> Result<()> {
    mutate_schedule(target, false)
}

pub fn cron_delete(target: &str) -> Result<()> {
    let mut config = onboarding_config::read_onboarding_config()?;
    if target == "all" {
        config.ingestion_schedule = None;
        for source in config.sources.values_mut() {
            source.schedule = None;
        }
    } else if is_connector_id(target) {
        if let Some(source) = config.sources.get_mut(target) {
            source.schedule = None;
        }
    } else {
        bail!("Unknown schedule target: {target}");
    }
    save_onboarding_config(&config)?;
    crate::ui::schedules::print_schedule_deleted(target);
    Ok(())
}

fn mutate_schedule(target: &str, pause: bool) -> Result<()> {
    let mut config = onboarding_config::read_onboarding_config()?;
    let now = Utc::now().to_rfc3339();
    let targets: Vec<String> = if target == "all" {
        config.sources.keys().cloned().collect::<Vec<_>>()
    } else if is_connector_id(target) {
        vec![target.to_string()]
    } else {
        bail!("Unknown schedule target: {target}");
    };

    if targets.is_empty() && config.ingestion_schedule.is_none() {
        bail!("No schedules configured.");
    }

    if let Some(schedule) = config.ingestion_schedule.as_mut()
        && (target == "all" || targets.len() == config.sources.len())
    {
        if pause {
            schedule.paused_at = Some(now.clone());
        } else {
            schedule.paused_at = None;
        }
    }

    for id in targets {
        let entry = config.sources.entry(id).or_default();
        let schedule = entry.schedule.get_or_insert(default_schedule());
        if pause {
            schedule.paused_at = Some(now.clone());
        } else {
            schedule.paused_at = None;
        }
    }
    save_onboarding_config(&config)?;
    crate::ui::schedules::print_schedule_mutated(pause, target);
    Ok(())
}

fn default_schedule() -> OnboardingSourceScheduleConfig {
    OnboardingSourceScheduleConfig {
        description: "Daily ingestion".into(),
        expression: "0 2 * * *".into(),
        launch_agent_path: None,
        paused_at: None,
        updated_at: Utc::now().to_rfc3339(),
        warning: None,
    }
}

pub fn ensure_default_schedule_for_connector(id: ConnectorId) -> Result<()> {
    let mut config = onboarding_config::read_onboarding_config()?;
    let key = id.as_str();
    let entry = config.sources.entry(key.to_string()).or_default();
    if entry.schedule.is_none() {
        entry.schedule = Some(default_schedule());
        save_onboarding_config(&config)?;
    }
    Ok(())
}
