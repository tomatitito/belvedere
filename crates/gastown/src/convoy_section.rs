use gpui::{
    ClickEvent, Hsla, InteractiveElement, IntoElement, ParentElement, StatefulInteractiveElement,
    Styled, div, px,
};
use std::sync::Arc;

use crate::dashboard_buffer::ConvoyInfo;

type ToggleHandler = Arc<dyn Fn(&ClickEvent, &mut gpui::Window, &mut gpui::App) + 'static>;

pub struct ConvoySection {
    convoys: Vec<ConvoyInfo>,
    palette: ConvoySectionPalette,
    expanded: bool,
    on_toggle: Option<ToggleHandler>,
}

#[derive(Clone, Copy)]
pub struct ConvoySectionPalette {
    pub panel_bg: Hsla,
    pub border_variant: Hsla,
    pub text: Hsla,
    pub text_muted: Hsla,
    pub accent_success: Hsla,
    pub element_bg: Hsla,
}

impl ConvoySection {
    pub fn new(convoys: &[ConvoyInfo], palette: ConvoySectionPalette) -> Self {
        Self {
            convoys: convoys.to_vec(),
            palette,
            expanded: true,
            on_toggle: None,
        }
    }

    pub fn expanded(mut self, expanded: bool) -> Self {
        self.expanded = expanded;
        self
    }

    pub fn on_toggle(
        mut self,
        on_toggle: impl Fn(&ClickEvent, &mut gpui::Window, &mut gpui::App) + 'static,
    ) -> Self {
        self.on_toggle = Some(Arc::new(on_toggle));
        self
    }
}

impl IntoElement for ConvoySection {
    type Element = gpui::Div;

    fn into_element(self) -> Self::Element {
        let palette = self.palette;
        let disclosure = if self.expanded { "▾" } else { "▸" };

        let items: Vec<gpui::AnyElement> = if self.convoys.is_empty() {
            vec![
                div()
                    .text_color(palette.text_muted)
                    .text_sm()
                    .child("No active convoys")
                    .into_any_element(),
            ]
        } else {
            self.convoys
                .iter()
                .map(|convoy| ConvoyRow::new(convoy.clone(), palette).into_any_element())
                .collect()
        };

        let header = div()
            .id("convoys-header")
            .flex()
            .items_center()
            .gap(px(4.0))
            .text_color(palette.text)
            .pb(px(4.0))
            .cursor_pointer()
            .child(disclosure)
            .child("Convoys");

        let header = if let Some(on_toggle) = self.on_toggle {
            header.on_click(move |event, window, cx| on_toggle(event, window, cx))
        } else {
            header
        };

        let section = div()
            .flex()
            .flex_col()
            .gap(px(8.0))
            .p(px(12.0))
            .rounded(px(6.0))
            .bg(palette.panel_bg)
            .border_1()
            .border_color(palette.border_variant)
            .child(header);

        if self.expanded {
            section.children(items)
        } else {
            section
        }
    }
}

struct ConvoyRow {
    convoy: ConvoyInfo,
    palette: ConvoySectionPalette,
}

impl ConvoyRow {
    fn new(convoy: ConvoyInfo, palette: ConvoySectionPalette) -> Self {
        Self { convoy, palette }
    }
}

impl IntoElement for ConvoyRow {
    type Element = gpui::Div;

    fn into_element(self) -> Self::Element {
        let palette = self.palette;
        let convoy = self.convoy;
        let progress_percent = (convoy.progress * 100.0).round() as u32;

        let bar_width = 200.0;
        let fill_width = bar_width * convoy.progress;

        div()
            .flex()
            .items_center()
            .gap(px(12.0))
            .py(px(4.0))
            .child(
                div()
                    .text_color(palette.text)
                    .w(px(120.0))
                    .child(convoy.id.clone()),
            )
            .child(
                div()
                    .w(px(bar_width))
                    .h(px(8.0))
                    .rounded(px(4.0))
                    .bg(palette.element_bg)
                    .child(
                        div()
                            .h_full()
                            .w(px(fill_width))
                            .rounded(px(4.0))
                            .bg(palette.accent_success),
                    ),
            )
            .child(
                div()
                    .text_color(palette.text_muted)
                    .text_sm()
                    .w(px(40.0))
                    .child(format!("{}%", progress_percent)),
            )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_palette() -> ConvoySectionPalette {
        use gpui::rgb;
        ConvoySectionPalette {
            panel_bg: rgb(0x2f343e).into(),
            border_variant: rgb(0x363c46).into(),
            text: rgb(0xdce0e5).into(),
            text_muted: rgb(0xa9afbc).into(),
            accent_success: rgb(0xa1c181).into(),
            element_bg: rgb(0x2e343e).into(),
        }
    }

    #[test]
    fn test_convoy_section_creation() {
        let palette = test_palette();
        let convoys = vec![
            ConvoyInfo {
                id: "refactor-auth".to_string(),
                progress: 0.65,
            },
            ConvoyInfo {
                id: "migrate-db".to_string(),
                progress: 0.30,
            },
        ];

        let _section = ConvoySection::new(&convoys, palette);
    }

    #[test]
    fn test_empty_convoys() {
        let palette = test_palette();
        let convoys: Vec<ConvoyInfo> = vec![];
        let _section = ConvoySection::new(&convoys, palette);
    }

    #[test]
    fn test_convoy_progress_display() {
        let palette = test_palette();
        let convoys = vec![
            ConvoyInfo {
                id: "full-progress".to_string(),
                progress: 1.0,
            },
            ConvoyInfo {
                id: "zero-progress".to_string(),
                progress: 0.0,
            },
            ConvoyInfo {
                id: "half-progress".to_string(),
                progress: 0.5,
            },
        ];

        let _section = ConvoySection::new(&convoys, palette);
    }
}
