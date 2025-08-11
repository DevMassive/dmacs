use dmacs::editor::Editor;
use pancurses::Input;

#[test]
fn test_debounced_undo_insertion() {
    let mut editor = Editor::new(None);
    editor.set_undo_debounce_threshold(1);

    // Type 'a' - should create a new undo entry
    editor.process_input(Input::Character('a'), false).unwrap();
    assert_eq!(editor.document.lines[0], "a");
    assert_eq!(
        editor.undo_stack.len(),
        1,
        "After 'a', undo stack should have 1 entry"
    );

    // Type 'b' within debounce threshold - should group with 'a'
    editor.process_input(Input::Character('b'), false).unwrap();
    assert_eq!(editor.document.lines[0], "ab");
    assert_eq!(
        editor.undo_stack.len(),
        1,
        "After 'b' (debounced), undo stack should still have 1 entry"
    );

    // Type 'c' within debounce threshold - should group with 'a' and 'b'
    editor.process_input(Input::Character('c'), false).unwrap();
    assert_eq!(editor.document.lines[0], "abc");
    assert_eq!(
        editor.undo_stack.len(),
        1,
        "After 'c' (debounced), undo stack should still have 1 entry"
    );

    // Type 'd' after debounce threshold - should create a new undo entry
    editor.set_undo_debounce_threshold(0);
    editor.process_input(Input::Character('d'), false).unwrap();
    assert_eq!(editor.document.lines[0], "abcd");
    assert_eq!(
        editor.undo_stack.len(),
        2,
        "After 'd' (not debounced), undo stack should have 2 entries"
    );

    // Undo 'd'
    editor.undo();
    assert_eq!(editor.document.lines[0], "abc");
    assert_eq!(editor.undo_stack.len(), 1, "Undo 'd'");

    // Undo 'abc' (grouped)
    editor.undo();
    assert_eq!(editor.document.lines[0], "");
    assert_eq!(editor.undo_stack.len(), 0, "Undo 'abc'");
}

#[test]
fn test_debounced_undo_deletion() {
    let mut editor = Editor::new(None);
    editor.set_undo_debounce_threshold(1);

    editor.process_input(Input::Character('a'), false).unwrap();
    editor.process_input(Input::Character('b'), false).unwrap();
    editor.process_input(Input::Character('c'), false).unwrap();
    editor.set_undo_debounce_threshold(0);
    editor.process_input(Input::Character('d'), false).unwrap();
    assert_eq!(editor.undo_stack.len(), 2);

    // Delete 'd' - should create a new undo entry
    editor.process_input(Input::KeyBackspace, false).unwrap();
    assert_eq!(editor.document.lines[0], "abc");
    assert_eq!(
        editor.undo_stack.len(),
        3,
        "After deleting 'd', undo stack should have 3 entries"
    );

    // Delete 'c' within debounce threshold - should group with 'd' deletion
    editor.set_undo_debounce_threshold(1);
    editor.process_input(Input::KeyBackspace, false).unwrap();
    assert_eq!(editor.document.lines[0], "ab");
    assert_eq!(
        editor.undo_stack.len(),
        3,
        "After deleting 'c' (debounced), undo stack should still have 3 entries"
    );

    // Delete 'b' within debounce threshold - should group with 'd' and 'c' deletions
    editor.set_undo_debounce_threshold(1);
    editor.process_input(Input::KeyBackspace, false).unwrap();
    assert_eq!(editor.document.lines[0], "a");
    assert_eq!(
        editor.undo_stack.len(),
        3,
        "After deleting 'b' (debounced), undo stack should still have 3 entries"
    );

    // Delete 'a' after debounce threshold
    editor.set_undo_debounce_threshold(0);
    editor.process_input(Input::KeyBackspace, false).unwrap();
    assert_eq!(editor.document.lines[0], "");
    assert_eq!(
        editor.undo_stack.len(),
        4,
        "After deleting 'a' (not debounced), undo stack should have 3 entries"
    );

    // Undo 'a'
    editor.undo();
    assert_eq!(editor.document.lines[0], "a");
    assert_eq!(editor.undo_stack.len(), 3, "Undo 'a'");

    // Undo 'bcd' (grouped)
    editor.undo();
    assert_eq!(editor.document.lines[0], "abcd");
    assert_eq!(editor.undo_stack.len(), 2, "Undo 'bcd'");
}

#[test]
fn test_debounced_undo_newline() {
    let mut editor = Editor::new(None);
    editor.set_undo_debounce_threshold(1);

    editor.process_input(Input::Character('a'), false).unwrap();
    editor.set_undo_debounce_threshold(0);

    // Insert first newline
    editor.process_input(Input::Character('\n'), false).unwrap();
    assert_eq!(editor.document.lines.len(), 2);
    assert_eq!(editor.undo_stack.len(), 2);

    // Insert second newline within debounce threshold
    editor.set_undo_debounce_threshold(1);
    editor.process_input(Input::Character('\n'), false).unwrap();
    assert_eq!(editor.document.lines.len(), 3);
    assert_eq!(
        editor.undo_stack.len(),
        2,
        "Second newline should be debounced"
    );

    // Insert third newline after debounce threshold
    editor.set_undo_debounce_threshold(0);
    editor.process_input(Input::Character('\n'), false).unwrap();
    assert_eq!(editor.document.lines.len(), 4);
    assert_eq!(
        editor.undo_stack.len(),
        3,
        "Third newline should not be debounced"
    );

    // Undo third newline
    editor.undo();
    assert_eq!(editor.document.lines.len(), 3);
    assert_eq!(editor.undo_stack.len(), 2);

    // Undo first and second newlines (grouped)
    editor.undo();
    assert_eq!(editor.document.lines.len(), 1);
    assert_eq!(editor.undo_stack.len(), 1);
}

#[test]
fn test_debounced_undo_mixed_actions() {
    let mut editor = Editor::new(None);
    editor.set_undo_debounce_threshold(1);

    assert_eq!(editor.undo_stack.len(), 0);

    // Type 'a' (insertion)
    editor.process_input(Input::Character('a'), false).unwrap();
    assert_eq!(editor.undo_stack.len(), 1);

    // Type 'b' (insertion) - debounced
    editor.process_input(Input::Character('b'), false).unwrap();
    assert_eq!(editor.undo_stack.len(), 1);

    // Newline (different action type) - not debounced
    editor.process_input(Input::Character('\n'), false).unwrap();
    assert_eq!(editor.undo_stack.len(), 2);

    // Type 'c' (insertion) - new group
    editor.process_input(Input::Character('c'), false).unwrap();
    assert_eq!(editor.undo_stack.len(), 3);

    // Delete (different action type) - not debounced
    editor.process_input(Input::KeyBackspace, false).unwrap();
    assert_eq!(editor.undo_stack.len(), 4);

    // Undo sequence
    editor.undo(); // Undo deletion
    assert_eq!(editor.document.lines[0], "ab");
    assert_eq!(editor.document.lines[1], "c");
    assert_eq!(editor.undo_stack.len(), 3);
}

#[test]
fn test_initial_state_undo() {
    let mut editor = Editor::new(None);
    editor.set_undo_debounce_threshold(1);

    assert_eq!(
        editor.undo_stack.len(),
        0,
        "Initial state should not be saved"
    );
    assert_eq!(editor.document.lines[0], "");

    editor.undo();
    assert_eq!(
        editor.undo_stack.len(),
        0,
        "Undo stack should be empty after trying to undo empty state"
    );
    assert_eq!(editor.document.lines[0], ""); // Document should remain empty
    assert_eq!(editor.status_message, "Nothing to undo.");

    editor.undo(); // Try to undo again
    assert_eq!(editor.undo_stack.len(), 0);
    assert_eq!(editor.status_message, "Nothing to undo.");
}

#[test]
fn test_redo() {
    let mut editor = Editor::new(None);
    editor.set_undo_debounce_threshold(1);

    // Perform some actions
    editor.process_input(Input::Character('a'), false).unwrap();
    editor.process_input(Input::Character('b'), false).unwrap();
    editor.process_input(Input::Character('c'), false).unwrap();
    editor.set_undo_debounce_threshold(0); // Ensure 'd' is a separate undo entry
    editor.process_input(Input::Character('d'), false).unwrap();
    editor.process_input(Input::Character('\n'), false).unwrap();
    editor.process_input(Input::Character('e'), false).unwrap();

    assert_eq!(editor.document.lines[0], "abcd");
    assert_eq!(editor.document.lines[1], "e");
    assert_eq!(editor.undo_stack.len(), 4); // 'abc', 'd', newline, 'e'
    assert_eq!(editor.redo_stack.len(), 0);
    assert_eq!(editor.cursor_y, 1);
    assert_eq!(editor.cursor_x, 1);

    // Undo 'e'
    editor.undo();
    assert_eq!(editor.document.lines[0], "abcd");
    assert_eq!(editor.document.lines.len(), 2); // Document should have 2 lines after undoing 'e'
    assert_eq!(editor.undo_stack.len(), 3);
    assert_eq!(editor.redo_stack.len(), 1); // 'e' should be in redo stack
    assert_eq!(editor.cursor_y, 1);
    assert_eq!(editor.cursor_x, 0);

    // Undo newline
    editor.undo();
    assert_eq!(editor.document.lines[0], "abcd");
    assert_eq!(editor.document.lines.len(), 1);
    assert_eq!(editor.undo_stack.len(), 2);
    assert_eq!(editor.redo_stack.len(), 2); // newline should be in redo stack
    assert_eq!(editor.cursor_y, 0);
    assert_eq!(editor.cursor_x, 4);

    // Undo 'd'
    editor.undo();
    assert_eq!(editor.document.lines[0], "abc");
    assert_eq!(editor.undo_stack.len(), 1);
    assert_eq!(editor.redo_stack.len(), 3); // 'd' should be in redo stack
    assert_eq!(editor.cursor_y, 0);
    assert_eq!(editor.cursor_x, 3);

    // Redo 'd'
    editor.redo();
    assert_eq!(editor.document.lines[0], "abcd");
    assert_eq!(editor.undo_stack.len(), 2);
    assert_eq!(editor.redo_stack.len(), 2); // newline should be in redo stack
    assert_eq!(editor.cursor_y, 0);
    assert_eq!(editor.cursor_x, 4);

    // Redo newline
    editor.redo();
    assert_eq!(editor.document.lines[0], "abcd");
    assert_eq!(editor.document.lines.len(), 2);
    assert_eq!(editor.undo_stack.len(), 3);
    assert_eq!(editor.redo_stack.len(), 1); // 'e' should be in redo stack
    assert_eq!(editor.cursor_y, 1);
    assert_eq!(editor.cursor_x, 0);

    // Redo 'e'
    editor.redo();
    assert_eq!(editor.document.lines[0], "abcd");
    assert_eq!(editor.document.lines[1], "e");
    assert_eq!(editor.undo_stack.len(), 4);
    assert_eq!(editor.redo_stack.len(), 0);
    assert_eq!(editor.cursor_y, 1);
    assert_eq!(editor.cursor_x, 1);

    // Try to redo when redo stack is empty
    editor.redo();
    assert_eq!(editor.status_message, "Nothing to redo.");
    assert_eq!(editor.undo_stack.len(), 4);
    assert_eq!(editor.redo_stack.len(), 0);
    assert_eq!(editor.cursor_y, 1);
    assert_eq!(editor.cursor_x, 1);

    // Perform a new action after undoing, then try to redo (should not work)
    editor.undo(); // Undo 'e'
    assert_eq!(editor.document.lines[0], "abcd");
    assert_eq!(editor.document.lines.len(), 2);
    assert_eq!(editor.undo_stack.len(), 3);
    assert_eq!(editor.redo_stack.len(), 1);
    assert_eq!(editor.cursor_y, 1);
    assert_eq!(editor.cursor_x, 0);

    editor.process_input(Input::Character('f'), false).unwrap(); // New action
    assert_eq!(editor.document.lines[0], "abcd");
    assert_eq!(editor.document.lines[1], "f");
    assert_eq!(editor.undo_stack.len(), 4);
    assert_eq!(editor.redo_stack.len(), 0); // Redo stack should be cleared
    assert_eq!(editor.cursor_y, 1);
    assert_eq!(editor.cursor_x, 1);

    editor.redo(); // Should not redo 'e'
    assert_eq!(editor.status_message, "Nothing to redo.");
    assert_eq!(editor.document.lines[0], "abcd");
    assert_eq!(editor.document.lines[1], "f");
    assert_eq!(editor.cursor_y, 1);
    assert_eq!(editor.cursor_x, 1);
}

#[test]
fn test_redo_simple() {
    let mut editor = Editor::new(None);
    editor.set_undo_debounce_threshold(1);

    // Perform some actions
    editor.process_input(Input::Character('a'), false).unwrap();
    editor.process_input(Input::Character('b'), false).unwrap();
    editor.process_input(Input::Character('c'), false).unwrap();

    assert_eq!(editor.document.lines[0], "abc");
    assert_eq!(editor.undo_stack.len(), 1); // 'abc'
    assert_eq!(editor.redo_stack.len(), 0);
    assert_eq!(editor.cursor_y, 0);
    assert_eq!(editor.cursor_x, 3);

    // Undo 'abc'
    editor.undo();
    assert_eq!(editor.document.lines[0], "");
    assert_eq!(editor.undo_stack.len(), 0);
    assert_eq!(editor.redo_stack.len(), 1); // 'abc' should be in redo stack
    assert_eq!(editor.cursor_y, 0);
    assert_eq!(editor.cursor_x, 0);

    // Redo 'abc'
    editor.redo();
    assert_eq!(editor.document.lines[0], "abc");
    assert_eq!(editor.undo_stack.len(), 1);
    assert_eq!(editor.redo_stack.len(), 0);
    assert_eq!(editor.cursor_y, 0);
    assert_eq!(editor.cursor_x, 3);
}

#[test]
fn test_undo_redo_kill_line() {
    let mut editor = Editor::new(None);
    editor.set_undo_debounce_threshold(0); // Disable debouncing for clear test cases

    editor.insert_text("Hello World").unwrap();
    assert_eq!(editor.cursor_x, 11);
    assert_eq!(editor.cursor_y, 0);

    editor.insert_newline().unwrap();
    assert_eq!(editor.cursor_x, 0);
    assert_eq!(editor.cursor_y, 1);

    editor.insert_text("Another Line").unwrap();
    assert_eq!(editor.cursor_x, 12);
    assert_eq!(editor.cursor_y, 1);

    editor.go_to_start_of_file();
    assert_eq!(editor.cursor_x, 0);
    assert_eq!(editor.cursor_y, 0);

    editor.move_cursor_down(); // Move to "Another Line"
    assert_eq!(editor.cursor_x, 0);
    assert_eq!(editor.cursor_y, 1);

    // Kill "Another Line"
    editor.kill_line().unwrap();
    assert_eq!(editor.document.lines.len(), 2);
    assert_eq!(editor.document.lines[0], "Hello World");
    assert_eq!(editor.document.lines[1], "");
    assert_eq!(editor.undo_stack.len(), 4); // Insertion, Newline, Insertion, Kill Line
    assert_eq!(editor.redo_stack.len(), 0);
    assert_eq!(editor.cursor_x, 0);
    assert_eq!(editor.cursor_y, 1);

    // Undo kill_line
    editor.undo();
    assert_eq!(editor.document.lines.len(), 2);
    assert_eq!(editor.document.lines[0], "Hello World");
    assert_eq!(editor.document.lines[1], "Another Line");
    assert_eq!(editor.undo_stack.len(), 3);
    assert_eq!(editor.redo_stack.len(), 1);
    assert_eq!(editor.cursor_x, 12);
    assert_eq!(editor.cursor_y, 1);

    // Redo kill_line
    editor.redo();
    assert_eq!(editor.document.lines.len(), 2);
    assert_eq!(editor.document.lines[0], "Hello World");
    assert_eq!(editor.document.lines[1], "");
    assert_eq!(editor.undo_stack.len(), 4);
    assert_eq!(editor.redo_stack.len(), 0);
    assert_eq!(editor.cursor_x, 0);
    assert_eq!(editor.cursor_y, 1);
}

#[test]
fn test_undo_redo_yank() {
    let mut editor = Editor::new(None);
    editor.set_undo_debounce_threshold(0);

    editor.insert_text("Yank Me").unwrap();
    assert_eq!(editor.cursor_x, 7);
    assert_eq!(editor.cursor_y, 0);

    editor.go_to_start_of_file();
    assert_eq!(editor.cursor_x, 0);
    assert_eq!(editor.cursor_y, 0);

    editor.set_marker_action();
    assert_eq!(editor.cursor_x, 0);
    assert_eq!(editor.cursor_y, 0);

    editor.go_to_end_of_line();
    assert_eq!(editor.cursor_x, 7);
    assert_eq!(editor.cursor_y, 0);

    editor.copy_selection_action().unwrap(); // Copy "Yank Me" to kill buffer
    assert_eq!(editor.cursor_x, 7);
    assert_eq!(editor.cursor_y, 0);

    editor.delete_char().unwrap(); // Delete "e" to make a change
    assert_eq!(editor.cursor_x, 6);
    assert_eq!(editor.cursor_y, 0);

    editor.delete_char().unwrap(); // Delete "M"
    assert_eq!(editor.cursor_x, 5);
    assert_eq!(editor.cursor_y, 0);

    editor.delete_char().unwrap(); // Delete " "
    assert_eq!(editor.cursor_x, 4);
    assert_eq!(editor.cursor_y, 0);

    editor.yank().unwrap(); // Yank "Yank Me"
    assert_eq!(editor.document.lines[0], "YankYank Me");
    assert_eq!(editor.undo_stack.len(), 5); // Insertion, Del, Del, Del, Yank
    assert_eq!(editor.redo_stack.len(), 0);
    assert_eq!(editor.cursor_x, 11);
    assert_eq!(editor.cursor_y, 0);

    // Undo yank
    editor.undo();
    assert_eq!(editor.document.lines[0], "Yank");
    assert_eq!(editor.undo_stack.len(), 4);
    assert_eq!(editor.redo_stack.len(), 1);
    assert_eq!(editor.cursor_x, 4);
    assert_eq!(editor.cursor_y, 0);

    // Redo yank
    editor.redo();
    assert_eq!(editor.document.lines[0], "YankYank Me");
    assert_eq!(editor.undo_stack.len(), 5);
    assert_eq!(editor.redo_stack.len(), 0);
    assert_eq!(editor.cursor_x, 11);
    assert_eq!(editor.cursor_y, 0);
}

#[test]
fn test_undo_redo_cut_selection() {
    let mut editor = Editor::new(None);
    editor.set_undo_debounce_threshold(0);

    editor.insert_text("Line One").unwrap();
    assert_eq!(editor.cursor_x, 8);
    assert_eq!(editor.cursor_y, 0);

    editor.insert_newline().unwrap();
    assert_eq!(editor.cursor_x, 0);
    assert_eq!(editor.cursor_y, 1);

    editor.insert_text("Line Two").unwrap();
    assert_eq!(editor.cursor_x, 8);
    assert_eq!(editor.cursor_y, 1);

    editor.insert_newline().unwrap();
    assert_eq!(editor.cursor_x, 0);
    assert_eq!(editor.cursor_y, 2);

    editor.insert_text("Line Three").unwrap();
    assert_eq!(editor.cursor_x, 10);
    assert_eq!(editor.cursor_y, 2);

    editor.go_to_start_of_file();
    assert_eq!(editor.cursor_x, 0);
    assert_eq!(editor.cursor_y, 0);

    editor.move_cursor_down(); // Move to "Line Two"
    assert_eq!(editor.cursor_x, 0);
    assert_eq!(editor.cursor_y, 1);

    editor.set_marker_action();
    assert_eq!(editor.cursor_x, 0);
    assert_eq!(editor.cursor_y, 1);

    editor.go_to_end_of_line();
    assert_eq!(editor.cursor_x, 8);
    assert_eq!(editor.cursor_y, 1);

    editor.move_cursor_down(); // Move to "Line Three"
    assert_eq!(editor.cursor_x, 8);
    assert_eq!(editor.cursor_y, 2);

    editor.move_cursor_right(); // Select "Line Thre"
    assert_eq!(editor.cursor_x, 9);
    assert_eq!(editor.cursor_y, 2);

    // Cut "Line Two\nLine Thre"
    editor.cut_selection_action().unwrap();
    assert_eq!(editor.document.lines.len(), 2);
    assert_eq!(editor.document.lines[0], "Line One");
    assert_eq!(editor.document.lines[1], "e");
    assert_eq!(editor.undo_stack.len(), 6); // Insertion, Newline, Insertion, Newline, Insertion, Cut
    assert_eq!(editor.redo_stack.len(), 0);
    assert_eq!(editor.cursor_x, 0);
    assert_eq!(editor.cursor_y, 1);

    // Undo cut
    editor.undo();
    assert_eq!(editor.document.lines.len(), 3);
    assert_eq!(editor.document.lines[0], "Line One");
    assert_eq!(editor.document.lines[1], "Line Two");
    assert_eq!(editor.document.lines[2], "Line Three");
    assert_eq!(editor.undo_stack.len(), 5);
    assert_eq!(editor.redo_stack.len(), 1);
    assert_eq!(editor.cursor_x, 9);
    assert_eq!(editor.cursor_y, 2);

    // Redo cut
    editor.redo();
    assert_eq!(editor.document.lines.len(), 2);
    assert_eq!(editor.document.lines[0], "Line One");
    assert_eq!(editor.document.lines[1], "e");
    assert_eq!(editor.undo_stack.len(), 6);
    assert_eq!(editor.redo_stack.len(), 0);
    assert_eq!(editor.cursor_x, 0);
    assert_eq!(editor.cursor_y, 1);
}
