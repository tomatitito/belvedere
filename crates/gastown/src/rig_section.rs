use gpui::{Hsla, IntoElement, ParentElement, Styled, div, px};

use crate::dashboard_buffer::RigInfo;

pub struct RigSection {
    rigs: Vec<RigInfo>,
    palette: RigSectionPalette,
}

#[derive(Clone, Copy)]
pub struct RigSectionPalette {
    pub panel_bg: Hsla,
    pub border_variant: Hsla,
    pub text: Hsla,
    pub text_muted: Hsla,
    pub accent_info: Hsla,
}

impl RigSection {
    pub fn new(rigs: &[RigInfo], palette: RigSectionPalette) -> Self {
        Self {
            rigs: rigs.to_vec(),
            palette,
        }
    }
}

impl IntoElement for RigSection {
    type Element = gpui::Div;

    fn into_element(self) -> Self::Element {
        let palette = self.palette;

        let items: Vec<gpui::AnyElement> = if self.rigs.is_empty() {
            vec![
                div()
                    .text_color(palette.text_muted)
                    .text_sm()
                    .child("No rigs configured")
                    .into_any_element(),
            ]
        } else {
            self.rigs
                .iter()
                .map(|rig| RigRow::new(rig.clone(), palette).into_any_element())
                .collect()
        };

        div()
            .flex()
            .flex_col()
            .gap(px(8.0))
            .p(px(12.0))
            .rounded(px(6.0))
            .bg(palette.panel_bg)
            .border_1()
            .border_color(palette.border_variant)
            .child(div().text_color(palette.text).pb(px(4.0)).child("▸ Rigs"))
            .children(items)
    }
}

struct RigRow {
    rig: RigInfo,
    palette: RigSectionPalette,
}

impl RigRow {
    fn new(rig: RigInfo, palette: RigSectionPalette) -> Self {
        Self { rig, palette }
    }

    fn truncate_path(path: &str, max_len: usize) -> String {
        if path.len() <= max_len {
            return path.to_string();
        }

        let home = std::env::var("HOME").unwrap_or_default();
        let display_path = if !home.is_empty() && path.starts_with(&home) {
            format!("~{}", &path[home.len()..])
        } else {
            path.to_string()
        };

        if display_path.len() <= max_len {
            return display_path;
        }

        let parts: Vec<&str> = display_path.split('/').collect();
        if parts.len() <= 2 {
            return display_path;
        }

        let first = parts.first().unwrap_or(&"");
        let last = parts.last().unwrap_or(&"");
        format!("{}/…/{}", first, last)
    }
}

impl IntoElement for RigRow {
    type Element = gpui::Div;

    fn into_element(self) -> Self::Element {
        let palette = self.palette;
        let rig = self.rig;

        let truncated_path = Self::truncate_path(&rig.path, 40);

        div()
            .flex()
            .items_center()
            .gap(px(8.0))
            .py(px(4.0))
            .px(px(4.0))
            .rounded(px(4.0))
            .child(
                div()
                    .text_color(palette.accent_info)
                    .flex_shrink_0()
                    .child("⚙"),
            )
            .child(
                div()
                    .text_color(palette.text)
                    .flex_shrink_0()
                    .min_w(px(80.0))
                    .child(rig.name.clone()),
            )
            .child(
                div()
                    .text_color(palette.text_muted)
                    .text_sm()
                    .overflow_hidden()
                    .text_ellipsis()
                    .child(truncated_path),
            )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_palette() -> RigSectionPalette {
        use gpui::rgb;
        RigSectionPalette {
            panel_bg: rgb(0x2f343e).into(),
            border_variant: rgb(0x363c46).into(),
            text: rgb(0xdce0e5).into(),
            text_muted: rgb(0xa9afbc).into(),
            accent_info: rgb(0x74ade8).into(),
        }
    }

    #[test]
    fn test_rig_section_creation() {
        let palette = test_palette();
        let rigs = vec![
            RigInfo {
                name: "frontend".to_string(),
                path: "~/projects/webapp/frontend".to_string(),
            },
            RigInfo {
                name: "backend".to_string(),
                path: "~/projects/webapp/backend".to_string(),
            },
        ];

        let _section = RigSection::new(&rigs, palette);
    }

    #[test]
    fn test_empty_rigs() {
        let palette = test_palette();
        let rigs: Vec<RigInfo> = vec![];
        let _section = RigSection::new(&rigs, palette);
    }

    #[test]
    fn test_path_truncation() {
        assert_eq!(RigRow::truncate_path("/short", 40), "/short");
        assert_eq!(
            RigRow::truncate_path("/very/long/path/that/exceeds/the/maximum/length/limit", 20),
            "/…/limit"
        );
    }
}
