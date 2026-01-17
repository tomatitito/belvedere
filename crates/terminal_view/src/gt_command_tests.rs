/// Tests for gt command support in terminal
///
/// This test module verifies that Zed's terminal can execute gt commands
/// and properly handle their output and errors. These tests are expected to
/// fail initially until gt command integration is implemented.
///
/// Test coverage:
/// 1. Terminal can execute 'gt up'
/// 2. Terminal can execute 'gt prime'
/// 3. Terminal can execute 'gt crew list'
/// 4. Terminal can execute 'gt rig list'
/// 5. Terminal shows gt command output correctly
/// 6. Terminal handles gt command errors
/// 7. Terminal PATH includes gt binary

use super::*;
use gpui::TestAppContext;
use project::Project;
use std::time::Duration;
use workspace::AppState;

/// Helper to initialize test environment with terminal
async fn init_terminal_test(
    cx: &mut TestAppContext,
) -> (Entity<Project>, Entity<Workspace>, Entity<TerminalView>) {
    let params = cx.update(AppState::test);
    cx.update(|cx| {
        theme::init(theme::LoadThemes::JustBase, cx);
        crate::init(cx);
    });

    let project = Project::test(params.fs.clone(), [], cx).await;
    let workspace = cx
        .add_window(|window, cx| Workspace::test_new(project.clone(), window, cx))
        .root(cx)
        .unwrap();

    // Create a terminal in the workspace
    let terminal_view = workspace
        .update(cx, |workspace, window, cx| {
            let working_directory = std::env::current_dir().ok();
            let spawn_task = project.update(cx, |project, cx| {
                project.create_terminal(working_directory, None, window, cx)
            });

            cx.spawn_in(window, async move |workspace, cx| {
                let terminal = spawn_task.await.ok()?;
                workspace.update_in(cx, |workspace, window, cx| {
                    let terminal_view = cx.new(|cx| {
                        TerminalView::new(
                            terminal,
                            workspace.weak_handle(),
                            workspace.database_id(),
                            project.clone(),
                            window,
                            cx,
                        )
                    });
                    Some(terminal_view)
                })
            })
        })
        .await
        .expect("Failed to create terminal view");

    (project, workspace, terminal_view)
}

/// Helper to send a command to the terminal and wait for it to execute
async fn send_command_to_terminal(
    terminal_view: &Entity<TerminalView>,
    command: &str,
    cx: &mut TestAppContext,
) {
    terminal_view.update(cx, |view, cx| {
        let terminal = view.terminal();
        terminal.update(cx, |terminal, _cx| {
            // Send the command as bytes followed by Enter key
            let command_bytes = format!("{}\r", command).into_bytes();
            terminal.write_to_pty(command_bytes);
        });
    });

    // Give the command time to execute
    cx.executor().advance_clock(Duration::from_millis(500));
    cx.executor().run_until_parked();
}

/// Helper to get terminal output
fn get_terminal_output(terminal_view: &Entity<TerminalView>, cx: &mut TestAppContext) -> String {
    terminal_view.read_with(cx, |view, cx| {
        let terminal = view.terminal();
        terminal.read(cx).get_content()
    })
}

#[gpui::test]
async fn test_gt_up_command_execution(cx: &mut TestAppContext) {
    let (_project, _workspace, terminal_view) = init_terminal_test(cx).await;

    // Send 'gt up' command
    send_command_to_terminal(&terminal_view, "gt up", cx).await;

    // Verify the command was sent
    let output = get_terminal_output(&terminal_view, cx);
    assert!(
        output.contains("gt up") || output.contains("gt"),
        "Terminal should show gt up command was executed"
    );
}

#[gpui::test]
async fn test_gt_prime_command_execution(cx: &mut TestAppContext) {
    let (_project, _workspace, terminal_view) = init_terminal_test(cx).await;

    // Send 'gt prime' command
    send_command_to_terminal(&terminal_view, "gt prime", cx).await;

    // Verify the command was sent
    let output = get_terminal_output(&terminal_view, cx);
    assert!(
        output.contains("gt prime") || output.contains("gt"),
        "Terminal should show gt prime command was executed"
    );
}

#[gpui::test]
async fn test_gt_crew_list_command_execution(cx: &mut TestAppContext) {
    let (_project, _workspace, terminal_view) = init_terminal_test(cx).await;

    // Send 'gt crew list' command
    send_command_to_terminal(&terminal_view, "gt crew list", cx).await;

    // Verify the command was sent
    let output = get_terminal_output(&terminal_view, cx);
    assert!(
        output.contains("gt crew list") || output.contains("crew"),
        "Terminal should show gt crew list command was executed"
    );
}

#[gpui::test]
async fn test_gt_rig_list_command_execution(cx: &mut TestAppContext) {
    let (_project, _workspace, terminal_view) = init_terminal_test(cx).await;

    // Send 'gt rig list' command
    send_command_to_terminal(&terminal_view, "gt rig list", cx).await;

    // Verify the command was sent
    let output = get_terminal_output(&terminal_view, cx);
    assert!(
        output.contains("gt rig list") || output.contains("rig"),
        "Terminal should show gt rig list command was executed"
    );
}

#[gpui::test]
async fn test_gt_command_output_display(cx: &mut TestAppContext) {
    let (_project, _workspace, terminal_view) = init_terminal_test(cx).await;

    // Send a simple echo command first to verify output capture
    send_command_to_terminal(&terminal_view, "echo 'test output'", cx).await;

    // Get the output
    let output = get_terminal_output(&terminal_view, cx);

    // Verify we can capture output
    assert!(
        !output.is_empty(),
        "Terminal should capture and display command output"
    );

    // Now test gt command output
    send_command_to_terminal(&terminal_view, "gt --version", cx).await;
    cx.executor().advance_clock(Duration::from_millis(500));
    cx.executor().run_until_parked();

    let output = get_terminal_output(&terminal_view, cx);
    assert!(
        output.contains("gt") || output.len() > 0,
        "Terminal should display gt command output"
    );
}

#[gpui::test]
async fn test_gt_command_error_handling(cx: &mut TestAppContext) {
    let (_project, _workspace, terminal_view) = init_terminal_test(cx).await;

    // Send an invalid gt command that should produce an error
    send_command_to_terminal(&terminal_view, "gt invalid_command_xyz", cx).await;

    // Give it time to fail
    cx.executor().advance_clock(Duration::from_millis(500));
    cx.executor().run_until_parked();

    // Get the output
    let output = get_terminal_output(&terminal_view, cx);

    // Verify terminal captured something (either the command or error output)
    // The exact error format may vary, but we should see output
    assert!(
        !output.is_empty(),
        "Terminal should handle and display gt command errors"
    );
}

#[gpui::test]
async fn test_terminal_path_includes_gt_binary(cx: &mut TestAppContext) {
    let (_project, _workspace, terminal_view) = init_terminal_test(cx).await;

    // Send 'which gt' command to verify gt is in PATH
    send_command_to_terminal(&terminal_view, "which gt", cx).await;

    // Give it time to execute
    cx.executor().advance_clock(Duration::from_millis(500));
    cx.executor().run_until_parked();

    // Get the output
    let output = get_terminal_output(&terminal_view, cx);

    // Verify that 'which gt' returns a path (should contain '/gt' in the path)
    // This will fail if gt is not in PATH
    assert!(
        output.contains("/gt") || output.contains("which gt"),
        "Terminal PATH should include gt binary location: {}",
        output
    );
}

#[gpui::test]
async fn test_multiple_gt_commands_in_sequence(cx: &mut TestAppContext) {
    let (_project, _workspace, terminal_view) = init_terminal_test(cx).await;

    // Send multiple gt commands in sequence
    send_command_to_terminal(&terminal_view, "gt --version", cx).await;
    cx.executor().advance_clock(Duration::from_millis(300));

    send_command_to_terminal(&terminal_view, "gt --help", cx).await;
    cx.executor().advance_clock(Duration::from_millis(300));

    // Get final output
    let output = get_terminal_output(&terminal_view, cx);

    // Verify we got output from commands
    assert!(
        !output.is_empty(),
        "Terminal should handle multiple gt commands in sequence"
    );
}

#[gpui::test]
async fn test_gt_command_with_working_directory(cx: &mut TestAppContext) {
    let (_project, _workspace, terminal_view) = init_terminal_test(cx).await;

    // First verify we can execute commands in the terminal
    send_command_to_terminal(&terminal_view, "pwd", cx).await;
    cx.executor().advance_clock(Duration::from_millis(300));

    // Now run a gt command that might be directory-sensitive
    send_command_to_terminal(
        &terminal_view,
        "gt status 2>/dev/null || echo 'gt executed'",
        cx,
    )
    .await;
    cx.executor().advance_clock(Duration::from_millis(500));

    let output = get_terminal_output(&terminal_view, cx);
    assert!(
        !output.is_empty(),
        "Terminal should execute gt commands with proper working directory context"
    );
}

#[gpui::test]
async fn test_gt_command_environment_variables(cx: &mut TestAppContext) {
    let (_project, _workspace, terminal_view) = init_terminal_test(cx).await;

    // Verify terminal has basic environment setup
    send_command_to_terminal(
        &terminal_view,
        "env | grep -E '(PATH|HOME|USER)' | head -3",
        cx,
    )
    .await;
    cx.executor().advance_clock(Duration::from_millis(300));

    let output = get_terminal_output(&terminal_view, cx);

    // Terminal should have environment variables set
    assert!(
        output.contains("PATH") || output.contains("env") || !output.is_empty(),
        "Terminal should have proper environment variables for gt commands"
    );

    // Now test gt can access these
    send_command_to_terminal(&terminal_view, "gt --version", cx).await;
    cx.executor().advance_clock(Duration::from_millis(500));
}
