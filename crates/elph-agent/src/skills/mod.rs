//! Skill discovery and formatting.

mod args;
mod format;
mod load;

pub use args::argument_hint_requires_args;
pub use args::format_skill_missing_args_notice;
pub use args::metadata_requires_arguments;
pub use args::skill_args_validation_notice;
pub use args::skill_requires_arguments;
pub use format::format_skill_invocation;
pub use load::LoadSkillsResult;
pub use load::LoadSourcedSkillsResult;
pub use load::SkillDiagnostic;
pub use load::SkillDiagnosticCode;
pub use load::SourcedSkill;
pub use load::SourcedSkillDiagnostic;
pub use load::{load_skills, load_skills_with_options, load_sourced_skills, load_sourced_skills_with_options};
