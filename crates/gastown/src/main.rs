use gpui::{div, rgb, App, Application, IntoElement, px, Render};

fn main() {
    Application::new().run(|cx| {
        cx.activate(true);

        let window_options = gpui::WindowOptions {
            window_bounds: Some(gpui::WindowBounds::Windowed(gpui::Bounds {
                origin: gpui::Point {
                    x: px(100.0),
                    y: px(100.0),
                },
                size: gpui::Size {
                    width: px(1024.0),
                    height: px(768.0),
                },
            })),
            titlebar: Some(gpui::TitlebarOptions {
                title: Some("Gas Town".into()),
                ..Default::default()
            }),
            focus: true,
            show: true,
            kind: gpui::WindowKind::Normal,
            ..Default::default()
        };

        cx.open_window(window_options, |_, cx| cx.new(|_| GasTownView))
            .unwrap();
    });
}

struct GasTownView;

impl Render for GasTownView {
    fn render(&mut self, _cx: &mut gpui::Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .items_center()
            .justify_center()
            .size_full()
            .bg(rgb(0x2e3440))
            .text_xl()
            .text_color(rgb(0xd8dee9))
            .child("Gas Town ⚙️")
    }
}
