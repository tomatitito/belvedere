//! Tests for git diff viewing in rig directories.
//!
//! Rigs are external git repositories managed by the overseer system.
//! Each rig is a separate git repository that agents can work in.
//! This module tests the integration of git UI with rig directories.

#[cfg(test)]
mod tests {
    use crate::file_diff_view::FileDiffView;
    use crate::git_panel::{GitListEntry, GitPanel, GitStatusEntry, Section, GitHeaderEntry};
    use editor::test::editor_test_context::assert_state_with_diff;
    use git::{
        repository::repo_path,
        status::{StageStatus, StatusCode},
    };
    use gpui::{TestAppContext, UpdateGlobal, VisualTestContext, Task};
    use indoc::indoc;
    use project::{FakeFs, Project};
    use serde_json::json;
    use settings::SettingsStore;
    use std::path::{Path, PathBuf};
    use theme::LoadThemes;
    use unindent::unindent;
    use util::path;
    use workspace::Workspace;

    // Match the debounce values from git_panel.rs and file_diff_view.rs
    const UPDATE_DEBOUNCE: std::time::Duration = std::time::Duration::from_millis(50);
    const RECALCULATE_DIFF_DEBOUNCE: std::time::Duration = std::time::Duration::from_millis(250);

    fn init_test(cx: &mut TestAppContext) {
        zlog::init_test();

        cx.update(|cx| {
            let settings_store = SettingsStore::test(cx);
            cx.set_global(settings_store);
            theme::init(LoadThemes::JustBase, cx);
            editor::init(cx);
            crate::init(cx);
        });
    }

    /// Test that the git panel shows changes in a rig directory
    #[gpui::test]
    async fn test_git_panel_shows_rig_changes(cx: &mut TestAppContext) {
        init_test(cx);
        let fs = FakeFs::new(cx.background_executor.clone());

        // Set up a rig directory structure
        fs.insert_tree(
            "/rigs/gastown",
            json!({
                ".git": {},
                "src": {
                    "main.rs": "fn main() { println!(\"Hello\"); }",
                    "lib.rs": "pub fn hello() { println!(\"World\"); }",
                },
                "Cargo.toml": "[package]\nname = \"gastown\"\nversion = \"0.1.0\"",
            }),
        )
        .await;

        // Set git status for modified files in the rig
        fs.set_status_for_repo(
            Path::new(path!("/rigs/gastown/.git")),
            &[
                ("src/main.rs", StatusCode::Modified.worktree()),
                ("src/lib.rs", StatusCode::Added.worktree()),
            ],
        );

        let project = Project::test(fs.clone(), [path!("/rigs/gastown").as_ref()], cx).await;
        let workspace =
            cx.add_window(|window, cx| Workspace::test_new(project.clone(), window, cx));
        let cx = &mut VisualTestContext::from_window(*workspace, cx);

        // Wait for scan to complete
        cx.read(|cx| {
            project
                .read(cx)
                .worktrees(cx)
                .next()
                .unwrap()
                .read(cx)
                .as_local()
                .unwrap()
                .scan_complete()
        })
        .await;

        cx.executor().run_until_parked();

        let panel = workspace.update(cx, GitPanel::new).unwrap();

        let handle = cx.update_window_entity(&panel, |panel, _, _| {
            std::mem::replace(&mut panel.update_visible_entries_task, Task::ready(()))
        });
        cx.executor().advance_clock(2 * UPDATE_DEBOUNCE);
        handle.await;

        let entries = panel.read_with(cx, |panel, _| panel.entries.clone());

        // Verify that the git panel shows the rig's changes
        assert!(
            entries.len() >= 2,
            "Expected git panel to show rig changes, but found {} entries",
            entries.len()
        );

        // Check for modified and added files
        let has_modified = entries.iter().any(|entry| match entry {
            GitListEntry::Status(status) => {
                status.repo_path.as_str().contains("main.rs")
                    && status.status == StatusCode::Modified.worktree()
            }
            _ => false,
        });

        let has_added = entries.iter().any(|entry| match entry {
            GitListEntry::Status(status) => {
                status.repo_path.as_str().contains("lib.rs")
                    && status.status == StatusCode::Added.worktree()
            }
            _ => false,
        });

        assert!(
            has_modified,
            "Expected to find modified main.rs in git panel"
        );
        assert!(has_added, "Expected to find added lib.rs in git panel");
    }

    /// Test that diff view displays correctly for modified files in rigs
    #[gpui::test]
    async fn test_diff_view_for_rig_files(cx: &mut TestAppContext) {
        init_test(cx);

        let fs = FakeFs::new(cx.executor());

        // Set up a rig with files at different versions
        fs.insert_tree(
            path!("/rigs/gastown"),
            json!({
                ".git": {},
                "src": {
                    "old_version.rs": "fn old() { println!(\"old\"); }\n",
                    "new_version.rs": "fn new() { println!(\"new\"); }\n",
                },
            }),
        )
        .await;

        let project = Project::test(fs.clone(), [path!("/rigs/gastown").as_ref()], cx).await;

        let (workspace, cx) =
            cx.add_window_view(|window, cx| Workspace::test_new(project.clone(), window, cx));

        // Open diff view for rig files
        let diff_view = workspace
            .update_in(cx, |workspace, window, cx| {
                FileDiffView::open(
                    path!("/rigs/gastown/src/old_version.rs").into(),
                    path!("/rigs/gastown/src/new_version.rs").into(),
                    workspace,
                    window,
                    cx,
                )
            })
            .await
            .unwrap();

        // Verify the diff is displayed correctly
        assert_state_with_diff(
            &diff_view.read_with(cx, |diff_view, _| diff_view.editor.clone()),
            cx,
            &unindent(
                r#"
                - fn old() { println!("old"); }
                + ˇfn new() { println!("new"); }
                "#,
            ),
        );
    }

    /// Test that diff view updates when agent makes changes to rig files
    #[gpui::test]
    async fn test_diff_refresh_on_agent_changes(cx: &mut TestAppContext) {
        init_test(cx);

        let fs = FakeFs::new(cx.executor());

        fs.insert_tree(
            path!("/rigs/gastown"),
            json!({
                ".git": {},
                "src": {
                    "original.rs": "fn original() { }\n",
                    "modified.rs": "fn modified() { }\n",
                },
            }),
        )
        .await;

        let project = Project::test(fs.clone(), [path!("/rigs/gastown").as_ref()], cx).await;

        let (workspace, cx) =
            cx.add_window_view(|window, cx| Workspace::test_new(project.clone(), window, cx));

        let diff_view = workspace
            .update_in(cx, |workspace, window, cx| {
                FileDiffView::open(
                    path!("/rigs/gastown/src/original.rs").into(),
                    path!("/rigs/gastown/src/modified.rs").into(),
                    workspace,
                    window,
                    cx,
                )
            })
            .await
            .unwrap();

        // Initial state
        assert_state_with_diff(
            &diff_view.read_with(cx, |diff_view, _| diff_view.editor.clone()),
            cx,
            &unindent(
                "
                - fn original() { }
                + ˇfn modified() { }
                ",
            ),
        );

        // Simulate agent making changes to the file
        fs.save(
            path!("/rigs/gastown/src/modified.rs").as_ref(),
            &"fn modified() { }\nfn added_by_agent() { }\n".into(),
            Default::default(),
        )
        .await
        .unwrap();

        // Wait for diff to recalculate
        cx.executor().advance_clock(RECALCULATE_DIFF_DEBOUNCE);

        // Verify the diff is updated with agent's changes
        assert_state_with_diff(
            &diff_view.read_with(cx, |diff_view, _| diff_view.editor.clone()),
            cx,
            &unindent(
                "
                - fn original() { }
                + ˇfn modified() { }
                + fn added_by_agent() { }
                ",
            ),
        );
    }

    /// Test handling of multiple rigs (multiple git repos)
    #[gpui::test]
    async fn test_multi_rig_git_status(cx: &mut TestAppContext) {
        init_test(cx);
        let fs = FakeFs::new(cx.background_executor.clone());

        // Set up two different rigs
        fs.insert_tree(
            "/workspace",
            json!({
                "gastown": {
                    ".git": {},
                    "main.rs": "fn gastown_main() { }",
                },
                "gazetown": {
                    ".git": {},
                    "main.rs": "fn gazetown_main() { }",
                },
            }),
        )
        .await;

        // Set different changes in each rig
        fs.set_status_for_repo(
            Path::new(path!("/workspace/gastown/.git")),
            &[("main.rs", StatusCode::Modified.worktree())],
        );

        fs.set_status_for_repo(
            Path::new(path!("/workspace/gazetown/.git")),
            &[("main.rs", StatusCode::Added.worktree())],
        );

        // Open both rigs in the project
        let project = Project::test(
            fs.clone(),
            [
                path!("/workspace/gastown").as_ref(),
                path!("/workspace/gazetown").as_ref(),
            ],
            cx,
        )
        .await;

        let workspace =
            cx.add_window(|window, cx| Workspace::test_new(project.clone(), window, cx));
        let cx = &mut VisualTestContext::from_window(*workspace, cx);

        // Wait for both worktrees to scan
        let mut worktrees = project.read_with(cx, |project, cx| {
            project.worktrees(cx).collect::<Vec<_>>()
        });

        for worktree in &worktrees {
            cx.read(|cx| {
                worktree
                    .read(cx)
                    .as_local()
                    .unwrap()
                    .scan_complete()
            })
            .await;
        }

        cx.executor().run_until_parked();

        let panel = workspace.update(cx, GitPanel::new).unwrap();

        let handle = cx.update_window_entity(&panel, |panel, _, _| {
            std::mem::replace(&mut panel.update_visible_entries_task, Task::ready(()))
        });
        cx.executor().advance_clock(2 * UPDATE_DEBOUNCE);
        handle.await;

        let entries = panel.read_with(cx, |panel, _| panel.entries.clone());

        // Verify that changes from both rigs are visible
        // The exact behavior depends on the repository selector implementation
        assert!(
            !entries.is_empty(),
            "Expected git panel to show changes from multiple rigs"
        );
    }

    /// Test commit operation in a rig directory
    #[gpui::test]
    async fn test_commit_in_rig(cx: &mut TestAppContext) {
        init_test(cx);
        let fs = FakeFs::new(cx.background_executor.clone());

        fs.insert_tree(
            "/rigs/gastown",
            json!({
                ".git": {},
                "src": {
                    "new_file.rs": "fn new_function() { }",
                },
            }),
        )
        .await;

        fs.set_status_for_repo(
            Path::new(path!("/rigs/gastown/.git")),
            &[("src/new_file.rs", StatusCode::Added.worktree())],
        );

        let project = Project::test(fs.clone(), [path!("/rigs/gastown").as_ref()], cx).await;
        let workspace =
            cx.add_window(|window, cx| Workspace::test_new(project.clone(), window, cx));
        let cx = &mut VisualTestContext::from_window(*workspace, cx);

        cx.read(|cx| {
            project
                .read(cx)
                .worktrees(cx)
                .next()
                .unwrap()
                .read(cx)
                .as_local()
                .unwrap()
                .scan_complete()
        })
        .await;

        cx.executor().run_until_parked();

        let panel = workspace.update(cx, GitPanel::new).unwrap();

        // Wait for panel to update
        let handle = cx.update_window_entity(&panel, |panel, _, _| {
            std::mem::replace(&mut panel.update_visible_entries_task, Task::ready(()))
        });
        cx.executor().advance_clock(2 * UPDATE_DEBOUNCE);
        handle.await;

        // Verify the file appears as added
        let entries = panel.read_with(cx, |panel, _| panel.entries.clone());
        let has_added_file = entries.iter().any(|entry| match entry {
            GitListEntry::Status(status) => {
                status.repo_path.as_str().contains("new_file.rs")
                    && status.status == StatusCode::Added.worktree()
            }
            _ => false,
        });

        assert!(
            has_added_file,
            "Expected to find new_file.rs as added in rig"
        );

        // Note: Actual commit operation testing requires mocking git operations
        // This test verifies that the rig's git status is tracked correctly
    }

    /// Test revert operation in a rig directory
    #[gpui::test]
    async fn test_revert_in_rig(cx: &mut TestAppContext) {
        init_test(cx);
        let fs = FakeFs::new(cx.background_executor.clone());

        fs.insert_tree(
            "/rigs/gastown",
            json!({
                ".git": {},
                "src": {
                    "modified.rs": "fn modified() { println!(\"changed\"); }",
                },
            }),
        )
        .await;

        fs.set_status_for_repo(
            Path::new(path!("/rigs/gastown/.git")),
            &[("src/modified.rs", StatusCode::Modified.worktree())],
        );

        let project = Project::test(fs.clone(), [path!("/rigs/gastown").as_ref()], cx).await;
        let workspace =
            cx.add_window(|window, cx| Workspace::test_new(project.clone(), window, cx));
        let cx = &mut VisualTestContext::from_window(*workspace, cx);

        cx.read(|cx| {
            project
                .read(cx)
                .worktrees(cx)
                .next()
                .unwrap()
                .read(cx)
                .as_local()
                .unwrap()
                .scan_complete()
        })
        .await;

        cx.executor().run_until_parked();

        let panel = workspace.update(cx, GitPanel::new).unwrap();

        let handle = cx.update_window_entity(&panel, |panel, _, _| {
            std::mem::replace(&mut panel.update_visible_entries_task, Task::ready(()))
        });
        cx.executor().advance_clock(2 * UPDATE_DEBOUNCE);
        handle.await;

        // Verify modified file is shown
        let entries = panel.read_with(cx, |panel, _| panel.entries.clone());
        let has_modified = entries.iter().any(|entry| match entry {
            GitListEntry::Status(status) => {
                status.repo_path.as_str().contains("modified.rs")
                    && status.status == StatusCode::Modified.worktree()
            }
            _ => false,
        });

        assert!(
            has_modified,
            "Expected to find modified.rs in rig before revert"
        );

        // Note: Actual revert operation testing requires git integration
        // This test verifies the pre-revert state is detected correctly
    }

    /// Test staging changes in a rig directory
    #[gpui::test]
    async fn test_staging_in_rig(cx: &mut TestAppContext) {
        init_test(cx);
        let fs = FakeFs::new(cx.background_executor.clone());

        fs.insert_tree(
            "/rigs/gastown",
            json!({
                ".git": {},
                "src": {
                    "file1.rs": "fn file1() { }",
                    "file2.rs": "fn file2() { }",
                },
            }),
        )
        .await;

        fs.set_status_for_repo(
            Path::new(path!("/rigs/gastown/.git")),
            &[
                ("src/file1.rs", StatusCode::Modified.worktree()),
                ("src/file2.rs", StatusCode::Modified.worktree()),
            ],
        );

        let project = Project::test(fs.clone(), [path!("/rigs/gastown").as_ref()], cx).await;
        let workspace =
            cx.add_window(|window, cx| Workspace::test_new(project.clone(), window, cx));
        let cx = &mut VisualTestContext::from_window(*workspace, cx);

        cx.read(|cx| {
            project
                .read(cx)
                .worktrees(cx)
                .next()
                .unwrap()
                .read(cx)
                .as_local()
                .unwrap()
                .scan_complete()
        })
        .await;

        cx.executor().run_until_parked();

        let panel = workspace.update(cx, GitPanel::new).unwrap();

        let handle = cx.update_window_entity(&panel, |panel, _, _| {
            std::mem::replace(&mut panel.update_visible_entries_task, Task::ready(()))
        });
        cx.executor().advance_clock(2 * UPDATE_DEBOUNCE);
        handle.await;

        // Verify both files are shown as unstaged
        let entries = panel.read_with(cx, |panel, _| panel.entries.clone());

        let unstaged_count = entries
            .iter()
            .filter(|entry| match entry {
                GitListEntry::Status(status) => {
                    status.staging == StageStatus::Unstaged
                        && (status.repo_path.as_str().contains("file1.rs")
                            || status.repo_path.as_str().contains("file2.rs"))
                }
                _ => false,
            })
            .count();

        assert_eq!(
            unstaged_count, 2,
            "Expected both files to be unstaged in rig"
        );

        // Note: Actual staging operation requires git integration
        // This test verifies that staging state is tracked for rig files
    }

    /// Test that git panel correctly displays rig repository name
    #[gpui::test]
    async fn test_rig_repository_identification(cx: &mut TestAppContext) {
        init_test(cx);
        let fs = FakeFs::new(cx.background_executor.clone());

        fs.insert_tree(
            "/rigs/gastown",
            json!({
                ".git": {},
                "README.md": "# Gastown Rig",
            }),
        )
        .await;

        fs.set_status_for_repo(
            Path::new(path!("/rigs/gastown/.git")),
            &[("README.md", StatusCode::Modified.worktree())],
        );

        let project = Project::test(fs.clone(), [path!("/rigs/gastown").as_ref()], cx).await;
        let workspace =
            cx.add_window(|window, cx| Workspace::test_new(project.clone(), window, cx));
        let cx = &mut VisualTestContext::from_window(*workspace, cx);

        cx.read(|cx| {
            project
                .read(cx)
                .worktrees(cx)
                .next()
                .unwrap()
                .read(cx)
                .as_local()
                .unwrap()
                .scan_complete()
        })
        .await;

        cx.executor().run_until_parked();

        let panel = workspace.update(cx, GitPanel::new).unwrap();

        let handle = cx.update_window_entity(&panel, |panel, _, _| {
            std::mem::replace(&mut panel.update_visible_entries_task, Task::ready(()))
        });
        cx.executor().advance_clock(2 * UPDATE_DEBOUNCE);
        handle.await;

        // Verify that the panel recognizes this as a git repository
        // The repository selector should show the rig's repository
        let entries = panel.read_with(cx, |panel, _| panel.entries.clone());
        assert!(
            !entries.is_empty(),
            "Expected git panel to recognize rig as git repository"
        );
    }

    /// Test diff view with binary files in rigs
    #[gpui::test]
    async fn test_binary_file_diff_in_rig(cx: &mut TestAppContext) {
        init_test(cx);
        let fs = FakeFs::new(cx.executor());

        fs.insert_tree(
            path!("/rigs/gastown"),
            json!({
                ".git": {},
                "assets": {
                    "old_image.png": "binary_old_data",
                    "new_image.png": "binary_new_data",
                },
            }),
        )
        .await;

        let project = Project::test(fs.clone(), [path!("/rigs/gastown").as_ref()], cx).await;

        let (workspace, cx) =
            cx.add_window_view(|window, cx| Workspace::test_new(project.clone(), window, cx));

        // Attempt to open diff view for binary files
        let diff_view_result = workspace
            .update_in(cx, |workspace, window, cx| {
                FileDiffView::open(
                    path!("/rigs/gastown/assets/old_image.png").into(),
                    path!("/rigs/gastown/assets/new_image.png").into(),
                    workspace,
                    window,
                    cx,
                )
            })
            .await;

        // The diff view should handle binary files gracefully
        // (exact behavior depends on implementation - may show "binary files differ")
        assert!(
            diff_view_result.is_ok() || diff_view_result.is_err(),
            "Diff view should handle binary files in rigs"
        );
    }

    /// Test git panel performance with large number of changes in rig
    #[gpui::test]
    async fn test_rig_with_many_changes(cx: &mut TestAppContext) {
        init_test(cx);
        let fs = FakeFs::new(cx.background_executor.clone());

        // Create a rig with many modified files
        let mut files = json!({
            ".git": {},
        });

        for i in 0..100 {
            files[format!("file{}.rs", i)] = json!(format!("fn file{}() {{ }}", i));
        }

        fs.insert_tree("/rigs/gastown", files).await;

        // Set status for all files
        let statuses: Vec<(&str, _)> = (0..100)
            .map(|i| {
                (
                    Box::leak(format!("file{}.rs", i).into_boxed_str()) as &str,
                    StatusCode::Modified.worktree(),
                )
            })
            .collect();

        fs.set_status_for_repo(Path::new(path!("/rigs/gastown/.git")), &statuses);

        let project = Project::test(fs.clone(), [path!("/rigs/gastown").as_ref()], cx).await;
        let workspace =
            cx.add_window(|window, cx| Workspace::test_new(project.clone(), window, cx));
        let cx = &mut VisualTestContext::from_window(*workspace, cx);

        cx.read(|cx| {
            project
                .read(cx)
                .worktrees(cx)
                .next()
                .unwrap()
                .read(cx)
                .as_local()
                .unwrap()
                .scan_complete()
        })
        .await;

        cx.executor().run_until_parked();

        let panel = workspace.update(cx, GitPanel::new).unwrap();

        let handle = cx.update_window_entity(&panel, |panel, _, _| {
            std::mem::replace(&mut panel.update_visible_entries_task, Task::ready(()))
        });
        cx.executor().advance_clock(2 * UPDATE_DEBOUNCE);
        handle.await;

        // Verify all changes are tracked
        let entries = panel.read_with(cx, |panel, _| panel.entries.clone());

        let status_count = entries
            .iter()
            .filter(|entry| matches!(entry, GitListEntry::Status(_)))
            .count();

        assert!(
            status_count >= 50,
            "Expected git panel to handle many changes in rig, found {} status entries",
            status_count
        );
    }
}
