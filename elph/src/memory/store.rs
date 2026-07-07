use std::sync::Arc;

use anyhow::{Context, Result};
use elph_core::memz::{
    DEFAULT_EMBEDDING_DIMS, EmbedFn, FastEmbedOptions, MemoryStore, MemzConfig, create_fastembed, create_memory_store,
    embedding_dims, resolve_embedding_model,
};

use elph_core::utils::path::AppPaths;

use crate::runtime::{Paths, Settings};

/// No-op embedder for read-only CLI commands (status, list, purge).
fn noop_embedder() -> EmbedFn {
    Arc::new(|_| Box::pin(async { Ok(vec![0.0f32; DEFAULT_EMBEDDING_DIMS as usize]) }))
}

fn local_embedder(paths: &Paths, settings: &Settings) -> Result<EmbedFn> {
    std::fs::create_dir_all(paths.models_dir()).with_context(|| format!("create {}", paths.models_dir().display()))?;

    let options = FastEmbedOptions {
        model: Some(settings.memory.embed_model.clone()),
        quantized: settings.memory.embed_quantized,
        cache_dir: Some(paths.models_dir()),
        show_download_progress: Some(true),
    };

    create_fastembed(options).context("failed to initialize local embedder")
}

fn memz_config(paths: &Paths, settings: &Settings) -> Result<MemzConfig> {
    let dims = resolve_embedding_model(&settings.memory.embed_model, settings.memory.embed_quantized)
        .map(|m| embedding_dims(&m))
        .unwrap_or(DEFAULT_EMBEDDING_DIMS);

    Ok(MemzConfig::new(paths.memory_db_path().to_string_lossy().into_owned(), "elph-cli").dimensions(dims))
}

pub fn open_store(paths: &Paths, needs_embed: bool) -> Result<MemoryStore> {
    std::fs::create_dir_all(paths.project_elph_dir())
        .with_context(|| format!("create {}", paths.project_elph_dir().display()))?;

    let settings = Settings::load(paths).context("load settings")?;

    let embed = if needs_embed {
        local_embedder(paths, &settings)?
    } else {
        noop_embedder()
    };

    Ok(create_memory_store(memz_config(paths, &settings)?, embed))
}
