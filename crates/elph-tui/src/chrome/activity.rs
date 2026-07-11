use std::time::Instant;

/// Activity line shown between chat and input while the agent is busy.
#[derive(Debug, Clone, Default)]
pub struct ActivityState {
    pub label: String,
    pub started: Option<Instant>,
    pub visible: bool,
    pub cancel_requested: bool,
}

impl ActivityState {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            started: Some(Instant::now()),
            visible: true,
            cancel_requested: false,
        }
    }

    pub fn working() -> Self {
        Self::new("Working")
    }

    pub fn responding() -> Self {
        Self::new("Responding")
    }

    pub fn awaiting_input() -> Self {
        Self::new("Waiting for your answer")
    }

    pub fn running_tool(name: &str, args: &str) -> Self {
        let detail = if args.is_empty() {
            name.to_string()
        } else {
            format!("{name} {args}")
        };
        Self::new(detail)
    }

    pub fn request_cancel(&mut self) {
        self.cancel_requested = true;
    }

    pub fn clear(&mut self) {
        self.visible = false;
        self.cancel_requested = false;
        self.started = None;
        self.label.clear();
    }
}
