use anyhow::Result;
use gpui::{App, Application, Bounds, WindowBounds, WindowOptions, actions, prelude::*, px, size};
use std::sync::Arc;

mod agent_section;
mod convoy_section;
mod dashboard_buffer;
mod rig_section;

use dashboard_buffer::{
    AgentInfo, AgentStatus, ConvoyInfo, DashboardData, DashboardDataSource, DashboardError,
    DashboardView, RigInfo, TokenUsage,
};

#[cfg(test)]
mod dashboard_buffer_tests;

actions!(gastown, [Quit]);

fn quit(_: &Quit, cx: &mut App) {
    cx.quit();
}

/// Sample data source that provides mock dashboard data for demonstration
struct SampleDataSource;

impl SampleDataSource {
    fn sample_data() -> DashboardData {
        DashboardData {
            agents: vec![
                AgentInfo {
                    name: "BlueLake".into(),
                    status: AgentStatus::Active,
                    token_usage: Some(TokenUsage {
                        input_tokens: 45_230,
                        output_tokens: 12_450,
                    }),
                    context_fill: Some(0.73),
                },
                AgentInfo {
                    name: "GreenCastle".into(),
                    status: AgentStatus::Idle,
                    token_usage: Some(TokenUsage {
                        input_tokens: 8_120,
                        output_tokens: 2_340,
                    }),
                    context_fill: Some(0.15),
                },
                AgentInfo {
                    name: "RedMountain".into(),
                    status: AgentStatus::Error("Connection timeout".into()),
                    token_usage: None,
                    context_fill: None,
                },
            ],
            convoys: vec![
                ConvoyInfo {
                    id: "refactor-auth".into(),
                    progress: 0.65,
                },
                ConvoyInfo {
                    id: "migrate-db".into(),
                    progress: 0.30,
                },
                ConvoyInfo {
                    id: "add-tests".into(),
                    progress: 0.95,
                },
            ],
            rigs: vec![
                RigInfo {
                    name: "frontend".into(),
                    path: "~/projects/webapp/frontend".into(),
                },
                RigInfo {
                    name: "backend".into(),
                    path: "~/projects/webapp/api".into(),
                },
            ],
        }
    }
}

impl DashboardDataSource for SampleDataSource {
    fn fetch(&self) -> Result<DashboardData, DashboardError> {
        Ok(Self::sample_data())
    }

    fn is_available(&self) -> bool {
        true
    }
}

fn main() -> Result<()> {
    env_logger::init();

    Application::new().run(|cx: &mut App| {
        cx.activate(true);
        cx.on_action(quit);

        let size = size(px(800.), px(600.));
        let bounds = Bounds::centered(None, size, cx);

        let data_source: Arc<dyn DashboardDataSource> = Arc::new(SampleDataSource);

        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                titlebar: Some(gpui::TitlebarOptions {
                    title: Some("Gas Town Dashboard".into()),
                    ..Default::default()
                }),
                ..Default::default()
            },
            |_, cx| cx.new(|cx| DashboardView::new(data_source, cx)),
        )
        .expect("Failed to open window");
    });

    Ok(())
}
