use gpui::{
    App, AppContext, Context, EventEmitter, FocusHandle, Focusable, IntoElement, ParentElement,
    Render, SharedString, Styled, TestAppContext, Window, div,
};
use std::sync::Arc;
use workspace::item::{Item, ItemEvent};

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
#[allow(dead_code)]
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

/// Mock data source for testing
pub struct MockDataSource {
    data: Option<DashboardData>,
    available: bool,
}

impl MockDataSource {
    pub fn available_with(data: DashboardData) -> Self {
        Self {
            data: Some(data),
            available: true,
        }
    }

    pub fn unavailable() -> Self {
        Self {
            data: None,
            available: false,
        }
    }
}

impl DashboardDataSource for MockDataSource {
    fn fetch(&self) -> Result<DashboardData, DashboardError> {
        if !self.available {
            return Err(DashboardError::NotAvailable);
        }
        self.data
            .clone()
            .ok_or_else(|| DashboardError::FetchFailed("No data configured".into()))
    }

    fn is_available(&self) -> bool {
        self.available
    }
}

/// Formats dashboard data for display in the buffer
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
                output.push_str(&format!("  {} {}\n", status_icon, agent.name));
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
                output.push_str(&format!("  {} {} ({:.0}%)\n", convoy.id, progress_bar, convoy.progress * 100.0));
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

/// Buffer view for displaying dashboard data
pub struct DashboardBufferView {
    focus_handle: FocusHandle,
    content: SharedString,
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

impl DashboardBufferView {
    pub fn new(data_source: Arc<dyn DashboardDataSource>, cx: &mut App) -> Self {
        let mut view = Self {
            focus_handle: cx.focus_handle(),
            content: SharedString::from("Loading..."),
            data_source,
            last_update: None,
            connection_status: ConnectionStatus::Unknown,
        };
        view.refresh_sync();
        view
    }

    pub fn content(&self) -> &str {
        &self.content
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

    pub fn refresh(&mut self, _cx: &mut Context<Self>) {
        self.refresh_sync();
    }

    fn refresh_sync(&mut self) {
        self.connection_status = if self.data_source.is_available() {
            ConnectionStatus::Connected
        } else {
            ConnectionStatus::Disconnected
        };

        match self.data_source.fetch() {
            Ok(data) => {
                self.content = DashboardFormatter::format(&data).into();
                self.last_update = Some(std::time::Instant::now());
            }
            Err(err) => {
                self.content = DashboardFormatter::format_error(&err).into();
            }
        }
    }
}

impl Focusable for DashboardBufferView {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl EventEmitter<ItemEvent> for DashboardBufferView {}

impl Render for DashboardBufferView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .size_full()
            .child(self.content.clone())
    }
}

impl Item for DashboardBufferView {
    type Event = ItemEvent;

    fn tab_content_text(&self, _detail: usize, _cx: &App) -> SharedString {
        "Dashboard".into()
    }
}

// =============================================================================
// Tests
// =============================================================================

fn sample_dashboard_data() -> DashboardData {
    DashboardData {
        agents: vec![
            AgentInfo {
                name: "agent-1".into(),
                status: AgentStatus::Active,
            },
            AgentInfo {
                name: "agent-2".into(),
                status: AgentStatus::Idle,
            },
        ],
        convoys: vec![ConvoyInfo {
            id: "convoy-1".into(),
            progress: 0.5,
        }],
        rigs: vec![RigInfo {
            name: "main".into(),
            path: "/project".into(),
        }],
    }
}

#[gpui::test]
async fn test_dashboard_buffer_displays_content(cx: &mut TestAppContext) {
    let data_source = Arc::new(MockDataSource::available_with(sample_dashboard_data()));
    let buffer = cx.new(|cx| DashboardBufferView::new(data_source, cx));

    buffer.update(cx, |buffer: &mut DashboardBufferView, _cx| {
        let content = buffer.content();
        assert!(content.contains("Gastown Dashboard"));
        assert!(content.contains("agent-1"));
        assert!(content.contains("agent-2"));
        assert!(content.contains("convoy-1"));
    });
}

#[gpui::test]
async fn test_dashboard_buffer_is_read_only(cx: &mut TestAppContext) {
    let data_source = Arc::new(MockDataSource::available_with(DashboardData::default()));
    let buffer = cx.new(|cx| DashboardBufferView::new(data_source, cx));

    buffer.update(cx, |buffer: &mut DashboardBufferView, _cx| {
        assert!(buffer.is_read_only(), "Dashboard buffer should be read-only");
    });
}

#[gpui::test]
async fn test_dashboard_buffer_refresh_updates_timestamp(cx: &mut TestAppContext) {
    let data_source = Arc::new(MockDataSource::available_with(DashboardData::default()));
    let buffer = cx.new(|cx| DashboardBufferView::new(data_source, cx));

    buffer.update(cx, |buffer: &mut DashboardBufferView, cx| {
        let first_update = buffer.last_update();
        assert!(first_update.is_some());

        std::thread::sleep(std::time::Duration::from_millis(10));
        buffer.refresh(cx);

        let second_update = buffer.last_update();
        assert!(second_update.is_some());
        assert!(second_update.unwrap() > first_update.unwrap());
    });
}

#[gpui::test]
async fn test_dashboard_shows_error_when_unavailable(cx: &mut TestAppContext) {
    let data_source = Arc::new(MockDataSource::unavailable());
    let buffer = cx.new(|cx| DashboardBufferView::new(data_source, cx));

    buffer.update(cx, |buffer: &mut DashboardBufferView, _cx| {
        assert_eq!(buffer.connection_status(), &ConnectionStatus::Disconnected);
        assert!(buffer.content().contains("unavailable"));
        assert!(buffer.content().contains("gt up"));
    });
}

#[gpui::test]
async fn test_dashboard_shows_connected_status(cx: &mut TestAppContext) {
    let data_source = Arc::new(MockDataSource::available_with(DashboardData::default()));
    let buffer = cx.new(|cx| DashboardBufferView::new(data_source, cx));

    buffer.update(cx, |buffer: &mut DashboardBufferView, _cx| {
        assert_eq!(buffer.connection_status(), &ConnectionStatus::Connected);
    });
}

#[gpui::test]
async fn test_dashboard_formatter_handles_empty_data(_cx: &mut TestAppContext) {
    let data = DashboardData::default();
    let formatted = DashboardFormatter::format(&data);

    assert!(formatted.contains("Gastown Dashboard"));
    assert!(formatted.contains("No agents running"));
    assert!(formatted.contains("No active convoys"));
    assert!(formatted.contains("No rigs configured"));
}

#[gpui::test]
async fn test_dashboard_formatter_shows_agent_status(_cx: &mut TestAppContext) {
    let data = DashboardData {
        agents: vec![
            AgentInfo {
                name: "active-agent".into(),
                status: AgentStatus::Active,
            },
            AgentInfo {
                name: "idle-agent".into(),
                status: AgentStatus::Idle,
            },
            AgentInfo {
                name: "error-agent".into(),
                status: AgentStatus::Error("connection lost".into()),
            },
        ],
        ..Default::default()
    };

    let formatted = DashboardFormatter::format(&data);

    assert!(formatted.contains("● active-agent"));
    assert!(formatted.contains("○ idle-agent"));
    assert!(formatted.contains("✗ error-agent"));
}

#[gpui::test]
async fn test_dashboard_formatter_shows_convoy_progress(_cx: &mut TestAppContext) {
    let data = DashboardData {
        convoys: vec![
            ConvoyInfo {
                id: "convoy-half".into(),
                progress: 0.5,
            },
            ConvoyInfo {
                id: "convoy-done".into(),
                progress: 1.0,
            },
        ],
        ..Default::default()
    };

    let formatted = DashboardFormatter::format(&data);

    assert!(formatted.contains("convoy-half"));
    assert!(formatted.contains("50%"));
    assert!(formatted.contains("convoy-done"));
    assert!(formatted.contains("100%"));
}

#[gpui::test]
async fn test_dashboard_formatter_shows_rigs(_cx: &mut TestAppContext) {
    let data = DashboardData {
        rigs: vec![
            RigInfo {
                name: "frontend".into(),
                path: "/app/frontend".into(),
            },
            RigInfo {
                name: "backend".into(),
                path: "/app/backend".into(),
            },
        ],
        ..Default::default()
    };

    let formatted = DashboardFormatter::format(&data);

    assert!(formatted.contains("frontend → /app/frontend"));
    assert!(formatted.contains("backend → /app/backend"));
}

#[gpui::test]
async fn test_dashboard_tab_shows_correct_title(cx: &mut TestAppContext) {
    let data_source = Arc::new(MockDataSource::available_with(DashboardData::default()));
    let buffer = cx.new(|cx| DashboardBufferView::new(data_source, cx));

    cx.read(|cx| {
        let title = buffer.read(cx).tab_content_text(0, cx);
        assert_eq!(title.as_ref(), "Dashboard");
    });
}

#[gpui::test]
async fn test_data_source_trait_with_mock(_cx: &mut TestAppContext) {
    // Test the trait works correctly with mock implementation
    let mock = MockDataSource::available_with(sample_dashboard_data());

    assert!(mock.is_available());

    let data = mock.fetch().expect("should fetch successfully");
    assert_eq!(data.agents.len(), 2);
    assert_eq!(data.convoys.len(), 1);
    assert_eq!(data.rigs.len(), 1);
}

#[gpui::test]
async fn test_data_source_unavailable_returns_error(_cx: &mut TestAppContext) {
    let mock = MockDataSource::unavailable();

    assert!(!mock.is_available());

    let result = mock.fetch();
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), DashboardError::NotAvailable));
}
