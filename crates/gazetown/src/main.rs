// Pure GPUI agent orchestration application
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod town;
mod views;

use std::path::PathBuf;

use assets::Assets;
use gpui::{actions, px, size, App, AppContext, Application, Bounds, WindowBounds, WindowOptions};

use crate::town::Town;

#[cfg(feature = "mimalloc")]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

actions!(gazetown, [Quit]);

fn quit(_: &Quit, cx: &mut App) {
    cx.quit();
}

fn main() {
    env_logger::init();

    let app = Application::new().with_assets(Assets);

    app.run(|cx: &mut App| {
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
                    title: Some("Gazetown".into()),
                    ..Default::default()
                }),
                ..Default::default()
            },
            |_window, cx| cx.new(|cx| Town::new(gt_path, cx)),
        )
        .expect("Failed to open window");
    });
}
