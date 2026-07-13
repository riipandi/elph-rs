//! Minimal tuie-based TUI for Elph.
//!
//! ans: basic implementation based on tuie-demo
//! (https://github.com/riipandi/tuie-demo) and tuie reference
//! (https://github.com/jake-stewart/tuie).
//! Will be expanded as the TUI is rebuilt.

use tuie::prelude::*;

/// A simple demo app to verify the tuie integration works.
struct ElphTui {
    root: Box<Pane>,
    input_id: WidgetId<Input>,
    output_id: WidgetId<Text>,
    #[allow(dead_code)]
    resume_id: Option<String>,
}

impl ElphTui {
    fn new(resume_id: Option<String>) -> Box<Self> {
        let mut input_id = WidgetId::EMPTY;
        let mut output_id = WidgetId::EMPTY;

        let header = Pane::new()
            .x_align(FlexAlign::Center)
            .padding(Spacing::balanced(1))
            .child(
                Text::new()
                    .content("Elph · coding agent companion")
                    .style(Style::new().bold()),
            );

        let output = Pane::new()
            .border(Border::SINGLE)
            .flex(1)
            .child(Text::new().content("Ready.").id(&mut output_id));

        let input = Pane::new().border(Border::SINGLE).child(
            Input::new()
                .placeholder(Text::new().content("type a message…".dim()))
                .id(&mut input_id),
        );

        let root =
            Pane::new()
                .vertical()
                .padding(Spacing::balanced(1))
                .children([header as Box<dyn Widget>, output, input]);

        Box::new(Self {
            root,
            input_id,
            output_id,
            resume_id,
        })
    }
}

impl DelegateWidget for ElphTui {
    tuie::delegate_widget!(root);

    fn after_on_event(&mut self, event: &mut WidgetEvent) {
        if event.of::<Input>() && event.source == self.input_id {
            if let Some(input) = self.root.get_widget(self.input_id) {
                let text = input.get_string().to_string();
                if !text.trim().is_empty() {
                    if let Some(output) = self.root.get_widget_mut(self.output_id) {
                        let current = output.get_content();
                        let ready: StyledString = "Ready.".into();
                        let new_content = if current == ready {
                            StyledString::from(format!("❯ {text}"))
                        } else {
                            StyledString::from(format!("{current}\n❯ {text}"))
                        };
                        output.set_content(new_content);
                    }
                    if let Some(input) = self.root.get_widget_mut(self.input_id) {
                        input.set_content("");
                    }
                }
            }
        }
    }
}

/// Launch the Elph TUI.
pub fn run_tui(resume_id: Option<String>) -> std::io::Result<()> {
    let app = ElphTui::new(resume_id);
    let _code = tuie::start_tui(app)?;
    Ok(())
}
