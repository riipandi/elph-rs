---
name: tui-design
description: >-
    Guide terminal UI development with iocraft: component mental model, flex layout, scroll regions,
    overlap decoration, state/hooks, input handling, and common layout pitfalls.
    Use when building or refactoring a TUI, fixing scroll/focus/layout bugs, choosing iocraft patterns,
    or the user mentions iocraft, terminal UI, TUI components, or runs /tui-design.
---

# TUI Design (iocraft)

## Objective

Help implement and refine terminal UIs using **iocraft** with a clear mental model and repeatable patterns — not tied to any single product layout or design doc.

Read existing TUI code in the repo first; apply the principles below to match local conventions.

## Mental model

### 1. Terminal as a single viewport

The UI is one tree of elements sized to the terminal (`use_terminal_size`). Think **web flexbox in a fixed viewport**: columns stack top-to-bottom; one region usually **grows** to fill leftover height; fixed chrome (header, input, footer) does not grow.

### 2. Component = function + props + element tree

Each visual unit is a `#[component]` function taking `&Props` and `Hooks`. Props are plain structs (`#[derive(Default, Props)]`). Parent passes data down; interactive state lives in hooks, not in props.

Decompose by **responsibility**, not by screen coordinates:

- Shell (root layout, global events, exit)
- Scrollable content panel
- Fixed chrome (header, status, input area)
- Small leaf widgets (labels, inputs, buttons)

Prefer several small components over one monolithic `element!` block.

### 3. State vs presentation

| Concern                                          | Where it lives                              |
| ------------------------------------------------ | ------------------------------------------- |
| Layout dimensions                                | `use_terminal_size`, derived widths/heights |
| User-driven values (mode, selection, draft text) | `use_state` / focused widget state          |
| Side effects (timers, async)                     | `use_future`                                |
| Global app control (quit)                        | `use_context_mut::<SystemContext>`          |
| Pure rendering                                   | Props + read-only hook state                |

Initialize state with a closure: `use_state(|| initial)` — not `use_state(initial)` for non-Copy values.

### 4. Events are edge-triggered

Handle `use_terminal_events` on **press**, not release:

```rust
if kind == KeyEventKind::Release {
    return;
}
```

Match on `(modifiers, code)`. Decide whether shortcuts are global (shell) or local (focused `TextInput`) before wiring keys.

### 5. Layering: flow vs overlap

**Flow layout** — children affect parent size and sibling position (default).

**Overlap** — child with `position: Position::Absolute` is taken out of flow. Use for badges, border labels, overlays. Parent needs `position: Position::Relative`. Set `background_color: Color::Reset` on the overlay so it paints over the border cleanly. Negative margin (`margin_bottom: -1`) can sit a label on the border line without adding row height.

## Layout best practices

### Full-height column shell

```rust
View(
    width: screen_width,
    height: screen_height,
    flex_direction: FlexDirection::Column,
    justify_content: JustifyContent::FlexStart,
    align_items: AlignItems::Center,
) { /* fixed chrome + growing panel */ }
```

Use `FlexStart` on the **root**. `FlexEnd` on the root combined with inner `auto_scroll` can push fixed chrome off-screen.

### Growing scroll region (flex + clip)

The classic pattern for a chat/log area between fixed bars:

```rust
View(
    flex_grow: 1f32,
    flex_shrink: 1f32,
    min_height: 0,
    overflow: Overflow::Hidden,
) {
    View(width: 100pct, height: 100pct, overflow: Overflow::Hidden) {
        ScrollView(
            scrollbar: true,
            auto_scroll: true,
            keyboard_scroll: true,
        ) { /* content */ }
    }
}
```

`min_height: 0` is required for flex children to shrink below content size (same idea as CSS `min-height: 0`).

**Bottom-anchored content:** inner column may use `justify_content: JustifyContent::End` so new items sit at the bottom; pair with `auto_scroll: true` to follow the latest line.

### Common layout pitfalls

| Symptom                     | Likely cause                                                       | Fix                                                                 |
| --------------------------- | ------------------------------------------------------------------ | ------------------------------------------------------------------- |
| Scrollbar missing           | `flex_grow` + `height: 100pct` on wrong node, or no bounded height | Use grow + `min_height: 0` + overflow hidden wrapper                |
| Header/footer scroll away   | Root `justify_content: FlexEnd` with auto-scroll                   | Root stays `FlexStart`; only inner content uses `FlexEnd` if needed |
| Extra blank row under input | Overlap label in normal flow                                       | `position: Absolute` on label container                             |
| Props compile errors        | Missing `Default` or wrong prop syntax                             | `#[derive(Default, Props)]`, named props in `element!`              |
| State won't init            | Passed value instead of initializer                                | `use_state(                                                         |     | value)` |

### Width discipline

Pass `screen_width` (or computed `width - padding`) into children that wrap text or draw borders so lines do not reflow unexpectedly. Split footers/status rows with `width / 2` or `justify_content: SpaceBetween`.

## iocraft component patterns

### Props

```rust
#[derive(Default, Props)]
struct PanelProps {
    width: u16,
    label: String,
}

#[derive(Clone, Copy, Default, Props)]
struct BadgeProps {
    width: u16,
    mode: Mode,
}
```

Use `Clone` on props when passing owned strings into child components. Use `Copy` for small enums passed by value.

### Hooks checklist

- `use_terminal_size()` — responsive layout
- `use_state(|| …)` — UI state
- `use_terminal_events(closure)` — keyboard/mouse
- `use_context_mut::<SystemContext>()` — `system.exit()`
- `use_future(async { … })` — intervals, background work (keep loops cancel-friendly)

### Entry point

```rust
element!(App(/* props */))
    .render_loop()
    .fullscreen()
    .enable_mouse_capture()  // when click/scroll matters
    .ignore_ctrl_c()         // handle Ctrl+C in app logic if needed
    .await?;
```

Use `.print()` for static snapshots in examples without a runtime loop.

## Input & focus

- One primary `TextInput` usually has `has_focus: true` unless building a multi-pane UI.
- `multiline: true` for prompt-style editors; document newline vs submit keys in status/help text.
- Global shortcuts (quit, cycle mode) belong in the shell's `use_terminal_events`; avoid duplicating handlers in every child.
- Tab may be consumed by the focused input — choose bindings that do not fight focus when simulating mode switches in demos.

## Visual design (general)

- Prefer a small set of semantic roles: primary text, secondary/dim, border, accent, error/success.
- Borders: structural chrome often neutral; accents on **text or small badges**, not every border, unless the UX calls for it.
- Scrollbars: enable explicitly; thumb/track colors should contrast with background.
- Padding: keep horizontal padding consistent between header, content, and footer so columns align visually.

Do not invent product-specific copy, palettes, or zone layouts unless the user or existing code defines them.

## Workflow

When implementing or fixing a TUI:

1. **Locate** existing shell/components in the repo; follow naming and file placement already in use.
2. **Sketch** the zone split (fixed vs growing) before writing `element!` trees.
3. **Extract** a new `#[component]` when a block exceeds ~40 lines or is reused.
4. **Wire state** at the lowest ancestor that needs it; pass derived values as props.
5. **Test layout** at small and large terminal sizes mentally: does the grow region still clip?
6. **Verify** compile the affected crate/example:
    ```bash
    cargo check -p <crate>
    cargo check -p <crate> --example <name>   # if applicable
    ```
7. **Change only what the task needs** — no drive-by refactors across unrelated widgets.

## Decision guide

| Need                                  | Pattern                                           |
| ------------------------------------- | ------------------------------------------------- |
| List/chat that fills remaining height | Flex grow + `ScrollView`                          |
| Pin to latest message                 | `auto_scroll: true` + bottom-aligned inner column |
| Label on border corner                | Absolute child + `background_color: Reset`        |
| Periodic UI refresh                   | `use_future` + `use_state` for clock/tick         |
| Quit app                              | `should_exit` state + `system.exit()`             |
| Modal / overlay (future)              | Absolute full-size `View` above siblings          |

## Anti-patterns

- Monolithic shell with all markup and handlers in one component
- Hard-coded terminal dimensions instead of `use_terminal_size`
- Scrollable root wrapping fixed header (fixed zones should be **siblings outside** the scroll view)
- Coloring every border by transient state when only a badge needs emphasis
- Ignoring `KeyEventKind::Release` and firing actions twice
- Copy-pasting layout trees instead of shared child components
