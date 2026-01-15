use gpui::{
    ClickEvent, Hsla, InteractiveElement, IntoElement, ParentElement, StatefulInteractiveElement,
    Styled, div, px,
};
use std::sync::Arc;

use crate::dashboard_buffer::{AgentInfo, AgentStatus};

type ToggleHandler = Arc<dyn Fn(&ClickEvent, &mut gpui::Window, &mut gpui::App) + 'static>;

pub struct AgentSection {
    agents: Vec<AgentInfo>,
    palette: AgentSectionPalette,
    expanded: bool,
    on_toggle: Option<ToggleHandler>,
}

#[derive(Clone, Copy)]
pub struct AgentSectionPalette {
    pub panel_bg: Hsla,
    pub border_variant: Hsla,
    pub text: Hsla,
    pub text_muted: Hsla,
    pub accent_success: Hsla,
    pub accent_warning: Hsla,
    pub accent_error: Hsla,
    pub accent_info: Hsla,
    pub element_bg: Hsla,
}

impl AgentSection {
    pub fn new(agents: &[AgentInfo], palette: AgentSectionPalette) -> Self {
        Self {
            agents: agents.to_vec(),
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

impl IntoElement for AgentSection {
    type Element = gpui::Div;

    fn into_element(self) -> Self::Element {
        let palette = self.palette;
        let disclosure = if self.expanded { "▾" } else { "▸" };

        let items: Vec<gpui::AnyElement> = if self.agents.is_empty() {
            vec![
                div()
                    .text_color(palette.text_muted)
                    .text_sm()
                    .child("No agents running")
                    .into_any_element(),
            ]
        } else {
            self.agents
                .iter()
                .map(|agent| AgentRow::new(agent.clone(), palette).into_any_element())
                .collect()
        };

        let header = div()
            .id("agents-header")
            .flex()
            .items_center()
            .gap(px(4.0))
            .text_color(palette.text)
            .pb(px(4.0))
            .cursor_pointer()
            .child(disclosure)
            .child("Agents");

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

struct AgentRow {
    agent: AgentInfo,
    palette: AgentSectionPalette,
}

impl AgentRow {
    fn new(agent: AgentInfo, palette: AgentSectionPalette) -> Self {
        Self { agent, palette }
    }
}

impl IntoElement for AgentRow {
    type Element = gpui::Div;

    fn into_element(self) -> Self::Element {
        let palette = self.palette;
        let agent = self.agent;

        let (status_icon, status_color) = match &agent.status {
            AgentStatus::Active => ("●", palette.accent_success),
            AgentStatus::Idle => ("○", palette.text_muted),
            AgentStatus::Error(_) => ("✗", palette.accent_error),
        };

        let mut row = div()
            .flex()
            .items_center()
            .gap(px(8.0))
            .py(px(4.0))
            .px(px(4.0))
            .rounded(px(4.0))
            .child(div().text_color(status_color).child(status_icon))
            .child(
                div()
                    .text_color(palette.text)
                    .flex_shrink_0()
                    .child(agent.name.clone()),
            );

        if let Some(fill) = agent.context_fill {
            row = row.child(ContextBar::new(fill, palette));
        }

        if let Some(ref tokens) = agent.token_usage {
            row = row.child(
                div()
                    .text_color(palette.text_muted)
                    .text_sm()
                    .ml_auto()
                    .child(format!(
                        "{}↓ {}↑",
                        tokens.input_tokens, tokens.output_tokens
                    )),
            );
        }

        row
    }
}

struct ContextBar {
    fill: f32,
    palette: AgentSectionPalette,
}

impl ContextBar {
    fn new(fill: f32, palette: AgentSectionPalette) -> Self {
        Self { fill, palette }
    }
}

impl IntoElement for ContextBar {
    type Element = gpui::Div;

    fn into_element(self) -> Self::Element {
        let palette = self.palette;
        let fill = self.fill;
        let fill_percent = (fill * 100.0).round() as u32;

        let bar_color = if fill > 0.8 {
            palette.accent_warning
        } else {
            palette.accent_info
        };

        div()
            .flex()
            .items_center()
            .gap(px(4.0))
            .child(
                div()
                    .w(px(60.0))
                    .h(px(6.0))
                    .rounded(px(3.0))
                    .bg(palette.element_bg)
                    .child(
                        div()
                            .h_full()
                            .w(px(60.0 * fill))
                            .rounded(px(3.0))
                            .bg(bar_color),
                    ),
            )
            .child(
                div()
                    .text_color(palette.text_muted)
                    .text_sm()
                    .child(format!("{}%", fill_percent)),
            )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dashboard_buffer::TokenUsage;

    fn test_palette() -> AgentSectionPalette {
        use gpui::rgb;
        AgentSectionPalette {
            panel_bg: rgb(0x2f343e).into(),
            border_variant: rgb(0x363c46).into(),
            text: rgb(0xdce0e5).into(),
            text_muted: rgb(0xa9afbc).into(),
            accent_success: rgb(0xa1c181).into(),
            accent_warning: rgb(0xdec184).into(),
            accent_error: rgb(0xd07277).into(),
            accent_info: rgb(0x74ade8).into(),
            element_bg: rgb(0x2e343e).into(),
        }
    }

    #[test]
    fn test_agent_section_creation() {
        let palette = test_palette();
        let agents = vec![
            AgentInfo {
                name: "BlueLake".to_string(),
                status: AgentStatus::Active,
                token_usage: Some(TokenUsage {
                    input_tokens: 45230,
                    output_tokens: 12450,
                }),
                context_fill: Some(0.73),
            },
            AgentInfo {
                name: "GreenForest".to_string(),
                status: AgentStatus::Idle,
                token_usage: None,
                context_fill: None,
            },
        ];

        let _section = AgentSection::new(&agents, palette);
    }

    #[test]
    fn test_empty_agents() {
        let palette = test_palette();
        let agents: Vec<AgentInfo> = vec![];
        let _section = AgentSection::new(&agents, palette);
    }
}
