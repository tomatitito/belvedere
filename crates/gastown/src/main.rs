use anyhow::Result;
use gpui::{
    App, Application, Bounds, Context, Window, WindowBounds, WindowOptions, actions, div,
    prelude::*, px, size,
};

mod dashboard_buffer;

#[cfg(test)]
mod dashboard_buffer_tests;

actions!(gastown, [Quit]);

fn quit(_: &Quit, cx: &mut App) {
    cx.quit();
}

struct GasTown;

impl Render for GasTown {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .items_center()
            .justify_center()
            .size_full()
            .child("Gas Town - Multi-Agent Development Workspace")
    }
}

fn main() -> Result<()> {
    env_logger::init();

    Application::new().run(|cx: &mut App| {
        cx.activate(true);
        cx.on_action(quit);

        let size = size(px(1200.), px(800.));
        let bounds = Bounds::centered(None, size, cx);

        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                titlebar: Some(gpui::TitlebarOptions {
                    title: Some("Gas Town".into()),
                    ..Default::default()
                }),
                ..Default::default()
            },
            |_, cx| cx.new(|_| GasTown),
        )
        .expect("Failed to open window");
    });

    Ok(())
}
