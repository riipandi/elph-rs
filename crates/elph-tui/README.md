# elph-tui

Terminal UI components for Elph agent applications. The interactive shell is built on [tuie](https://crates.io/crates/tuie)
with a pi-tui-inspired `diff/` engine for differential rendering, overlays, and rich components.

## Usage Sketch

```rust
use std::cell::RefCell;
use std::rc::Rc;
use elph_tui::{AgentShell, ShellHost, Theme, configure_runtime, start_shell};

struct MyHost;
impl ShellHost for MyHost { /* poll, chrome, transcript_lines, on_prompt_action, … */ }

let host: Rc<RefCell<dyn ShellHost>> = Rc::new(RefCell::new(MyHost));
configure_runtime();
start_shell(AgentShell::new(host))?;
```

## License

Licensed under the [MIT License](https://www.tldrlegal.com/license/mit-license).