use collections::HashMap;
use gpui::{
    div, prelude::*, AnyView, Context, FocusHandle, Focusable, Render, Window,
};
use ui::ActiveTheme;

/// Holds the tabbed items in the center pane
struct CenterPane {
    /// List of open items (as views)
    items: Vec<AnyView>,
    /// Index of the currently active item
    active_index: usize,
}

impl CenterPane {
    fn new() -> Self {
        Self {
            items: Vec::new(),
            active_index: 0,
        }
    }

    fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    fn active_item(&self) -> Option<&AnyView> {
        self.items.get(self.active_index)
    }

    fn add_item(&mut self, item: AnyView) {
        self.items.push(item);
        self.active_index = self.items.len().saturating_sub(1);
    }

    fn close_item(&mut self, index: usize) -> Option<AnyView> {
        if index < self.items.len() {
            let removed = self.items.remove(index);
            if self.active_index >= self.items.len() && !self.items.is_empty() {
                self.active_index = self.items.len() - 1;
            }
            Some(removed)
        } else {
            None
        }
    }

    fn set_active(&mut self, index: usize) {
        if index < self.items.len() {
            self.active_index = index;
        }
    }
}

/// Town represents a ~/gt/ workspace.
///
/// This is the root entity for the Gastown application, analogous to Workspace in Zed.
/// It manages the overall layout with dock areas and center pane for displaying items.
pub struct Town {
    /// Path to the ~/gt/ directory
    pub path: std::path::PathBuf,

    /// Collection of rig directories
    pub rigs: HashMap<String, ()>,

    /// Discovered agent instances
    pub agents: HashMap<String, ()>,

    /// Multi-agent coordination groups
    pub convoys: HashMap<String, ()>,

    /// Center pane holding tabbed items
    center_pane: CenterPane,

    /// Focus handle for keyboard navigation
    pub focus_handle: FocusHandle,
}

impl Town {
    pub fn new(path: std::path::PathBuf, cx: &mut Context<Self>) -> Self {
        Self {
            path,
            rigs: HashMap::default(),
            agents: HashMap::default(),
            convoys: HashMap::default(),
            center_pane: CenterPane::new(),
            focus_handle: cx.focus_handle(),
        }
    }

    /// Opens a new item in the center pane
    pub fn open_item(&mut self, item: AnyView, cx: &mut Context<Self>) {
        self.center_pane.add_item(item);
        cx.notify();
    }

    /// Returns the currently active item in the center pane
    pub fn active_item(&self) -> Option<&AnyView> {
        self.center_pane.active_item()
    }

    /// Closes an item at the specified index
    pub fn close_item(&mut self, index: usize, cx: &mut Context<Self>) -> Option<AnyView> {
        let removed = self.center_pane.close_item(index);
        cx.notify();
        removed
    }

    /// Sets the active item by index
    pub fn set_active_item(&mut self, index: usize, cx: &mut Context<Self>) {
        self.center_pane.set_active(index);
        cx.notify();
    }

    fn render_center_pane(&mut self, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = cx.theme().colors();

        div()
            .id("center-pane")
            .flex()
            .flex_col()
            .flex_1()
            .h_full()
            .bg(colors.editor_background)
            .when(!self.center_pane.is_empty(), |div| {
                div.child(self.render_tabs(cx))
                   .child(self.render_active_item_content(cx))
            })
            .when(self.center_pane.is_empty(), |div| {
                div.items_center()
                   .justify_center()
                   .child("No items open")
            })
    }

    fn render_tabs(&mut self, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = cx.theme().colors();
        let active_index = self.center_pane.active_index;

        div()
            .id("tabs")
            .flex()
            .flex_row()
            .h_8()
            .bg(colors.tab_bar_background)
            .border_b_1()
            .border_color(colors.border)
            .children(
                self.center_pane.items.iter().enumerate().map(|(index, _)| {
                    let is_active = index == active_index;
                    div()
                        .id(("tab", index))
                        .flex()
                        .items_center()
                        .px_3()
                        .h_full()
                        .when(is_active, |div| {
                            div.bg(colors.tab_active_background)
                               .border_b_2()
                               .border_color(colors.element_selected)
                        })
                        .when(!is_active, |div| {
                            div.bg(colors.tab_inactive_background)
                        })
                        .child(format!("Item {}", index + 1))
                })
            )
    }

    fn render_active_item_content(&mut self, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .id("item-content")
            .flex()
            .flex_1()
            .size_full()
            .when_some(self.center_pane.active_item(), |div, view| {
                div.child(view.clone())
            })
    }
}

impl Focusable for Town {
    fn focus_handle(&self, _cx: &gpui::App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl gpui::EventEmitter<()> for Town {}

impl Render for Town {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_row()
            .size_full()
            .bg(cx.theme().colors().editor_background)
            .child(
                // Left dock area (placeholder)
                div()
                    .id("left-dock")
                    .flex()
                    .flex_col()
                    .w_64()
                    .h_full()
                    .bg(cx.theme().colors().panel_background)
                    .border_r_1()
                    .border_color(cx.theme().colors().border)
                    .child("Left Dock")
            )
            .child(self.render_center_pane(cx))
            .child(
                // Right dock area (placeholder)
                div()
                    .id("right-dock")
                    .flex()
                    .flex_col()
                    .w_64()
                    .h_full()
                    .bg(cx.theme().colors().panel_background)
                    .border_l_1()
                    .border_color(cx.theme().colors().border)
                    .child("Right Dock")
            )
    }
}
