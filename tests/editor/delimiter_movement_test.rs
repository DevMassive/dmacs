use dmacs::editor::Editor;

fn setup_editor_with_content(content: Vec<&str>) -> Editor {
    let mut editor = Editor::new(None);
    editor.document.lines = content.iter().map(|&s| s.to_string()).collect();
    editor.screen_rows = 20; // Set a reasonable screen size for testing
    editor.screen_cols = 80;
    editor
}

#[test]
fn test_move_to_next_delimiter_no_delimiters() {
    let mut editor = setup_editor_with_content(vec!["line 1", "line 2", "line 3"]);
    editor.cursor_y = 1; // Start in the middle

    editor.move_to_next_delimiter();
    assert_eq!(
        editor.cursor_y, 1,
        "Cursor should remain in original position if no delimiters"
    );
    assert_eq!(editor.cursor_x, 0);
}

#[test]
fn test_move_to_next_delimiter_after_current_position() {
    let mut editor = setup_editor_with_content(vec!["line 1", "---", "line 3", "---", "line 5"]);
    editor.cursor_y = 0; // Start at the beginning

    editor.move_to_next_delimiter();
    assert_eq!(
        editor.cursor_y, 2,
        "Cursor should move to line after first delimiter"
    );
    assert_eq!(editor.cursor_x, 0);

    editor.move_to_next_delimiter();
    assert_eq!(
        editor.cursor_y, 4,
        "Cursor should move to line after second delimiter"
    );
    assert_eq!(editor.cursor_x, 0);
}

#[test]
fn test_move_to_next_delimiter_from_delimiter_line() {
    let mut editor = setup_editor_with_content(vec!["line 1", "---", "line 3", "line 4"]);
    editor.cursor_y = 1; // Start on the delimiter

    editor.move_to_next_delimiter();
    assert_eq!(
        editor.cursor_y, 2,
        "Cursor should move to line after the current delimiter"
    );
    assert_eq!(editor.cursor_x, 0);
}

#[test]
fn test_move_to_next_delimiter_multiple_delimiters() {
    let mut editor =
        setup_editor_with_content(vec!["---", "line 1", "---", "line 2", "---", "line 3"]);
    editor.cursor_y = 0;

    editor.move_to_next_delimiter();
    assert_eq!(editor.cursor_y, 1);

    editor.move_to_next_delimiter();
    assert_eq!(editor.cursor_y, 3);

    editor.move_to_next_delimiter();
    assert_eq!(editor.cursor_y, 5);

    // No more delimiters, cursor should not move
    editor.move_to_next_delimiter();
    assert_eq!(editor.cursor_y, 5);
}

#[test]
fn test_move_to_next_delimiter_at_end_of_file() {
    let mut editor = setup_editor_with_content(vec!["line 1", "line 2", "---"]);
    editor.cursor_y = 0;

    editor.move_to_next_delimiter();
    assert_eq!(
        editor.cursor_y, 0,
        "Cursor should wrap around to beginning if delimiter is last line"
    );
    assert_eq!(editor.cursor_x, 0);
}

#[test]
fn test_move_to_next_delimiter_empty_document() {
    let mut editor = setup_editor_with_content(vec![]);
    editor.cursor_y = 0;

    editor.move_to_next_delimiter();
    assert_eq!(
        editor.cursor_y, 0,
        "Cursor should remain at 0,0 in empty document"
    );
    assert_eq!(editor.cursor_x, 0);
}

#[test]
fn test_move_to_next_delimiter_no_further_delimiters() {
    let mut editor = setup_editor_with_content(vec!["line 1", "line 2", "---", "line 4"]);
    editor.cursor_y = 3; // Start after the last delimiter

    editor.move_to_next_delimiter();
    assert_eq!(
        editor.cursor_y, 3,
        "Cursor should not move if no further delimiters"
    );
    assert_eq!(editor.cursor_x, 0);
}
