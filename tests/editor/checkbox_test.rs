use dmacs::editor::Editor;
use pancurses::Input;

fn simulate_ctrl_t(editor: &mut Editor) {
    editor
        .process_input(Input::Character('\x14'), false)
        .unwrap();
}

#[test]
fn test_toggle_checkbox_add() {
    let mut editor = Editor::new(None, None, None);
    editor.insert_text("Hello world").unwrap();
    editor.go_to_start_of_line();
    simulate_ctrl_t(&mut editor);
    assert_eq!(editor.document.lines[0], "- Hello world");
    assert_eq!(editor.cursor_pos(), (2, 0));
    assert_eq!(editor.status_message, "Toggled to ListItem.");
}

#[test]
fn test_toggle_checkbox_check() {
    let mut editor = Editor::new(None, None, None);
    editor.insert_text("- [ ] Hello world").unwrap();
    editor.go_to_start_of_line();
    simulate_ctrl_t(&mut editor);
    assert_eq!(editor.document.lines[0], "- [x] Hello world");
    assert_eq!(editor.cursor_pos(), (0, 0));
    assert_eq!(editor.status_message, "Toggled to Checked.");
}

#[test]
fn test_toggle_checkbox_uncheck() {
    let mut editor = Editor::new(None, None, None);
    editor.insert_text("- [x] Hello world").unwrap();
    editor.go_to_start_of_line();
    simulate_ctrl_t(&mut editor);
    assert_eq!(editor.document.lines[0], "Hello world");
    assert_eq!(editor.cursor_pos(), (0, 0));
    assert_eq!(editor.status_message, "Toggled to Plain.");
}

#[test]
fn test_toggle_checkbox_undo_redo() {
    let mut editor = Editor::new(None, None, None);
    editor.insert_text("Hello world").unwrap();
    editor.go_to_start_of_line();
    let initial_pos = editor.cursor_pos();

    // Add list item
    simulate_ctrl_t(&mut editor);
    assert_eq!(editor.document.lines[0], "- Hello world");
    let after_list_item_pos = editor.cursor_pos();

    // Add checkbox
    simulate_ctrl_t(&mut editor);
    assert_eq!(editor.document.lines[0], "- [ ] Hello world");
    let after_add_pos = editor.cursor_pos();

    // Check it
    simulate_ctrl_t(&mut editor);
    assert_eq!(editor.document.lines[0], "- [x] Hello world");
    let after_check_pos = editor.cursor_pos();

    // Uncheck it
    simulate_ctrl_t(&mut editor);
    assert_eq!(editor.document.lines[0], "Hello world");
    let after_uncheck_pos = editor.cursor_pos();

    // Undo uncheck
    editor.undo();
    assert_eq!(editor.document.lines[0], "- [x] Hello world");
    assert_eq!(editor.cursor_pos(), after_check_pos);

    // Undo check
    editor.undo();
    assert_eq!(editor.document.lines[0], "- [ ] Hello world");
    assert_eq!(editor.cursor_pos(), after_add_pos);

    // Undo add checkbox
    editor.undo();
    assert_eq!(editor.document.lines[0], "- Hello world");
    assert_eq!(editor.cursor_pos(), after_list_item_pos);

    // Undo add list item
    editor.undo();
    assert_eq!(editor.document.lines[0], "Hello world");
    assert_eq!(editor.cursor_pos(), initial_pos);

    // Redo add list item
    editor.redo();
    assert_eq!(editor.document.lines[0], "- Hello world");
    assert_eq!(editor.cursor_pos(), after_list_item_pos);

    // Redo add checkbox
    editor.redo();
    assert_eq!(editor.document.lines[0], "- [ ] Hello world");
    assert_eq!(editor.cursor_pos(), after_add_pos);

    // Redo check
    editor.redo();
    assert_eq!(editor.document.lines[0], "- [x] Hello world");
    assert_eq!(editor.cursor_pos(), after_check_pos);

    // Redo uncheck
    editor.redo();
    assert_eq!(editor.document.lines[0], "Hello world");
    assert_eq!(editor.cursor_pos(), after_uncheck_pos);
}

#[test]
fn test_toggle_indented_checkbox_add() {
    let mut editor = Editor::new(None, None, None);
    editor.insert_text("  Hello world").unwrap();
    editor.go_to_start_of_line();
    simulate_ctrl_t(&mut editor);
    assert_eq!(editor.document.lines[0], "  - Hello world");
    assert_eq!(editor.cursor_pos(), (4, 0));
}

#[test]
fn test_toggle_indented_checkbox_check() {
    let mut editor = Editor::new(None, None, None);
    editor.insert_text("  - [ ] Hello world").unwrap();
    editor.go_to_start_of_line();
    simulate_ctrl_t(&mut editor);
    assert_eq!(editor.document.lines[0], "  - [x] Hello world");
    assert_eq!(editor.cursor_pos(), (0, 0));
}

#[test]
fn test_toggle_indented_checkbox_uncheck() {
    let mut editor = Editor::new(None, None, None);
    editor.insert_text("  - [x] Hello world").unwrap();
    editor.go_to_start_of_line();
    simulate_ctrl_t(&mut editor);
    assert_eq!(editor.document.lines[0], "  Hello world");
    assert_eq!(editor.cursor_pos(), (0, 0));
}

#[test]
fn test_toggle_indented_checkbox_add_cursor_middle() {
    let mut editor = Editor::new(None, None, None);
    editor.insert_text("  Hello world").unwrap();
    editor.set_cursor_pos(4, 0); // "  He|llo world"
    simulate_ctrl_t(&mut editor);
    assert_eq!(editor.document.lines[0], "  - Hello world");
    assert_eq!(editor.cursor_pos(), (6, 0)); // "  - He|llo world"
}

#[test]
fn test_toggle_checkbox_selection_mixed_to_list() {
    let mut editor = Editor::new(None, None, None);
    editor.document.lines = vec![
        "Plain text".to_string(),
        "- List item".to_string(),
        "- [ ] Unchecked".to_string(),
        "- [x] Checked".to_string(),
        "Another plain".to_string(),
    ];
    editor.set_cursor_pos(0, 0);
    editor.set_marker_action();
    editor.set_cursor_pos(5, 4); // Select all lines

    simulate_ctrl_t(&mut editor);

    assert_eq!(editor.document.lines[0], "- Plain text");
    assert_eq!(editor.document.lines[1], "- List item");
    assert_eq!(editor.document.lines[2], "- Unchecked");
    assert_eq!(editor.document.lines[3], "- Checked");
    assert_eq!(editor.document.lines[4], "- Another plain");
    assert_eq!(editor.status_message, "Toggled selection to ListItem.");

    // Test that selection is not cleared
    assert!(editor.selection.is_selection_active());

    editor.undo();
    assert_eq!(editor.document.lines[0], "Plain text");
    assert_eq!(editor.document.lines[1], "- List item");
    assert_eq!(editor.document.lines[2], "- [ ] Unchecked");
    assert_eq!(editor.document.lines[3], "- [x] Checked");
}

#[test]
fn test_toggle_checkbox_selection_ignores_empty_lines() {
    let mut editor = Editor::new(None, None, None);
    editor.document.lines = vec!["Line 1".to_string(), "".to_string(), "Line 3".to_string()];
    editor.set_cursor_pos(0, 0);
    editor.set_marker_action();
    editor.set_cursor_pos(6, 2);

    simulate_ctrl_t(&mut editor);

    assert_eq!(editor.document.lines[0], "- Line 1");
    assert_eq!(editor.document.lines[1], "");
    assert_eq!(editor.document.lines[2], "- Line 3");
}

#[test]
fn test_toggle_checkbox_selection_excludes_last_line_if_cursor_x_is_zero() {
    let mut editor = Editor::new(None, None, None);
    editor.document.lines = vec!["Line 1".to_string(), "Line 2".to_string()];
    editor.set_cursor_pos(1, 0); // Mark start of selection
    editor.set_marker_action();
    editor.set_cursor_pos(0, 1); // Move cursor to x=0 on last line

    simulate_ctrl_t(&mut editor);

    // Only line 1 should be changed
    assert_eq!(editor.document.lines[0], "- Line 1");
    assert_eq!(editor.document.lines[1], "Line 2");
}
