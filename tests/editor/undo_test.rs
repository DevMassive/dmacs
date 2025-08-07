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
