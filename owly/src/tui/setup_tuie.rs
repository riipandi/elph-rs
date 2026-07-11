//! First-run credential setup as a tuie widget.

use std::sync::{Arc, Mutex};

use elph_tui::Theme;
use tuie::prelude::{Text as TuiText, *};

use crate::onboarding::{
    SetupCredentials, api_key_label, base_url_label, default_model_for_provider, provider_select_items,
    setup_base_url_required, setup_collects_base_url,
};

use super::app::OwlyApp;

#[derive(Clone, Copy, PartialEq, Eq)]
enum SetupStep {
    Provider,
    ApiKey,
    BaseUrl,
    Model,
}

struct ProviderCtx {
    labels: Vec<String>,
    selected: usize,
}

/// Minimal tuie setup wizard; exits the TUI when credentials are applied.
pub struct SetupTuie {
    root: Box<Pane>,
    app: Arc<Mutex<OwlyApp>>,
    step: SetupStep,
    providers: Vec<String>,
    provider_labels: Vec<String>,
    selected: usize,
    api_key: String,
    base_url: String,
    model_id: String,
    input_id: WidgetId<Input>,
    title_id: WidgetId<TuiText>,
    hint_id: WidgetId<TuiText>,
}

impl SetupTuie {
    pub fn new(app: Arc<Mutex<OwlyApp>>, theme: Theme) -> Box<Self> {
        let (provider, model) = {
            let guard = app.lock().expect("owly app lock");
            (guard.provider.clone(), guard.model.clone())
        };

        let items: Vec<(String, String)> = provider_select_items();
        let providers: Vec<String> = items.iter().map(|(v, _)| v.clone()).collect();
        let provider_labels: Vec<String> = items.into_iter().map(|(_, label)| label).collect();
        let selected = providers.iter().position(|p| p == &provider).unwrap_or(0);

        let mut list_id = WidgetId::EMPTY;
        let mut input_id = WidgetId::EMPTY;
        let mut title_id = WidgetId::EMPTY;
        let mut hint_id = WidgetId::EMPTY;

        let mut list = List::new().vertical().scroll(Scrollbar::AutoHide);
        list.set_renderer(
            ProviderCtx {
                labels: provider_labels.clone(),
                selected,
            },
            |ctx: &mut ProviderCtx, idx: usize| -> Option<Box<dyn Widget>> {
                let label = ctx.labels.get(idx)?;
                let marker = if idx == ctx.selected { "› " } else { "  " };
                Some(TuiText::new().content(format!("{marker}{label}")) as Box<dyn Widget>)
            },
        );
        list.set_item_count(provider_labels.len());
        let list = list.id(&mut list_id);

        let input = Input::new()
            .placeholder(
                TuiText::new()
                    .content("…")
                    .style(Style::new().fg(theme.input_placeholder())),
            )
            .id(&mut input_id);

        let root = Pane::new()
            .vertical()
            .flex(1)
            .gap(1)
            .padding(Spacing::balanced(2))
            .x_place(Place::Center)
            .y_place(Place::Center)
            .max_width(72)
            .children([
                TuiText::new().content(">_ Owly setup".bold()).id(&mut title_id) as Box<dyn Widget>,
                TuiText::new()
                    .content("Configure your inference provider and API key.".dim())
                    .id(&mut hint_id),
                list,
                input as Box<dyn Widget>,
            ]);

        Box::new(Self {
            root,
            app,
            step: SetupStep::Provider,
            providers,
            provider_labels,
            selected,
            api_key: String::new(),
            base_url: String::new(),
            model_id: model,
            input_id,
            title_id,
            hint_id,
        })
    }

    fn current_provider(&self) -> &str {
        self.providers.get(self.selected).map(String::as_str).unwrap_or("")
    }

    fn sync_labels(&mut self, _theme: Theme) {
        let (title, hint) = match self.step {
            SetupStep::Provider => (">_ Owly setup".to_string(), "Select provider (↑/↓, Enter)".to_string()),
            SetupStep::ApiKey => (
                api_key_label(self.current_provider()).unwrap_or_else(|| "API key".into()),
                "Enter to continue · Esc back".into(),
            ),
            SetupStep::BaseUrl => (
                base_url_label(self.current_provider()).unwrap_or_else(|| "Base URL".into()),
                "Enter to continue · Esc back".into(),
            ),
            SetupStep::Model => ("Model ID".into(), "Enter to finish · Esc back".into()),
        };
        if let Some(text) = self.root.get_widget_mut::<TuiText>(self.title_id) {
            text.set_content(title.bold());
        }
        if let Some(text) = self.root.get_widget_mut(self.hint_id) {
            text.set_content(hint.dim());
        }
    }

    fn advance(&mut self, theme: Theme) {
        match self.step {
            SetupStep::Provider => {
                if let Some(provider) = self.providers.get(self.selected)
                    && let Some(default_model) = default_model_for_provider(provider)
                {
                    self.model_id = default_model.to_string();
                }
                self.step = SetupStep::ApiKey;
            }
            SetupStep::ApiKey => {
                if let Some(input) = self.root.get_widget(self.input_id) {
                    self.api_key = input.get_string();
                }
                self.step = if setup_collects_base_url(self.current_provider()) {
                    SetupStep::BaseUrl
                } else {
                    SetupStep::Model
                };
            }
            SetupStep::BaseUrl => {
                if let Some(input) = self.root.get_widget(self.input_id) {
                    self.base_url = input.get_string();
                }
                self.step = SetupStep::Model;
            }
            SetupStep::Model => {
                if let Some(input) = self.root.get_widget(self.input_id) {
                    let draft = input.get_string();
                    if !draft.is_empty() {
                        self.model_id = draft;
                    }
                }
                let credentials = SetupCredentials {
                    provider: self.current_provider().to_string(),
                    api_key: self.api_key.clone(),
                    base_url: if setup_base_url_required(self.current_provider()) {
                        Some(self.base_url.clone())
                    } else {
                        None
                    },
                    model_id: self.model_id.clone(),
                };
                let mut guard = self.app.lock().expect("owly app lock");
                guard.complete_setup(credentials);
                if guard.setup_complete {
                    tuie::quit(0);
                }
                return;
            }
        }
        if let Some(input) = self.root.get_widget_mut(self.input_id) {
            input.set_content("");
        }
        self.sync_labels(theme);
    }
}

impl DelegateWidget for SetupTuie {
    tuie::delegate_widget!(root);

    fn override_on_input(&mut self, queue: &mut InputQueue) -> InputResult {
        let theme = self.app.lock().expect("owly app lock").theme;
        let Some(event) = queue.peek() else {
            return InputResult::Rejected;
        };

        if self.step == SetupStep::Provider {
            if event.chord == chord!(Up) {
                queue.next();
                self.selected = self.selected.saturating_sub(1);
                return InputResult::Handled;
            }
            if event.chord == chord!(Down) {
                queue.next();
                if self.selected + 1 < self.provider_labels.len() {
                    self.selected += 1;
                }
                return InputResult::Handled;
            }
        }

        if event.chord == chord!(Esc) {
            queue.next();
            self.step = match self.step {
                SetupStep::Provider => SetupStep::Provider,
                SetupStep::ApiKey => SetupStep::Provider,
                SetupStep::BaseUrl => SetupStep::ApiKey,
                SetupStep::Model => {
                    if setup_collects_base_url(self.current_provider()) {
                        SetupStep::BaseUrl
                    } else {
                        SetupStep::ApiKey
                    }
                }
            };
            self.sync_labels(theme);
            return InputResult::Handled;
        }

        if event.chord == chord!(Enter) {
            queue.next();
            self.advance(theme);
            return InputResult::Handled;
        }

        if self.step != SetupStep::Provider {
            return self.root.on_input(queue);
        }
        InputResult::Handled
    }
}
