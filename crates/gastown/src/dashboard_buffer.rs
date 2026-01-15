#![allow(dead_code)]

use gpui::{
    AnyElement, App, Context, EventEmitter, FocusHandle, Focusable, Hsla, IntoElement,
    ParentElement, Render, Styled, Window, div, px, rgb,
};
use std::sync::Arc;

use crate::agent_section::{AgentSection, AgentSectionPalette};
use crate::convoy_section::{ConvoySection, ConvoySectionPalette};
use crate::rig_section::{RigSection, RigSectionPalette};

/// Dashboard color palette matching Zed's One Dark theme.
/// Values from: assets/themes/one/one.json
struct DashboardPalette {
    panel_bg: Hsla,
    editor_bg: Hsla,
    border: Hsla,
    border_variant: Hsla,
    text: Hsla,
    text_muted: Hsla,
    accent_success: Hsla,
    accent_warning: Hsla,
    accent_error: Hsla,
    accent_info: Hsla,
    element_bg: Hsla,
    element_hover: Hsla,
}

impl DashboardPalette {
    fn one_dark() -> Self {
        Self {
            panel_bg: rgb(0x2f343e).into(),
            editor_bg: rgb(0x282c33).into(),
            border: rgb(0x464b57).into(),
            border_variant: rgb(0x363c46).into(),
            text: rgb(0xdce0e5).into(),
            text_muted: rgb(0xa9afbc).into(),
            accent_success: rgb(0xa1c181).into(),
            accent_warning: rgb(0xdec184).into(),
            accent_error: rgb(0xd07277).into(),
            accent_info: rgb(0x74ade8).into(),
            element_bg: rgb(0x2e343e).into(),
            element_hover: rgb(0x363c46).into(),
        }
    }

    fn to_agent_section_palette(&self) -> AgentSectionPalette {
        AgentSectionPalette {
            panel_bg: self.panel_bg,
            border_variant: self.border_variant,
            text: self.text,
            text_muted: self.text_muted,
            accent_success: self.accent_success,
            accent_warning: self.accent_warning,
            accent_error: self.accent_error,
            accent_info: self.accent_info,
            element_bg: self.element_bg,
        }
    }

    fn to_convoy_section_palette(&self) -> ConvoySectionPalette {
        ConvoySectionPalette {
            panel_bg: self.panel_bg,
            border_variant: self.border_variant,
            text: self.text,
            text_muted: self.text_muted,
            accent_success: self.accent_success,
            element_bg: self.element_bg,
        }
    }

    fn to_rig_section_palette(&self) -> RigSectionPalette {
        RigSectionPalette {
            panel_bg: self.panel_bg,
            border_variant: self.border_variant,
            text: self.text,
            text_muted: self.text_muted,
            accent_info: self.accent_info,
        }
    }
}

/// Events emitted by the dashboard when state changes
#[derive(Clone, Debug)]
pub enum DashboardEvent {
    /// Dashboard data was refreshed
    DataRefreshed,
    /// Connection status changed
    ConnectionChanged(ConnectionStatus),
    /// An agent was added
    AgentAdded(String),
    /// An agent was removed
    AgentRemoved(String),
    /// An agent's status changed
    AgentStatusChanged { name: String, status: AgentStatus },
}

/// Dashboard data returned by any data source
#[derive(Clone, Debug, Default)]
pub struct DashboardData {
    pub agents: Vec<AgentInfo>,
    pub convoys: Vec<ConvoyInfo>,
    pub rigs: Vec<RigInfo>,
}

#[derive(Clone, Debug)]
pub struct AgentInfo {
    pub name: String,
    pub status: AgentStatus,
    pub token_usage: Option<TokenUsage>,
    pub context_fill: Option<f32>,
}

#[derive(Clone, Debug, Default)]
pub struct TokenUsage {
    pub input_tokens: u64,
    pub output_tokens: u64,
}

#[derive(Clone, Debug, PartialEq)]
pub enum AgentStatus {
    Active,
    Idle,
    Error(String),
}

#[derive(Clone, Debug)]
pub struct ConvoyInfo {
    pub id: String,
    pub progress: f32,
}

#[derive(Clone, Debug)]
pub struct RigInfo {
    pub name: String,
    pub path: String,
}

/// Trait for fetching dashboard data - implementations can be direct, mock, or HTTP
pub trait DashboardDataSource: Send + Sync {
    fn fetch(&self) -> Result<DashboardData, DashboardError>;
    fn is_available(&self) -> bool;
}

#[derive(Debug, Clone)]
pub enum DashboardError {
    NotAvailable,
    FetchFailed(String),
    ParseError(String),
}

impl std::fmt::Display for DashboardError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DashboardError::NotAvailable => write!(f, "Dashboard not available"),
            DashboardError::FetchFailed(msg) => write!(f, "Fetch failed: {}", msg),
            DashboardError::ParseError(msg) => write!(f, "Parse error: {}", msg),
        }
    }
}

/// Formats dashboard data for display
pub struct DashboardFormatter;

impl DashboardFormatter {
    pub fn format(data: &DashboardData) -> String {
        let mut output = String::new();

        output.push_str("═══ Gastown Dashboard ═══\n\n");

        // Agents section
        output.push_str("▸ Agents\n");
        if data.agents.is_empty() {
            output.push_str("  No agents running\n");
        } else {
            for agent in &data.agents {
                let status_icon = match &agent.status {
                    AgentStatus::Active => "●",
                    AgentStatus::Idle => "○",
                    AgentStatus::Error(_) => "✗",
                };
                let mut line = format!("  {} {}", status_icon, agent.name);

                if let Some(fill) = agent.context_fill {
                    line.push_str(&format!(" [ctx: {:.0}%]", fill * 100.0));
                }
                if let Some(tokens) = &agent.token_usage {
                    line.push_str(&format!(
                        " [tokens: {}↓ {}↑]",
                        tokens.input_tokens, tokens.output_tokens
                    ));
                }
                output.push_str(&line);
                output.push('\n');
            }
        }
        output.push('\n');

        // Convoys section
        output.push_str("▸ Convoys\n");
        if data.convoys.is_empty() {
            output.push_str("  No active convoys\n");
        } else {
            for convoy in &data.convoys {
                let progress_bar = Self::progress_bar(convoy.progress);
                output.push_str(&format!(
                    "  {} {} ({:.0}%)\n",
                    convoy.id,
                    progress_bar,
                    convoy.progress * 100.0
                ));
            }
        }
        output.push('\n');

        // Rigs section
        output.push_str("▸ Rigs\n");
        if data.rigs.is_empty() {
            output.push_str("  No rigs configured\n");
        } else {
            for rig in &data.rigs {
                output.push_str(&format!("  {} → {}\n", rig.name, rig.path));
            }
        }

        output
    }

    pub fn format_error(error: &DashboardError) -> String {
        match error {
            DashboardError::NotAvailable => {
                "Dashboard unavailable\n\nRun 'gt up' to start Gastown".into()
            }
            DashboardError::FetchFailed(msg) => {
                format!("Failed to load dashboard\n\n{}", msg)
            }
            DashboardError::ParseError(msg) => {
                format!("Failed to parse dashboard data\n\n{}", msg)
            }
        }
    }

    fn progress_bar(progress: f32) -> String {
        let filled = (progress * 10.0) as usize;
        let empty = 10 - filled;
        format!("[{}{}]", "█".repeat(filled), "░".repeat(empty))
    }
}

/// View for displaying dashboard data.
/// This is a pure GPUI view - not a workspace Item.
pub struct DashboardView {
    focus_handle: FocusHandle,
    data: Option<DashboardData>,
    error: Option<DashboardError>,
    data_source: Arc<dyn DashboardDataSource>,
    last_update: Option<std::time::Instant>,
    connection_status: ConnectionStatus,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ConnectionStatus {
    Connected,
    Disconnected,
    Unknown,
}

impl DashboardView {
    pub fn new(data_source: Arc<dyn DashboardDataSource>, cx: &mut App) -> Self {
        let mut view = Self {
            focus_handle: cx.focus_handle(),
            data: None,
            error: None,
            data_source,
            last_update: None,
            connection_status: ConnectionStatus::Unknown,
        };
        view.refresh_sync();
        view
    }

    pub fn content(&self) -> String {
        match (&self.data, &self.error) {
            (Some(data), _) => DashboardFormatter::format(data),
            (_, Some(err)) => DashboardFormatter::format_error(err),
            _ => "Loading...".into(),
        }
    }

    pub fn is_read_only(&self) -> bool {
        true
    }

    pub fn connection_status(&self) -> &ConnectionStatus {
        &self.connection_status
    }

    pub fn last_update(&self) -> Option<std::time::Instant> {
        self.last_update
    }

    pub fn refresh(&mut self, cx: &mut Context<Self>) {
        self.refresh_sync();
        cx.emit(DashboardEvent::DataRefreshed);
        cx.notify();
    }

    fn refresh_sync(&mut self) {
        self.connection_status = if self.data_source.is_available() {
            ConnectionStatus::Connected
        } else {
            ConnectionStatus::Disconnected
        };

        match self.data_source.fetch() {
            Ok(data) => {
                self.data = Some(data);
                self.error = None;
                self.last_update = Some(std::time::Instant::now());
            }
            Err(err) => {
                self.data = None;
                self.error = Some(err);
            }
        }
    }
}

impl Focusable for DashboardView {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl EventEmitter<DashboardEvent> for DashboardView {}

impl Render for DashboardView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let palette = DashboardPalette::one_dark();

        let content: AnyElement = if let Some(ref data) = self.data {
            self.render_data(data, &palette).into_any_element()
        } else if let Some(ref err) = self.error {
            self.render_error(err, &palette).into_any_element()
        } else {
            self.render_loading(&palette).into_any_element()
        };

        div()
            .flex()
            .flex_col()
            .size_full()
            .bg(palette.editor_bg)
            .text_color(palette.text)
            .child(content)
    }
}

impl DashboardView {
    fn render_data(&self, data: &DashboardData, palette: &DashboardPalette) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .size_full()
            .p(px(16.0))
            .gap(px(16.0))
            .child(self.render_header(palette))
            .child(self.render_agents_section(&data.agents, palette))
            .child(self.render_convoys_section(&data.convoys, palette))
            .child(self.render_rigs_section(&data.rigs, palette))
    }

    fn render_error(&self, error: &DashboardError, palette: &DashboardPalette) -> impl IntoElement {
        let message = match error {
            DashboardError::NotAvailable => "Dashboard unavailable\n\nRun 'gt up' to start Gastown",
            DashboardError::FetchFailed(msg) => msg.as_str(),
            DashboardError::ParseError(msg) => msg.as_str(),
        };

        div()
            .flex()
            .flex_col()
            .size_full()
            .items_center()
            .justify_center()
            .gap(px(8.0))
            .child(
                div()
                    .text_color(palette.accent_error)
                    .text_xl()
                    .child("⚠ Error"),
            )
            .child(
                div()
                    .text_color(palette.text_muted)
                    .child(message.to_string()),
            )
    }

    fn render_loading(&self, palette: &DashboardPalette) -> impl IntoElement {
        div()
            .flex()
            .size_full()
            .items_center()
            .justify_center()
            .text_color(palette.text_muted)
            .child("Loading...")
    }

    fn render_header(&self, palette: &DashboardPalette) -> impl IntoElement {
        div()
            .flex()
            .items_center()
            .justify_between()
            .pb(px(8.0))
            .border_b_1()
            .border_color(palette.border_variant)
            .child(
                div()
                    .text_xl()
                    .text_color(palette.text)
                    .child("Gastown Dashboard"),
            )
            .child(self.render_connection_status(palette))
    }

    fn render_connection_status(&self, palette: &DashboardPalette) -> impl IntoElement {
        let (color, label) = match self.connection_status {
            ConnectionStatus::Connected => (palette.accent_success, "● Connected"),
            ConnectionStatus::Disconnected => (palette.accent_error, "○ Disconnected"),
            ConnectionStatus::Unknown => (palette.text_muted, "? Unknown"),
        };

        div().text_sm().text_color(color).child(label)
    }

    fn render_agents_section(
        &self,
        agents: &[AgentInfo],
        palette: &DashboardPalette,
    ) -> impl IntoElement {
        AgentSection::new(agents, palette.to_agent_section_palette())
    }

    fn render_convoys_section(
        &self,
        convoys: &[ConvoyInfo],
        palette: &DashboardPalette,
    ) -> impl IntoElement {
        ConvoySection::new(convoys, palette.to_convoy_section_palette())
    }

    fn render_rigs_section(
        &self,
        rigs: &[RigInfo],
        palette: &DashboardPalette,
    ) -> impl IntoElement {
        RigSection::new(rigs, palette.to_rig_section_palette())
    }

    fn render_section(
        &self,
        title: &str,
        items: Vec<gpui::AnyElement>,
        palette: &DashboardPalette,
    ) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .gap(px(8.0))
            .p(px(12.0))
            .rounded(px(6.0))
            .bg(palette.panel_bg)
            .border_1()
            .border_color(palette.border_variant)
            .child(
                div()
                    .text_color(palette.text)
                    .pb(px(4.0))
                    .child(format!("▸ {}", title)),
            )
            .children(items)
    }
}
