use dmacs::editor::{Editor, EditorMode};
use pancurses::Input;

// Helper to create an editor with some initial content
fn setup_editor(content: &[&str]) -> Editor {
    let mut editor = Editor::new(None, None, None);
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

    // Set screen size for the test
    editor.update_screen_size(25, 80); // 25 rows, 80 cols. task_ui_height = 10, task_list_visible_rows = 9

    // Initial state
    assert_eq!(editor.mode, EditorMode::TaskSelection);
    assert_eq!(editor.task.tasks.len(), 10);
    assert_eq!(editor.task.selected_task_index, Some(0));
    assert_eq!(editor.task.task_display_offset, 0);

    // Scroll down within visible area (9 visible rows: 0-8)
    // Move to Task 8 (index 8)
    for _ in 0..8 {
        editor.handle_task_selection_input(Input::KeyDown);
    }
    assert_eq!(editor.task.selected_task_index, Some(8));
    assert_eq!(editor.task.task_display_offset, 0); // Still 0, as Task 8 is visible

    // Move to Task 9 (index 9) - this should cause scroll
    editor.handle_task_selection_input(Input::KeyDown);
    assert_eq!(editor.task.selected_task_index, Some(9));
    // task_display_offset should be 1 (tasks 1-9 visible)
    assert_eq!(editor.task.task_display_offset, 1);

    // Wrap around to Task 0
    editor.handle_task_selection_input(Input::KeyDown);
    assert_eq!(editor.task.selected_task_index, Some(0));
    assert_eq!(editor.task.task_display_offset, 0);

    // Scroll up
    // Move to Task 9 (index 9)
    editor.handle_task_selection_input(Input::KeyUp);
    assert_eq!(editor.task.selected_task_index, Some(9));
    // When wrapping from 0 to 9, the offset should be adjusted to show 9.
    // If 9 rows are visible, and 10 tasks (0-9), then to show 9, the offset should be 1.
    // (tasks.len() - visible_rows) = 10 - 9 = 1
    assert_eq!(editor.task.task_display_offset, 1);

    // Move to Task 8 (index 8)
    editor.handle_task_selection_input(Input::KeyUp);
    assert_eq!(editor.task.selected_task_index, Some(8));
    assert_eq!(editor.task.task_display_offset, 1); // Should be 1, as 8 is visible with offset 1

    // Move to Task 0 (index 0)
    for _ in 0..8 {
        // From 8 to 0
        editor.handle_task_selection_input(Input::KeyUp);
    }
    assert_eq!(editor.task.selected_task_index, Some(0));
    assert_eq!(editor.task.task_display_offset, 0);
}

#[test]
fn test_task_command_move_task_bug() {
    let mut editor = setup_editor(&["- [ ] Task 1", "- [ ] Task 2"]);
    editor.cursor_y = 0; // Cursor at "Current line"
    editor.cursor_x = 0;

    editor.document.lines.insert(0, "/task".to_string()); // Insert /task at the beginning
    editor.cursor_y = 0; // Cursor on the /task line
    editor.cursor_x = 5; // Cursor at the end of /task
    editor.insert_newline().unwrap(); // Now call insert_newline // Enter task selection mode

    assert_eq!(editor.document.lines.len(), 3);
    assert_eq!(editor.document.lines[0], ""); // Empty line
    assert_eq!(editor.document.lines[1], "- [ ] Task 1");
    assert_eq!(editor.document.lines[2], "- [ ] Task 2"); // Original Task 1 removed, so Task 2 is now at index 3

    assert_eq!(editor.task.tasks.len(), 2);
    assert_eq!(editor.task.tasks[0].1, "- [ ] Task 1");
    assert_eq!(editor.task.tasks[1].1, "- [ ] Task 2");
    assert_eq!(editor.task.selected_task_index, Some(0));

    // Move Task 2
    editor.handle_task_selection_input(Input::KeyDown); // Press DOWN
    editor.handle_task_selection_input(Input::Character(' ')); // Press SPACE

    assert_eq!(editor.document.lines.len(), 3);
    assert_eq!(editor.document.lines[0], "- [ ] Task 2");
    assert_eq!(editor.document.lines[1], ""); // Empty line
    assert_eq!(editor.document.lines[2], "- [ ] Task 1");

    assert_eq!(editor.task.tasks.len(), 1); // Task 2 moved, only Task 1 remains
    assert_eq!(editor.task.tasks[0].1, "- [ ] Task 1");
    assert_eq!(editor.task.selected_task_index, Some(0)); // Still selects the first remaining task

    // Move Task 1
    editor.handle_task_selection_input(Input::Character(' ')); // Press SPACE again

    assert_eq!(editor.document.lines.len(), 3);
    assert_eq!(editor.document.lines[0], "- [ ] Task 2");
    assert_eq!(editor.document.lines[1], "- [ ] Task 1");
    assert_eq!(editor.document.lines[2], "");

    assert!(editor.task.tasks.is_empty()); // All tasks moved
    assert_eq!(editor.mode, EditorMode::Normal); // Should exit mode
    assert_eq!(
        editor.status_message,
        "All tasks moved. Exiting task selection mode."
    );
}

#[test]
fn test_task_command_comment_out_task() {
    let mut editor = setup_editor(&["Task list:", "- [ ] Task 1", "- [ ] Task 2", "- [ ] Task 3"]);
    editor.cursor_y = 0;
    editor.cursor_x = 0;

    // Enter task selection mode
    editor.document.lines.insert(0, "/task".to_string());
    editor.cursor_y = 0;
    editor.cursor_x = 5;
    editor.insert_newline().unwrap();

    // Initial state: 3 tasks found
    assert_eq!(editor.mode, EditorMode::TaskSelection);
    assert_eq!(editor.task.tasks.len(), 3);
    assert_eq!(editor.task.selected_task_index, Some(0)); // Task 1

    // Select Task 2
    editor.handle_task_selection_input(Input::KeyDown);
    assert_eq!(editor.task.selected_task_index, Some(1));

    // Comment out Task 2
    editor.handle_task_selection_input(Input::Character('#'));

    // Assertions for commenting out Task 2
    // The original line index of Task 2 is 3 (after /task line processing)
    assert_eq!(editor.document.lines[3], "# - [ ] Task 2");
    assert_eq!(editor.task.tasks.len(), 2);
    assert_eq!(editor.task.tasks[0].1, "- [ ] Task 1");
    assert_eq!(editor.task.tasks[1].1, "- [ ] Task 3");
    assert_eq!(editor.task.selected_task_index, Some(1)); // Selection moves to Task 3
    assert_eq!(
        editor.status_message,
        "Task commented out. 2 tasks remaining."
    );

    // Now, the selected task is Task 3 (index 1 in the list). Comment it out.
    editor.handle_task_selection_input(Input::Character('#'));

    // Assertions for commenting out Task 3
    assert_eq!(editor.document.lines[4], "# - [ ] Task 3");
    assert_eq!(editor.task.tasks.len(), 1);
    assert_eq!(editor.task.tasks[0].1, "- [ ] Task 1");
    assert_eq!(editor.task.selected_task_index, Some(0)); // Selection moves to last item

    // Comment out the final task, Task 1
    editor.handle_task_selection_input(Input::Character('#'));

    // Assertions for commenting out the last task
    assert_eq!(editor.document.lines[2], "# - [ ] Task 1");
    assert!(editor.task.tasks.is_empty());
    assert_eq!(editor.task.selected_task_index, None);
    assert_eq!(editor.mode, EditorMode::Normal); // Should exit mode
    assert_eq!(
        editor.status_message,
        "All tasks handled. Exiting task selection mode."
    );
}

#[test]
fn test_task_command_comment_out_undo_redo() {
    let mut editor = setup_editor(&["Task list:", "- [ ] The only task"]);
    editor.cursor_y = 0;
    editor.cursor_x = 0;

    // Enter task selection mode
    editor.document.lines.insert(0, "/task".to_string());
    editor.cursor_y = 0;
    editor.cursor_x = 5;
    editor.insert_newline().unwrap();

    // Initial state: 1 task found
    assert_eq!(editor.mode, EditorMode::TaskSelection);
    assert_eq!(editor.task.tasks.len(), 1);
    assert_eq!(editor.task.selected_task_index, Some(0));

    // Comment out the task
    editor.handle_task_selection_input(Input::Character('#'));

    // Assertions for commenting out the task
    assert_eq!(editor.mode, EditorMode::Normal);
    assert_eq!(editor.document.lines[2], "# - [ ] The only task");
    assert_eq!(
        editor.status_message,
        "All tasks handled. Exiting task selection mode."
    );

    // Undo the change
    editor.undo();

    // Assertions after undo
    assert_eq!(editor.document.lines[2], "- [ ] The only task");
    assert_eq!(editor.mode, EditorMode::Normal);

    // Redo the change
    editor.redo();

    // Assertions after redo
    assert_eq!(editor.document.lines[2], "# - [ ] The only task");
    assert_eq!(editor.mode, EditorMode::Normal);
}

#[test]
fn test_task_command_fuzzy_search() {
    let mut editor = setup_editor(&[
        "Task list:",
        "- [ ] Apple",
        "- [ ] Banana",
        "Some other line",
        "- [ ] Apricot",
        "- [x] Done Task",
        "- [ ] Avocado",
    ]);
    editor.cursor_y = 0;
    editor.cursor_x = 0;

    // Enter task selection mode
    editor.document.lines.insert(0, "/task".to_string());
    editor.cursor_y = 0;
    editor.cursor_x = 5;
    editor.insert_newline().unwrap();

    assert_eq!(editor.mode, EditorMode::TaskSelection);
    assert_eq!(editor.task.tasks.len(), 4);
    assert_eq!(editor.task.all_tasks.len(), 4);

    // Type "Ap" to search
    editor.handle_task_selection_input(Input::Character('A'));
    editor.handle_task_selection_input(Input::Character('p'));

    assert_eq!(editor.task.fuzzy_search.query, "Ap");
    assert_eq!(editor.task.tasks.len(), 2);
    assert_eq!(editor.task.tasks[0].1, "- [ ] Apple");
    assert_eq!(editor.task.tasks[1].1, "- [ ] Apricot");
    assert_eq!(editor.task.selected_task_index, Some(0));

    // Press backspace
    editor.handle_task_selection_input(Input::KeyBackspace);
    assert_eq!(editor.task.fuzzy_search.query, "A");
    assert_eq!(editor.task.tasks.len(), 3); // "Apple", "Apricot", "Avocado"

    // Sort to have a deterministic order for assertion
    let mut matched_tasks: Vec<String> = editor.task.tasks.iter().map(|(_, s)| s.clone()).collect();
    matched_tasks.sort();
    assert_eq!(matched_tasks[0], "- [ ] Apple");
    assert_eq!(matched_tasks[1], "- [ ] Apricot");
    assert_eq!(matched_tasks[2], "- [ ] Avocado");

    // Clear query
    editor.handle_task_selection_input(Input::KeyBackspace);
    assert_eq!(editor.task.fuzzy_search.query, "");
    assert_eq!(editor.task.tasks.len(), 4);

    // Search for something unique
    editor.handle_task_selection_input(Input::Character('v'));
    assert_eq!(editor.task.fuzzy_search.query, "v");
    assert_eq!(editor.task.tasks.len(), 1);
    assert_eq!(editor.task.tasks[0].1, "- [ ] Avocado");

    // Move the filtered task
    editor.handle_task_selection_input(Input::Character(' '));

    // After moving, the task list should be updated, and the query is still active.
    // Since "Avocado" was the only match, the list is now empty.
    assert_eq!(editor.task.tasks.len(), 0);
    assert_eq!(editor.task.all_tasks.len(), 3); // Avocado removed from all_tasks

    // Clear the query
    editor.handle_task_selection_input(Input::KeyBackspace);
    assert_eq!(editor.task.fuzzy_search.query, "");

    // The list should now show the remaining tasks
    assert_eq!(editor.task.tasks.len(), 3);

    let mut remaining_tasks: Vec<String> =
        editor.task.tasks.iter().map(|(_, s)| s.clone()).collect();
    remaining_tasks.sort();
    assert_eq!(remaining_tasks[0], "- [ ] Apple");
    assert_eq!(remaining_tasks[1], "- [ ] Apricot");
    assert_eq!(remaining_tasks[2], "- [ ] Banana");
}

#[test]
fn test_task_command_fuzzy_search_ctrl_g_exit() {
    let mut editor = setup_editor(&["- [ ] Task A", "- [ ] Task B"]);
    editor.cursor_y = 0;
    editor.cursor_x = 0;

    // Enter task mode
    editor.document.lines.insert(0, "/task".to_string());
    editor.cursor_y = 0;
    editor.cursor_x = 5;
    editor.insert_newline().unwrap();
    assert_eq!(editor.mode, EditorMode::TaskSelection);
    assert_eq!(editor.task.tasks.len(), 2);

    // Press Ctrl+G with empty query, should exit
    editor.handle_task_selection_input(Input::Character('\x07'));
    assert_eq!(editor.mode, EditorMode::Normal);

    // Re-enter task mode
    editor.document.lines.insert(0, "/task".to_string());
    editor.cursor_y = 0;
    editor.cursor_x = 5;
    editor.insert_newline().unwrap();
    assert_eq!(editor.mode, EditorMode::TaskSelection);

    // Type a query
    editor.handle_task_selection_input(Input::Character('A'));
    assert_eq!(editor.task.fuzzy_search.query, "A");
    assert_eq!(editor.task.tasks.len(), 1);

    // Press Ctrl+G with non-empty query, should clear query but not exit
    editor.handle_task_selection_input(Input::Character('\x07'));
    assert_eq!(editor.mode, EditorMode::TaskSelection);
    assert_eq!(editor.task.fuzzy_search.query, "");
    assert_eq!(editor.task.tasks.len(), 2);

    // Press Ctrl+G again with empty query, should exit
    editor.handle_task_selection_input(Input::Character('\x07'));
    assert_eq!(editor.mode, EditorMode::Normal);
}
