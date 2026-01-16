// Belvedere - Multi-agent development workspace
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use anyhow::Result;
use assets::Assets;
use belvedere::Town;
use gpui::{App, AppContext, Application, Bounds, WindowBounds, WindowOptions, actions, px, size};
use std::path::PathBuf;

actions!(belvedere, [Quit]);

fn quit(_: &Quit, cx: &mut App) {
    cx.quit();
}

fn main() -> Result<()> {
    env_logger::init();

    let app = Application::new().with_assets(Assets);

    app.run(|cx: &mut App| {
        settings::init(cx);
        theme::init(theme::LoadThemes::JustBase, cx);

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
                    title: Some("Belvedere".into()),
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
