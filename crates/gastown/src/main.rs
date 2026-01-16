use anyhow::Result;
use gastown::Town;
use gpui::{App, AppContext, Application, Bounds, WindowBounds, WindowOptions, actions, px, size};
use std::path::PathBuf;

actions!(gastown, [Quit]);

fn quit(_: &Quit, cx: &mut App) {
    cx.quit();
}

fn main() -> Result<()> {
    env_logger::init();

    Application::new().run(|cx: &mut App| {
        cx.activate(true);
        cx.on_action(quit);

        let size = size(px(1200.), px(800.));
        let bounds = Bounds::centered(None, size, cx);

        // Default to ~/gt/ directory
        let gt_path = dirs::home_dir()
            .map(|home| home.join("gt"))
            .unwrap_or_else(|| PathBuf::from("gt"));

        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                titlebar: Some(gpui::TitlebarOptions {
                    title: Some("Gas Town".into()),
                    ..Default::default()
                }),
                ..Default::default()
            },
            |_, cx| cx.new(|cx| Town::new(gt_path, cx)),
        )
        .expect("Failed to open window");
    });

    Ok(())
}
