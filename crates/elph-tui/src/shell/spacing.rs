use slt::Context;

/// Gap between major shell regions (status / chat / input / footer).
pub fn shell_section_gap(ui: &Context) -> u32 {
    ui.spacing().xs()
}

/// Gap inside the input stack (activity → palette → prompt).
pub fn shell_input_gap(ui: &Context, composer: bool) -> u32 {
    if composer { 0 } else { ui.spacing().xs() }
}

/// Padding inside bordered panels (user cards, slash palette).
pub fn shell_panel_pad(ui: &Context) -> u32 {
    ui.spacing().xs()
}

/// Tight padding for the prompt chrome.
pub fn shell_prompt_pad(ui: &Context) -> u32 {
    let _ = ui;
    0
}
