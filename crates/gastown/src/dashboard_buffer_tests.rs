use gpui::{AppContext, TestAppContext};
use std::sync::Arc;
use workspace::item::Item;

use crate::dashboard_buffer::{
    AgentInfo, AgentStatus, ConnectionStatus, ConvoyInfo, DashboardBufferView, DashboardData,
    DashboardDataSource, DashboardError, DashboardFormatter, RigInfo,
};

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
        assert!(
            buffer.is_read_only(),
            "Dashboard buffer should be read-only"
        );
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
