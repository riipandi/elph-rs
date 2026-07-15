//! Scrollbar styling helpers and position indicator.

use iocraft::prelude::*;

/// Colors for an iocraft [`ScrollView`] scrollbar.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ScrollbarStyle {
    pub thumb_color: Option<Color>,
    pub track_color: Option<Color>,
}

impl ScrollbarStyle {
    pub fn dark() -> Self {
        Self {
            thumb_color: Some(Color::Rgb { r: 88, g: 88, b: 88 }),
            track_color: Some(Color::Rgb { r: 48, g: 48, b: 48 }),
        }
    }
}

/// Props for [`ScrollIndicator`].
#[derive(Clone, Copy, Default, Props)]
pub struct ScrollIndicatorProps {
    pub offset: u32,
    pub total: u32,
    pub visible: u32,
    pub width: u16,
}

/// Read-only scroll position indicator (e.g. `12/40`).
#[component]
pub fn ScrollIndicator(props: &ScrollIndicatorProps) -> impl Into<AnyElement<'static>> {
    let top = props.offset.saturating_add(1);
    let bottom = (props.offset + props.visible).min(props.total);
    let label = format!("{top}-{bottom}/{total}", total = props.total.max(1));

    element! {
        View(width: props.width, align_items: AlignItems::End) {
            Text(content: label, color: Color::DarkGrey, wrap: TextWrap::NoWrap)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dark_style_has_colors() {
        let style = ScrollbarStyle::dark();
        assert!(style.thumb_color.is_some());
    }
}
