//! Connector schedule list and mutation feedback.

use crate::setup::onboarding_config::{OnboardingConfig, OnboardingSourceScheduleConfig};

pub fn print_schedule_list(config: &OnboardingConfig) {
    println!("Owly connector schedules (~/.owly/onboarding.json)\n");
    if let Some(schedule) = &config.ingestion_schedule {
        print_schedule("all", schedule);
    } else {
        println!("  (no global ingestion schedule)");
    }
    for (id, source) in &config.sources {
        if let Some(schedule) = &source.schedule {
            print_schedule(id, schedule);
        }
    }
    if config.ingestion_schedule.is_none() && config.sources.values().all(|s| s.schedule.is_none()) {
        println!("\nNo schedules configured. Add ingestion_schedule to onboarding.json.");
    }
}

fn print_schedule(id: &str, schedule: &OnboardingSourceScheduleConfig) {
    let paused = schedule.paused_at.as_deref().unwrap_or("");
    let status = if paused.is_empty() { "active" } else { "paused" };
    println!("  {:<16} {}  ({})", id, schedule.expression, status);
    if !schedule.description.is_empty() {
        println!("    {}", schedule.description);
    }
}

pub fn print_schedule_deleted(target: &str) {
    println!("Deleted schedule for {target}.");
}

pub fn print_schedule_mutated(pause: bool, target: &str) {
    println!("{} schedule for {target}.", if pause { "Paused" } else { "Resumed" });
}
