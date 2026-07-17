//! Bootstrap helpers for fullscreen interactive examples.

use anyhow::Result;
use iocraft::prelude::*;

pub async fn run_fullscreen<C>(component: C) -> Result<()>
where
    C: Into<AnyElement<'static>>,
{
    component
        .into()
        .render_loop()
        .fullscreen()
        .enable_mouse_capture()
        .await?;
    Ok(())
}
