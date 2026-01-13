# Zed Agent Panel Analysis
**File Analyzed:** `crates/agent_ui/src/agent_panel.rs` (3020 lines)
**Analysis Date:** 2026-01-13
**Purpose:** Understand Zed's agent panel architecture to inform Gas Town UI development

---

## Executive Summary

The Zed Agent Panel (`agent_panel.rs`) implements a sophisticated multi-agent chat interface using the GPUI framework. It manages multiple agent types (native Zed Agent, Claude Code, Codex, Gemini, custom agents), provides context management, and handles complex state persistence. The architecture demonstrates patterns that can be adapted for Gas Town's Mayor/Polecat interaction UI.

**Key Strengths:**
- Clear separation between agent types through enum-based abstraction
- Robust state management with Entity pattern and serialization
- Flexible view switching (threads, history, configuration)
- Context persistence across sessions via SQLite

**Adaptation Potential for Gas Town:** High - the core patterns of multi-agent management, thread persistence, and UI state handling map well to Gas Town's needs.

---

## 1. Chat Interface Implementation

### Architecture Overview

The chat interface uses a **view-based architecture** with four primary view types:

```rust
enum ActiveView {
    ExternalAgentThread {
        thread_view: Entity<AcpThreadView>,
    },
    TextThread {
        text_thread_editor: Entity<TextThreadEditor>,
        title_editor: Entity<Editor>,
        buffer_search_bar: Entity<BufferSearchBar>,
        _subscriptions: Vec<gpui::Subscription>,
    },
    History,
    Configuration,
}
```

### View Management Pattern

**File:** `agent_panel.rs:1280-1325`

```rust
fn set_active_view(
    &mut self,
    new_view: ActiveView,
    focus: bool,
    window: &mut Window,
    cx: &mut Context<Self>,
) {
    // Smart view switching: special views (History, Configuration)
    // preserve previous view for "back" navigation
    let current_is_special = current_is_history || current_is_config;
    let new_is_special = new_is_history || new_is_config;

    if current_is_special && !new_is_special {
        self.active_view = new_view;
    } else if !current_is_special && new_is_special {
        self.previous_view = Some(std::mem::replace(&mut self.active_view, new_view));
    }
    // ... focus handling
}
```

**Key Pattern:** The panel maintains `active_view` and `previous_view` to support navigation hierarchies. Special views (History, Configuration) act as overlays that can be dismissed to return to the previous context.

### Thread Creation Flow

**File:** `agent_panel.rs:843-923`

1. User selects agent type from toolbar dropdown
2. `new_agent_thread(agent: AgentType)` dispatched (line 1423)
3. For external agents: `external_thread()` spawns async task to load agent server
4. `_external_thread()` creates `AcpThreadView` entity (line 1484)
5. View set as active, focus granted

```rust
fn _external_thread(
    &mut self,
    server: Rc<dyn AgentServer>,
    resume_thread: Option<DbThreadMetadata>,
    // ...
) {
    let thread_view = cx.new(|cx| {
        crate::acp::AcpThreadView::new(
            server,
            resume_thread,
            summarize_thread,
            workspace.clone(),
            project,
            self.history_store.clone(),
            self.prompt_store.clone(),
            !loading,
            window,
            cx,
        )
    });

    self.set_active_view(
        ActiveView::ExternalAgentThread { thread_view },
        !loading,
        window,
        cx,
    );
}
```

**Gas Town Adaptation:**
- Map `AgentType` to Gas Town agent roles (Mayor, Polecat, Witness, etc.)
- Replace `AcpThreadView` with custom Gas Town thread view
- Maintain same view-switching pattern for navigation

---

## 2. Model Selection Mechanism

### AgentType Enum

**File:** `agent_panel.rs:232-277`

```rust
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub enum AgentType {
    #[default]
    NativeAgent,
    TextThread,
    Gemini,
    ClaudeCode,
    Codex,
    Custom {
        name: SharedString,
    },
}

impl AgentType {
    fn label(&self) -> SharedString {
        match self {
            Self::NativeAgent | Self::TextThread => "Zed Agent".into(),
            Self::Gemini => "Gemini CLI".into(),
            Self::ClaudeCode => "Claude Code".into(),
            Self::Codex => "Codex".into(),
            Self::Custom { name, .. } => name.into(),
        }
    }

    fn icon(&self) -> Option<IconName> {
        match self {
            Self::Gemini => Some(IconName::AiGemini),
            Self::ClaudeCode => Some(IconName::AiClaude),
            Self::Codex => Some(IconName::AiOpenAi),
            Self::Custom { .. } => Some(IconName::Sparkle),
            _ => None,
        }
    }
}
```

**Key Pattern:** The enum centralizes agent metadata (label, icon) while supporting extensibility through the `Custom` variant. This enables dynamic agent registration via extensions.

### Toolbar Agent Selection UI

**File:** `agent_panel.rs:2000-2383`

The toolbar renders a **popover menu** triggered by a "+" button:

```rust
let new_thread_menu = PopoverMenu::new("new_thread_menu")
    .trigger_with_tooltip(
        IconButton::new("new_thread_menu_btn", IconName::Plus)
            .icon_size(IconSize::Small),
        move |_window, cx| {
            Tooltip::for_action_in(
                "New Thread…",
                &ToggleNewThreadMenu,
                &focus_handle,
                cx,
            )
        },
    )
    .anchor(Corner::TopRight)
    .with_handle(self.new_thread_menu_handle.clone())
    .menu(/* build context menu with agent options */)
```

Menu structure (lines 2056-2306):
1. **Native Agents Section:**
   - "Zed Agent" (NativeAgent)
   - "Text Thread" (TextThread)
2. **Separator**
3. **External Agents Section:**
   - Claude Code
   - Codex CLI
   - Gemini CLI
   - Dynamically loaded custom agents from extensions
4. **Separator**
5. **"Add More Agents"** (opens extension marketplace)

**Dynamic Agent Loading (lines 2223-2289):**

```rust
.map(|mut menu| {
    let agent_server_store = agent_server_store.read(cx);
    let agent_names = agent_server_store
        .external_agents()
        .filter(|name| {
            name.0 != GEMINI_NAME
                && name.0 != CLAUDE_CODE_NAME
                && name.0 != CODEX_NAME
        })
        .cloned()
        .collect::<Vec<_>>();

    for agent_name in agent_names {
        let icon_path = agent_server_store.agent_icon(&agent_name);
        let display_name = agent_server_store
            .agent_display_name(&agent_name)
            .unwrap_or_else(|| agent_name.0.clone());

        let mut entry = ContextMenuEntry::new(display_name);

        if let Some(icon_path) = icon_path {
            entry = entry.custom_icon_svg(icon_path);
        } else {
            entry = entry.icon(IconName::Sparkle);
        }
        // ... handler setup
        menu = menu.item(entry);
    }
    menu
})
```

**Gas Town Adaptation:**
- Create `GasTownRole` enum: `Mayor`, `Polecat`, `Witness`, `Deacon`, `Refinery`
- Toolbar shows current active role (e.g., Mayor icon + label)
- Popover menu for switching between roles
- Dynamic loading of polecats from rig configuration
- Use same `ContextMenu` + `PopoverMenu` GPUI components

---

## 3. Context Management

### Core Context Stores

**File:** `agent_panel.rs:418-446`

The `AgentPanel` struct maintains multiple context stores:

```rust
pub struct AgentPanel {
    workspace: WeakEntity<Workspace>,
    project: Entity<Project>,
    fs: Arc<dyn Fs>,
    language_registry: Arc<LanguageRegistry>,

    // History & Thread Management
    acp_history: Entity<AcpThreadHistory>,
    history_store: Entity<agent::HistoryStore>,
    text_thread_store: Entity<assistant_text_thread::TextThreadStore>,

    // Context & Configuration
    prompt_store: Option<Entity<PromptStore>>,
    context_server_registry: Entity<ContextServerRegistry>,
    configuration: Option<Entity<AgentConfiguration>>,

    // UI State
    active_view: ActiveView,
    previous_view: Option<ActiveView>,
    selected_agent: AgentType,
    width: Option<Pixels>,
    zoomed: bool,
    // ... more fields
}
```

### HistoryStore Pattern

**File:** `agent_panel.rs:546-567`

```rust
let history_store = cx.new(|cx| {
    agent::HistoryStore::new(text_thread_store.clone(), cx)
});

let acp_history = cx.new(|cx| {
    AcpThreadHistory::new(history_store.clone(), window, cx)
});

cx.subscribe_in(
    &acp_history,
    window,
    |this, _, event, window, cx| match event {
        ThreadHistoryEvent::Open(HistoryEntry::AcpThread(thread)) => {
            this.external_thread(
                Some(crate::ExternalAgent::NativeAgent),
                Some(thread.clone()),
                None,
                window,
                cx,
            );
        }
        ThreadHistoryEvent::Open(HistoryEntry::TextThread(thread)) => {
            this.open_saved_text_thread(thread.path.clone(), window, cx)
                .detach_and_log_err(cx);
        }
    },
)
.detach();
```

**Key Patterns:**
1. **Entity-based state containers:** `Entity<T>` provides reference-counted shared state with change notifications
2. **Event subscriptions:** `cx.subscribe_in()` creates reactive bindings between entities
3. **Weak references:** `WeakEntity<Workspace>` prevents circular references

### ContextServerRegistry

**File:** `agent_panel.rs:543-544`

```rust
let context_server_registry = cx.new(|cx| {
    ContextServerRegistry::new(project.read(cx).context_server_store(), cx)
});
```

This manages MCP (Model Context Protocol) servers that provide additional context to agents. Relevant for Gas Town's tool/skill integration.

### State Persistence

**File:** `agent_panel.rs:449-464`

```rust
fn serialize(&mut self, cx: &mut Context<Self>) {
    let width = self.width;
    let selected_agent = self.selected_agent.clone();

    self.pending_serialization = Some(cx.background_spawn(async move {
        KEY_VALUE_STORE
            .write_kvp(
                AGENT_PANEL_KEY.into(),
                serde_json::to_string(&SerializedAgentPanel {
                    width,
                    selected_agent: Some(selected_agent),
                })?,
            )
            .await?;
        anyhow::Ok(())
    }));
}
```

**Serialized state:**
```rust
#[derive(Serialize, Deserialize, Debug)]
struct SerializedAgentPanel {
    width: Option<Pixels>,
    selected_agent: Option<AgentType>,
}
```

**Loading state (lines 466-527):**
- Async load from KEY_VALUE_STORE (SQLite-backed)
- Restore panel width and selected agent
- Create appropriate default view

**Gas Town Adaptation:**
- Store selected role (Mayor, specific Polecat, etc.)
- Persist convoy tracking state
- Save panel layout (convoy list visibility, polecat monitor visibility)
- Use same async persistence pattern

---

## 4. State Handling Patterns

### GPUI Entity Pattern

**Core concept:** GPUI uses an Entity-Component-System (ECS) inspired architecture:

- **Entity<T>:** Type-safe handle to component `T` stored in app context
- **Context<T>:** Mutable access to component `T` and app state
- **App:** Global application state container

**Creation:**
```rust
let entity = cx.new(|cx| MyComponent::new(cx));
```

**Access:**
```rust
entity.update(cx, |component, cx| {
    // Mutable access to component
    component.do_something(cx);
});

entity.read(cx).get_value(); // Immutable access
```

### Subscription Pattern

**File:** `agent_panel.rs:548-567, 375-401**

Two main subscription patterns:

**1. Subscribe to entity events:**
```rust
cx.subscribe_in(
    &source_entity,
    window,
    |this, source, event, window, cx| {
        // Handle event
    }
).detach();
```

**2. Subscribe to editor events:**
```rust
window.subscribe(&editor, cx, {
    move |editor, event, window, cx| match event {
        EditorEvent::BufferEdited => { /* handle */ }
        EditorEvent::Blurred => { /* handle */ }
        _ => {}
    }
})
```

### Action Dispatch System

**File:** `agent_panel.rs:83-209**

Actions are registered at workspace level during initialization:

```rust
pub fn init(cx: &mut App) {
    cx.observe_new(|workspace: &mut Workspace, _window, _cx| {
        workspace
            .register_action(|workspace, action: &NewThread, window, cx| {
                if let Some(panel) = workspace.panel::<AgentPanel>(cx) {
                    panel.update(cx, |panel, cx| {
                        panel.new_thread(action, window, cx)
                    });
                    workspace.focus_panel::<AgentPanel>(window, cx);
                }
            })
            .register_action(/* ... more actions ... */)
    })
    .detach();
}
```

**Render-level action handling (lines 2770-2792):**
```rust
v_flex()
    .on_action(cx.listener(|this, action: &NewThread, window, cx| {
        this.new_thread(action, window, cx);
    }))
    .on_action(cx.listener(|this, _: &OpenHistory, window, cx| {
        this.open_history(window, cx);
    }))
    // ... more actions
```

**Gas Town Adaptation:**
- Define Gas Town actions: `SelectRole`, `ViewConvoy`, `MonitorPolecat`, etc.
- Register at workspace level for keyboard shortcuts
- Attach to UI elements for click handlers
- Use same listener pattern for consistency

### Focus Management

**File:** `agent_panel.rs:1526-1543**

```rust
impl Focusable for AgentPanel {
    fn focus_handle(&self, cx: &App) -> FocusHandle {
        match &self.active_view {
            ActiveView::ExternalAgentThread { thread_view, .. } => {
                thread_view.focus_handle(cx)
            }
            ActiveView::History => self.acp_history.focus_handle(cx),
            ActiveView::TextThread { text_thread_editor, .. } => {
                text_thread_editor.focus_handle(cx)
            }
            ActiveView::Configuration => {
                if let Some(configuration) = self.configuration.as_ref() {
                    configuration.focus_handle(cx)
                } else {
                    cx.focus_handle()
                }
            }
        }
    }
}
```

**Pattern:** Focus is delegated to the active view's component, enabling seamless keyboard navigation across different UI modes.

---

## 5. GPUI Integration

### Panel Trait Implementation

**File:** `agent_panel.rs:1551-1601**

```rust
impl Panel for AgentPanel {
    fn persistent_name() -> &'static str {
        "AgentPanel"
    }

    fn panel_key() -> &'static str {
        AGENT_PANEL_KEY
    }

    fn position(&self, _window: &Window, cx: &App) -> DockPosition {
        agent_panel_dock_position(cx)
    }

    fn position_is_valid(&self, position: DockPosition) -> bool {
        position != DockPosition::Bottom
    }

    fn set_position(&mut self, position: DockPosition, _: &mut Window, cx: &mut Context<Self>) {
        settings::update_settings_file(self.fs.clone(), cx, move |settings, _| {
            settings
                .agent
                .get_or_insert_default()
                .set_dock(position.into());
        });
    }

    fn size(&self, window: &Window, cx: &App) -> Pixels {
        let settings = AgentSettings::get_global(cx);
        match self.position(window, cx) {
            DockPosition::Left | DockPosition::Right => {
                self.width.unwrap_or(settings.default_width)
            }
            DockPosition::Bottom => self.height.unwrap_or(settings.default_height),
        }
    }

    fn set_size(&mut self, size: Option<Pixels>, window: &mut Window, cx: &mut Context<Self>) {
        match self.position(window, cx) {
            DockPosition::Left | DockPosition::Right => self.width = size,
            DockPosition::Bottom => self.height = size,
        }
        self.serialize(cx);
        cx.notify();
    }
}
```

**Key methods:**
- `persistent_name()`: Unique identifier for panel type
- `panel_key()`: Key for state persistence
- `position()` / `set_position()`: Docking location (left/right/bottom)
- `size()` / `set_size()`: Panel dimensions with persistence

### Render Trait Implementation

**File:** `agent_panel.rs:2755-2845**

```rust
impl Render for AgentPanel {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let content = v_flex()
            .relative()
            .size_full()
            .justify_between()
            .key_context(self.key_context())
            .on_action(/* ... action handlers ... */)
            .child(self.render_toolbar(window, cx))
            .children(self.render_workspace_trust_message(cx))
            .children(self.render_onboarding(window, cx))
            .map(|parent| match &self.active_view {
                ActiveView::ExternalAgentThread { thread_view, .. } => {
                    parent
                        .child(thread_view.clone())
                        .child(self.render_drag_target(cx))
                }
                ActiveView::History => parent.child(self.acp_history.clone()),
                ActiveView::TextThread {
                    text_thread_editor,
                    buffer_search_bar,
                    ..
                } => parent.child(self.render_text_thread(
                    text_thread_editor,
                    buffer_search_bar,
                    window,
                    cx,
                )),
                ActiveView::Configuration => {
                    parent.children(self.configuration.clone())
                }
            })
            .children(self.render_trial_end_upsell(window, cx));

        // Dynamic font size wrapper based on view type
        match self.active_view.which_font_size_used() {
            WhichFontSize::AgentFont => {
                WithRemSize::new(ThemeSettings::get_global(cx).agent_ui_font_size(cx))
                    .size_full()
                    .child(content)
                    .into_any()
            }
            _ => content.into_any(),
        }
    }
}
```

**Layout structure:**
```
v_flex (vertical flexbox)
├── Toolbar (render_toolbar)
├── Trust message (conditional)
├── Onboarding (conditional)
├── Active view (match on view type)
│   ├── ExternalAgentThread → AcpThreadView + drag target
│   ├── History → AcpThreadHistory
│   ├── TextThread → TextThreadEditor + search bar
│   └── Configuration → AgentConfiguration
└── Trial upsell (conditional)
```

**GPUI Layout Primitives Used:**
- `v_flex()` / `h_flex()`: Vertical/horizontal flexbox containers
- `.size_full()`: 100% width and height
- `.child()`: Add single child element
- `.children()`: Add optional children
- `.map()`: Conditionally transform element
- `.when()` / `.when_some()`: Conditional rendering

### Event Emission

**File:** `agent_panel.rs:1549**

```rust
impl EventEmitter<PanelEvent> for AgentPanel {}
```

Enables the panel to emit events like:
- `PanelEvent::ZoomIn` / `PanelEvent::ZoomOut` (line 1152, 1157)
- Other workspace-level panel events

**Gas Town Usage:** Emit events when convoy states change, polecat status updates, etc., to trigger UI refreshes in other components.

---

## 6. Adaptation Strategy for Mayor/Polecat Interaction

### Recommended Architecture

Based on the Zed Agent Panel patterns, here's a proposed Gas Town UI architecture:

#### 1. Core Panel Structure

```rust
pub struct GasTownPanel {
    workspace: WeakEntity<Workspace>,

    // Gas Town specific stores
    town_state: Entity<TownState>,           // Town-level state (mayor, rigs)
    convoy_store: Entity<ConvoyStore>,        // Convoy tracking
    polecat_monitor: Entity<PolecatMonitor>,  // Polecat health/status
    beads_store: Entity<BeadsStore>,          // Issue tracking

    // UI State
    active_view: GasTownView,
    selected_role: GasTownRole,

    // Layout
    width: Option<Pixels>,
    height: Option<Pixels>,
}

enum GasTownView {
    MayorConsole {
        console_view: Entity<MayorConsoleView>,
    },
    PolecatMonitor {
        monitor_view: Entity<PolecatMonitorView>,
    },
    ConvoyTracker {
        tracker_view: Entity<ConvoyTrackerView>,
    },
    BeadsExplorer {
        explorer_view: Entity<BeadsExplorerView>,
    },
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub enum GasTownRole {
    Mayor,
    Polecat { name: String },
    Witness,
    Deacon,
    Refinery,
}
```

#### 2. View Components

**MayorConsoleView:**
- Chat interface with Mayor
- Command input (convoy creation, work assignment)
- Real-time mayor responses
- Pattern: Similar to `AcpThreadView` from Zed

**PolecatMonitorView:**
- Grid/list of active polecats
- Status indicators (active, in_progress, blocked)
- Live log tailing
- Click to focus individual polecat
- Pattern: Similar to History view but with live updates

**ConvoyTrackerView:**
- List of convoys with progress bars
- Issue completion status per convoy
- Dependency visualization (optional)
- Pattern: Table-based layout with nested issue lists

**BeadsExplorerView:**
- Issue browser (similar to file explorer)
- Filters: status, priority, assignee
- Quick actions: create, edit, close
- Pattern: Tree view with context menus

#### 3. Toolbar Implementation

```rust
fn render_toolbar(&self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
    h_flex()
        .h(Tab::container_height(cx))
        .justify_between()
        .child(
            // Left: Role selector (Mayor icon + dropdown)
            self.render_role_selector(cx)
        )
        .child(
            // Center: View title
            self.render_view_title(cx)
        )
        .child(
            h_flex()
                .gap_2()
                .child(self.render_action_buttons(cx))  // Quick actions
                .child(self.render_options_menu(cx))    // Settings
        )
}

fn render_role_selector(&self, cx: &mut Context<Self>) -> impl IntoElement {
    PopoverMenu::new("role_selector")
        .trigger_with_tooltip(
            h_flex()
                .gap_1()
                .child(Icon::new(self.selected_role.icon()))
                .child(Label::new(self.selected_role.label())),
            |_, cx| Tooltip::text("Switch Role", cx)
        )
        .menu(|window, cx| {
            Some(ContextMenu::build(window, cx, |menu, _, _| {
                menu.header("Town Roles")
                    .item("Mayor")
                    .item("Witness")
                    .item("Deacon")
                    .item("Refinery")
                    .separator()
                    .header("Polecats")
                    // Dynamically load polecats from TownState
                    // ... polecat items ...
            }))
        })
}
```

#### 4. State Management

**TownState Entity:**
```rust
pub struct TownState {
    mayor_status: MayorStatus,
    rigs: Vec<RigInfo>,
    polecats: HashMap<String, PolecatInfo>,
    last_update: Instant,
}

impl TownState {
    pub fn sync_from_gt_commands(&mut self, cx: &mut Context<Self>) {
        // Run `gt status`, `gt agents`, etc. and parse output
        // Emit events on changes
    }
}
```

**ConvoyStore Entity:**
```rust
pub struct ConvoyStore {
    convoys: Vec<ConvoyInfo>,
}

impl ConvoyStore {
    pub fn sync_from_gt_convoy_list(&mut self, cx: &mut Context<Self>) {
        // Run `gt convoy list` and parse
        // Emit ConvoyUpdated events
    }
}
```

**Event-driven updates:**
```rust
cx.subscribe(&self.town_state, |panel, _, event, cx| {
    match event {
        TownStateEvent::PolecatStatusChanged(polecat_id) => {
            panel.refresh_polecat_monitor(cx);
        }
        TownStateEvent::ConvoyCreated(convoy_id) => {
            panel.notify_convoy_created(convoy_id, cx);
        }
    }
});
```

#### 5. Integration with Gas Town CLI

**Command execution pattern:**
```rust
fn execute_gt_command(&self, args: Vec<&str>, cx: &mut Context<Self>) -> Task<Result<String>> {
    cx.background_spawn(async move {
        let output = tokio::process::Command::new("gt")
            .args(&args)
            .output()
            .await?;

        String::from_utf8(output.stdout)
            .map_err(|e| anyhow!("Invalid UTF-8: {}", e))
    })
}

// Usage:
fn create_convoy(&mut self, name: &str, issues: Vec<&str>, cx: &mut Context<Self>) {
    let mut args = vec!["convoy", "create", name];
    args.extend(issues);

    cx.spawn(|this, cx| async move {
        let result = this.update(&cx, |this, cx| {
            this.execute_gt_command(args, cx)
        })?.await?;

        this.update(&cx, |this, cx| {
            this.convoy_store.update(cx, |store, cx| store.sync_from_gt_convoy_list(cx));
        })
    }).detach();
}
```

#### 6. Real-time Updates

**Polling pattern (for initial implementation):**
```rust
impl GasTownPanel {
    fn start_polling(&mut self, cx: &mut Context<Self>) {
        cx.spawn(|this, cx| async move {
            loop {
                tokio::time::sleep(Duration::from_secs(2)).await;

                this.update(&cx, |this, cx| {
                    this.town_state.update(cx, |state, cx| state.sync_from_gt_commands(cx));
                    this.convoy_store.update(cx, |store, cx| store.sync_from_gt_convoy_list(cx));
                }).ok();
            }
        }).detach();
    }
}
```

**Event-driven pattern (future):**
- Gas Town daemon WebSocket API
- Subscribe to town events
- Push updates to UI in real-time

---

## Key Takeaways

### Patterns to Adopt

1. **Entity-based State Management**
   - Use `Entity<T>` for all stateful components
   - Leverage GPUI's reactive subscription system
   - Separate concerns: one entity per logical domain

2. **View Enum Pattern**
   - Define `ActiveView` enum with variants for each major UI mode
   - Maintain `previous_view` for navigation stacks
   - Delegate rendering based on active view

3. **Toolbar as Command Center**
   - Toolbar houses primary navigation (role selector)
   - PopoverMenu for contextual actions
   - IconButtons with tooltips for discoverability

4. **Async Command Execution**
   - All `gt` commands run in background tasks
   - Update UI state after command completion
   - Show loading indicators during execution

5. **Persistence**
   - Serialize critical UI state (selected role, panel size)
   - Use `KEY_VALUE_STORE` for SQLite-backed persistence
   - Async save to avoid blocking UI

### GPUI Components to Use

- **Layout:** `v_flex`, `h_flex`, `div`
- **Buttons:** `IconButton`, `Button`
- **Menus:** `PopoverMenu`, `ContextMenu`
- **Text:** `Label`, `Headline`
- **Lists:** Custom list components with scroll containers
- **Icons:** `Icon::new(IconName::...)`, `Icon::from_external_svg(...)`

### Differences from Zed's Use Case

| Aspect | Zed Agent Panel | Gas Town Panel |
|--------|----------------|----------------|
| **Primary interaction** | Direct chat with single agent | Monitor/coordinate multiple agents |
| **Agent lifecycle** | Long-lived conversation threads | Ephemeral polecats (spawn → work → exit) |
| **State source** | In-memory + SQLite history | External CLI (`gt` commands) |
| **Updates** | User-driven (message send) | Polling + event-driven (agent actions) |
| **Focus** | Content creation | Work orchestration |

### Implementation Priority

**Phase 1: Basic Structure**
1. Create `GasTownPanel` with Panel trait
2. Implement role enum and basic toolbar
3. Single view: Mayor console (chat-like interface)

**Phase 2: Monitoring**
4. Add Polecat monitor view
5. Implement polling for agent status
6. Display convoy list

**Phase 3: Interactivity**
7. Mayor console command input
8. Convoy creation UI
9. Polecat detail view (logs, metrics)

**Phase 4: Polish**
10. Real-time event subscriptions (if daemon WebSocket available)
11. Dependency graphs for convoys
12. Notification system for agent state changes

---

## Code Snippets for Reference

### Creating a Simple View

```rust
impl Render for SimplePanelView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .size_full()
            .gap_2()
            .p_4()
            .child(
                h_flex()
                    .justify_between()
                    .child(Headline::new("Gas Town").size(HeadlineSize::Large))
                    .child(IconButton::new("refresh", IconName::Refresh)
                        .on_click(cx.listener(|this, _, cx| this.refresh(cx))))
            )
            .child(
                div()
                    .flex_1()
                    .overflow_y_scroll()
                    .child(self.render_content(cx))
            )
    }
}
```

### Async Data Loading Pattern

```rust
fn load_data(&mut self, cx: &mut Context<Self>) -> Task<Result<DataType>> {
    cx.spawn(|this, cx| async move {
        // Fetch data
        let data = fetch_from_source().await?;

        // Update state
        this.update(&cx, |this, cx| {
            this.data = Some(data.clone());
            cx.notify();
        })?;

        Ok(data)
    })
}
```

### Subscription to External Entity

```rust
let subscription = cx.subscribe(&self.data_store, |panel, store, event, cx| {
    match event {
        DataStoreEvent::Updated => {
            panel.refresh_view(cx);
        }
    }
});
```

---

## Conclusion

The Zed Agent Panel provides an excellent blueprint for Gas Town's UI. Its architecture demonstrates:
- **Scalable state management** through GPUI's Entity system
- **Flexible view switching** for different workflows
- **Robust persistence** for user preferences
- **Clean separation** between UI and business logic

By adapting these patterns, Gas Town can achieve:
- **Unified interface** for Mayor, Polecats, and other agents
- **Real-time monitoring** of distributed agent work
- **Intuitive navigation** between different operational views
- **Persistent configuration** across sessions

The primary adaptation challenge is integrating with the `gt` CLI rather than in-process APIs, but the async execution patterns demonstrated in Zed's panel provide a solid foundation for this integration.
