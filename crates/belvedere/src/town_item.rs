use gpui::{
    AnyElement, App, Context, EventEmitter, Focusable, IntoElement, Render, SharedString, Window,
};
use ui::{Color, Icon, Label, LabelCommon};

/// Event types that TownItems can emit
#[derive(Clone, Copy, Eq, PartialEq, Hash, Debug)]
pub enum TownItemEvent {
    /// Request to close this item
    CloseItem,
    /// Tab content has changed and should be redrawn
    UpdateTab,
    /// Item content has been edited
    Edit,
}

/// Parameters for rendering tab content
#[derive(Clone, Copy, Default, Debug)]
pub struct TabContentParams {
    /// Whether this tab is currently selected
    pub selected: bool,
    /// Whether this tab should be deemphasized (pane not focused)
    pub deemphasized: bool,
}

impl TabContentParams {
    /// Returns the text color to be used for the tab content
    pub fn text_color(&self) -> Color {
        if self.deemphasized {
            if self.selected {
                Color::Muted
            } else {
                Color::Hidden
            }
        } else if self.selected {
            Color::Default
        } else {
            Color::Muted
        }
    }
}

/// Trait for items that can be displayed in the Town center pane
///
/// TownItem is a simplified version of workspace::Item, focused on
/// essential functionality for center pane views in Gazetown.
pub trait TownItem: Focusable + EventEmitter<Self::Event> + Render + Sized {
    /// The event type this item emits
    type Event;

    /// Returns the tab contents as a rendered element
    ///
    /// By default this returns a Label displaying the text from tab_content_text
    fn tab_content(&self, params: TabContentParams, _window: &Window, cx: &App) -> AnyElement {
        Label::new(self.tab_content_text(cx))
            .color(params.text_color())
            .into_any_element()
    }

    /// Returns the textual contents of the tab
    fn tab_content_text(&self, cx: &App) -> SharedString;

    /// Returns an optional icon to display in the tab
    fn tab_icon(&self, _window: &Window, _cx: &App) -> Option<Icon> {
        None
    }

    /// Returns optional tooltip text for the tab
    fn tab_tooltip_text(&self, _cx: &App) -> Option<SharedString> {
        None
    }

    /// Maps this item's events to TownItemEvent
    ///
    /// This allows the town to react to item events (e.g., close requests, tab updates)
    fn to_town_item_events(_event: &Self::Event, _f: impl FnMut(TownItemEvent)) {}

    /// Called when this item is deactivated (loses focus)
    fn deactivated(&mut self, _window: &mut Window, _cx: &mut Context<Self>) {}

    /// Called when this item is removed from the town
    fn on_removed(&self, _cx: &App) {}

    /// Returns whether this item has unsaved changes
    fn is_dirty(&self, _cx: &App) -> bool {
        false
    }

    /// Returns whether this item can be closed
    ///
    /// Items with unsaved changes might want to prevent closing or show a prompt
    fn can_close(&self, _cx: &App) -> bool {
        true
    }
}
