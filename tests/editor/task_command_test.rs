use dmacs::editor::{Editor, EditorMode};
use pancurses::Input;

// Helper to create an editor with some initial content
fn setup_editor(content: &[&str]) -> Editor {
    let mut editor = Editor::new(None);
    editor.document.lines = content.iter().map(|&s| s.to_string()).collect();
    editor.cursor_y = 0;
    editor.cursor_x = 0;
    editor
}

#[test]
fn test_task_command_enter_mode_and_find_tasks() {
    let mut editor = setup_editor(&[
        "Task list:",
        "- [ ] Task 1",
        "Some other line",
        "- [ ] Task 2",
        "- [x] Done Task",
        "- [ ] Task 3",
    ]);
    editor.cursor_y = 0; // Cursor at "Task list:"
    editor.cursor_x = 0;

    // Simulate typing "/task" and pressing Enter
    editor.document.lines.insert(0, "/task".to_string()); // Insert /task at the beginning
    editor.cursor_y = 0; // Cursor on the /task line
    editor.cursor_x = 5; // Cursor at the end of /task
    editor.insert_newline().unwrap(); // Now call insert_newline

    assert_eq!(editor.mode, EditorMode::TaskSelection);
    assert_eq!(editor.task.tasks.len(), 3);
    assert_eq!(editor.task.tasks[0].1, "- [ ] Task 1");
    assert_eq!(editor.task.tasks[1].1, "- [ ] Task 2");
    assert_eq!(editor.task.tasks[2].1, "- [ ] Task 3");
    assert_eq!(editor.task.selected_task_index, Some(0));
    assert_eq!(
        editor.status_message,
        "Found 3 unchecked tasks. Use Up/Down to select, SPACE to move, ESC/ENTER to exit."
    );

    // Ensure "/task" command is removed
    assert_eq!(editor.document.lines.len(), 7); // Original 6 lines + 1
    assert_eq!(editor.document.lines[0], "");
}

#[test]
fn test_task_command_no_tasks_found() {
    let mut editor = setup_editor(&["No tasks here", "Another line", "- [x] Done Task"]);
    editor.cursor_y = 0;
    editor.cursor_x = 0;

    editor.document.lines.insert(0, "/task".to_string()); // Insert /task at the beginning
    editor.cursor_y = 0; // Cursor on the /task line
    editor.cursor_x = 5; // Cursor at the end of /task
    editor.insert_newline().unwrap(); // Now call insert_newline

    assert_eq!(editor.mode, EditorMode::TaskSelection); // Still enters mode
    assert!(editor.task.tasks.is_empty());
    assert_eq!(editor.task.selected_task_index, None);
    assert_eq!(
        editor.status_message,
        "No unchecked tasks found below current line."
    );
}

#[test]
fn test_task_command_navigate_tasks() {
    let mut editor = setup_editor(&["Start", "- [ ] A", "- [ ] B", "- [ ] C"]);
    editor.cursor_y = 0;
    editor.cursor_x = 0;
    editor.document.lines.insert(0, "/task".to_string()); // Insert /task at the beginning
    editor.cursor_y = 0; // Cursor on the /task line
    editor.cursor_x = 5; // Cursor at the end of /task
    editor.insert_newline().unwrap(); // Now call insert_newline // Enter task selection mode

    assert_eq!(editor.task.selected_task_index, Some(0)); // Task A

    // Move down
    editor.handle_task_selection_input(Input::KeyDown);
    assert_eq!(editor.task.selected_task_index, Some(1)); // Task B

    editor.handle_task_selection_input(Input::KeyDown);
    assert_eq!(editor.task.selected_task_index, Some(2)); // Task C

    // Wrap around down
    editor.handle_task_selection_input(Input::KeyDown);
    assert_eq!(editor.task.selected_task_index, Some(0)); // Task A

    // Move up
    editor.handle_task_selection_input(Input::KeyUp);
    assert_eq!(editor.task.selected_task_index, Some(2)); // Task C

    editor.handle_task_selection_input(Input::KeyUp);
    assert_eq!(editor.task.selected_task_index, Some(1)); // Task B
}

#[test]
fn test_task_command_move_task() {
    let mut editor = setup_editor(&[
        "Current line",
        "- [ ] Task 1",
        "Middle line",
        "- [ ] Task 2",
        "End line",
    ]);
    editor.cursor_y = 0; // Cursor at "Current line"
    editor.cursor_x = 0;

    editor.document.lines.insert(0, "/task".to_string()); // Insert /task at the beginning
    editor.cursor_y = 0; // Cursor on the /task line
    editor.cursor_x = 5; // Cursor at the end of /task
    editor.insert_newline().unwrap(); // Now call insert_newline // Enter task selection mode

    assert_eq!(editor.document.lines.len(), 6);
    assert_eq!(editor.document.lines[0], ""); // Empty line
    assert_eq!(editor.document.lines[1], "Current line");
    assert_eq!(editor.document.lines[2], "- [ ] Task 1");
    assert_eq!(editor.document.lines[3], "Middle line");
    assert_eq!(editor.document.lines[4], "- [ ] Task 2"); // Original Task 1 removed, so Task 2 is now at index 3
    assert_eq!(editor.document.lines[5], "End line");

    assert_eq!(editor.task.tasks.len(), 2);
    assert_eq!(editor.task.tasks[0].1, "- [ ] Task 1");
    assert_eq!(editor.task.tasks[1].1, "- [ ] Task 2");
    assert_eq!(editor.task.selected_task_index, Some(0));

    // Move Task 1
    editor.handle_task_selection_input(Input::Character(' ')); // Press SPACE

    assert_eq!(editor.document.lines.len(), 6);
    assert_eq!(editor.document.lines[0], "- [ ] Task 1"); // Task 1 moved here
    assert_eq!(editor.document.lines[1], "");
    assert_eq!(editor.document.lines[2], "Current line");
    assert_eq!(editor.document.lines[3], "Middle line");
    assert_eq!(editor.document.lines[4], "- [ ] Task 2"); // Original Task 1 removed, so Task 2 is now at index 3
    assert_eq!(editor.document.lines[5], "End line");

    assert_eq!(editor.task.tasks.len(), 1); // Task 1 moved, only Task 2 remains
    assert_eq!(editor.task.tasks[0].1, "- [ ] Task 2");
    assert_eq!(editor.task.selected_task_index, Some(0)); // Still selects the first remaining task

    // Move Task 2
    editor.handle_task_selection_input(Input::Character(' ')); // Press SPACE again

    assert_eq!(editor.document.lines.len(), 6);
    assert_eq!(editor.document.lines[0], "- [ ] Task 1");
    assert_eq!(editor.document.lines[1], "- [ ] Task 2"); // Task 2 moved here
    assert_eq!(editor.document.lines[2], "");
    assert_eq!(editor.document.lines[3], "Current line");
    assert_eq!(editor.document.lines[4], "Middle line");
    assert_eq!(editor.document.lines[5], "End line");

    assert!(editor.task.tasks.is_empty()); // All tasks moved
    assert_eq!(editor.mode, EditorMode::Normal); // Should exit mode
    assert_eq!(
        editor.status_message,
        "All tasks moved. Exiting task selection mode."
    );
}

#[test]
fn test_task_command_exit_mode() {
    let mut editor = setup_editor(&["Start", "- [ ] A", "- [ ] B"]);
    editor.cursor_y = 0;
    editor.cursor_x = 0;
    editor.document.lines.insert(0, "/task".to_string()); // Insert /task at the beginning
    editor.cursor_y = 0; // Cursor on the /task line
    editor.cursor_x = 5; // Cursor at the end of /task
    editor.insert_newline().unwrap(); // Now call insert_newline // Enter task selection mode

    assert_eq!(editor.mode, EditorMode::TaskSelection);

    // Exit with ESC
    editor.handle_task_selection_input(Input::Character('\x1b'));
    assert_eq!(editor.mode, EditorMode::Normal);
    assert!(editor.task.tasks.is_empty());
    assert_eq!(editor.status_message, "Exited task selection mode.");

    // Re-enter mode
    editor.document.lines.insert(0, "/task".to_string()); // Insert /task at the beginning
    editor.cursor_y = 0; // Cursor on the /task line
    editor.cursor_x = 5; // Cursor at the end of /task
    editor.insert_newline().unwrap(); // Now call insert_newline
    assert_eq!(editor.mode, EditorMode::TaskSelection);

    // Exit with Enter
    editor.handle_task_selection_input(Input::Character('\x0a'));
    assert_eq!(editor.mode, EditorMode::Normal);
    assert!(editor.task.tasks.is_empty());
    assert_eq!(editor.status_message, "Exited task selection mode.");
}

#[test]
fn test_task_command_scroll_tasks() {
    let mut editor = setup_editor(&[
        "Start",
        "- [ ] Task 0",
        "- [ ] Task 1",
        "- [ ] Task 2",
        "- [ ] Task 3",
        "- [ ] Task 4",
        "- [ ] Task 5",
        "- [ ] Task 6",
        "- [ ] Task 7",
        "- [ ] Task 8",
        "- [ ] Task 9",
    ]);
    editor.cursor_y = 0;
    editor.cursor_x = 0;
    editor.document.lines.insert(0, "/task".to_string()); // Insert /task at the beginning
    editor.cursor_y = 0; // Cursor on the /task line
    editor.cursor_x = 5; // Cursor at the end of /task
    editor.insert_newline().unwrap(); // Now call insert_newline // Enter task selection mode

    // Initial state
    assert_eq!(editor.mode, EditorMode::TaskSelection);
    assert_eq!(editor.task.tasks.len(), 10);
    assert_eq!(editor.task.selected_task_index, Some(0));
    assert_eq!(editor.task.task_display_offset, 0);

    // Scroll down past visible area (TASK_UI_HEIGHT - 1 = 6 visible rows)
    // Tasks 0-5 are visible initially.
    // Move to Task 6 (index 6)
    for _ in 0..6 {
        editor.handle_task_selection_input(Input::KeyDown);
    }
    assert_eq!(editor.task.selected_task_index, Some(6));
    // task_display_offset should be 1 (tasks 1-6 visible)
    assert_eq!(editor.task.task_display_offset, 1);

    // Move to Task 7 (index 7)
    editor.handle_task_selection_input(Input::KeyDown);
    assert_eq!(editor.task.selected_task_index, Some(7));
    // task_display_offset should be 2 (tasks 2-7 visible)
    assert_eq!(editor.task.task_display_offset, 2);

    // Move to Task 9 (index 9) - last task
    for _ in 0..2 {
        // From 7 to 9
        editor.handle_task_selection_input(Input::KeyDown);
    }
    assert_eq!(editor.task.selected_task_index, Some(9));
    // task_display_offset should be 4 (tasks 4-9 visible)
    assert_eq!(editor.task.task_display_offset, 4);

    // Wrap around to Task 0
    editor.handle_task_selection_input(Input::KeyDown);
    assert_eq!(editor.task.selected_task_index, Some(0));
    assert_eq!(editor.task.task_display_offset, 0);

    // Scroll up
    // Move to Task 9 (index 9)
    editor.handle_task_selection_input(Input::KeyUp);
    assert_eq!(editor.task.selected_task_index, Some(9));
    assert_eq!(editor.task.task_display_offset, 4); // Should be 4

    // Move to Task 8 (index 8)
    editor.handle_task_selection_input(Input::KeyUp);
    assert_eq!(editor.task.selected_task_index, Some(8));
    assert_eq!(editor.task.task_display_offset, 4); // Should be 4

    // Move to Task 0 (index 0)
    for _ in 0..8 {
        // From 8 to 0
        editor.handle_task_selection_input(Input::KeyUp);
    }
    assert_eq!(editor.task.selected_task_index, Some(0));
    assert_eq!(editor.task.task_display_offset, 0);
}
