/// Startup tips shown in the full banner (randomized via [`pick_tip`]).
pub const BANNER_TIPS: &[&str] = &[
    "Use --no-session for ephemeral mode — no session file is saved, useful for one-off queries.",
    "Press ? in the prompt for keyboard shortcuts.",
    "Use /help to list slash commands.",
    "Shift+↑/↓ scrolls the transcript; Shift+End jumps to the latest message.",
    "Ctrl+K opens the command palette.",
];

/// Session metadata rendered in the banner.
#[derive(Debug, Clone, Copy)]
pub struct BannerInfo<'a> {
    pub app_name: &'a str,
    pub version: &'a str,
    pub update_available: bool,
    pub directory: &'a str,
    pub model: Option<&'a str>,
    pub provider: Option<&'a str>,
    pub extensions: u32,
    pub commands: u32,
    pub skills: u32,
    pub tools: u32,
    pub mcp_connected: u32,
    pub mcp_total: u32,
    pub mcp_tools: u32,
    pub tip: &'a str,
}

impl<'a> BannerInfo<'a> {
    pub fn header_line(self) -> String {
        let mut line = format!("Welcome to {} v{}", self.app_name, self.version);
        if self.update_available {
            line.push_str(" (update available)");
        }
        line
    }

    pub fn subtitle(self) -> &'static str {
        "Send /changelog to show version history."
    }
}

/// Tracks whether the banner is full or compact.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct BannerState {
    pub compact: bool,
}

impl BannerState {
    pub fn on_user_message(&mut self) {
        self.compact = true;
    }
}

/// Picks a tip deterministically from the session id hash.
pub fn pick_tip(session_seed: &str) -> &'static str {
    let mut hash = 0u64;
    for byte in session_seed.bytes() {
        hash = hash.wrapping_mul(31).wrapping_add(u64::from(byte));
    }
    let idx = (hash as usize) % BANNER_TIPS.len();
    BANNER_TIPS[idx]
}

/// Plain-text lines for printing the simple banner into terminal scrollback.
pub fn simple_banner_lines(info: BannerInfo<'_>) -> Vec<String> {
    let model = info.model.unwrap_or("—");
    let provider = info.provider.unwrap_or("—");
    vec![
        format!("{} v{}", info.app_name, info.version),
        format!("{model} · {provider} · {}", info.directory),
        String::new(),
    ]
}
