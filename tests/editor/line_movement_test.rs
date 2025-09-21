use dmacs::editor::Editor;
use pancurses::Input;

#[test]
fn test_editor_move_line_up() {
    let mut editor = Editor::new(None, None, None);
    editor.document.lines = vec![
        "line1".to_string(),
        "line2".to_string(),
        "line3".to_string(),
    ];
    editor.set_cursor_pos(0, 1); // Cursor on line2
    editor.process_input(Input::KeyUp, true).unwrap();
    assert_eq!(editor.document.lines[0], "line2");
    assert_eq!(editor.document.lines[1], "line1");
    assert_eq!(editor.document.lines[2], "line3");
    assert_eq!(editor.cursor_pos(), (0, 0)); // Cursor should move up with the line

    // Try moving up from the first line (should not change document, only status message)
    editor.process_input(Input::KeyUp, true).unwrap();
    assert_eq!(editor.document.lines[0], "line2");
    assert_eq!(editor.document.lines[1], "line1");
    assert_eq!(editor.document.lines[2], "line3");
    assert_eq!(editor.cursor_pos(), (0, 0));
}

#[test]
fn test_editor_move_line_down() {
    let mut editor = Editor::new(None, None, None);
    editor.document.lines = vec![
        "line1".to_string(),
        "line2".to_string(),
        "line3".to_string(),
    ];
    editor.set_cursor_pos(0, 1); // Cursor on line2
    editor.process_input(Input::KeyDown, true).unwrap();
    assert_eq!(editor.document.lines[0], "line1");
    assert_eq!(editor.document.lines[1], "line3");
    assert_eq!(editor.document.lines[2], "line2");
    assert_eq!(editor.cursor_pos(), (0, 2)); // Cursor should move down with the line

    // Try moving down from the last line (should not change document, only status message)
    editor.process_input(Input::KeyDown, true).unwrap();
    assert_eq!(editor.document.lines[0], "line1");
    assert_eq!(editor.document.lines[1], "line3");
    assert_eq!(editor.document.lines[2], "line2");
    assert_eq!(editor.cursor_pos(), (0, 2));
}
