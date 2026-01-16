// Town workspace - will be replaced with full implementation from gastown
use std::path::PathBuf;

use gpui::{Context, FocusHandle, Focusable, Render, Window, div, prelude::*};

/// Town represents a ~/gt/ workspace
/// This is a minimal stub - full implementation will come from gastown crate
pub struct Town {
    path: PathBuf,
    focus_handle: FocusHandle,
}

impl Town {
    pub fn new(path: PathBuf, cx: &mut Context<Self>) -> Self {
        Self {
            path,
            focus_handle: cx.focus_handle(),
        }
    }
}

impl Focusable for Town {
    fn focus_handle(&self, _cx: &gpui::App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl gpui::EventEmitter<()> for Town {}

impl Render for Town {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .size_full()
            .items_center()
            .justify_center()
            .child(format!("Town workspace: {:?}", self.path))
    }
}
