use dmacs::editor::Editor;
use pancurses::Input;

fn editor_with_clipboard_disabled() -> Editor {
    let mut editor = Editor::new(None);
    editor._set_clipboard_enabled_for_test(false);
    editor
}

#[test]
fn test_set_marker() {
    let mut editor = Editor::new(None);
    editor.document.lines = vec!["hello world".to_string()];
    editor.set_cursor_pos(0, 0);

    // Set marker at (0,0)
    editor
        .process_input(Input::Character('\x00'), true)
        .unwrap(); // Ctrl-Space
    assert_eq!(editor.selection.marker_pos, Some((0, 0)));
}

#[test]
fn test_clear_marker() {
    let mut editor = Editor::new(None);
    editor.document.lines = vec!["hello world".to_string()];
    editor.set_cursor_pos(0, 0);
    editor.selection.marker_pos = Some((0, 0)); // Manually set marker for testing

    // Clear marker
    editor
        .process_input(Input::Character('\x07'), true)
        .unwrap(); // Ctrl-G
    assert_eq!(editor.selection.marker_pos, None);
}

#[test]
fn test_cut_selection() {
    let mut editor = editor_with_clipboard_disabled();
    editor.document.lines = vec!["hello world".to_string()];
    editor.set_cursor_pos(11, 0); // Cursor at end of "world"
    editor.selection.marker_pos = Some((6, 0)); // Marker at 'w'

    // Cut "world"
    editor
        .process_input(Input::Character('\x17'), true)
        .unwrap(); // Ctrl-W
    assert_eq!(editor.document.lines[0], "hello ");
    assert_eq!(editor.clipboard.kill_buffer, "world");
    assert_eq!(editor.selection.marker_pos, None); // Marker should be cleared
    assert_eq!(editor.cursor_pos(), (6, 0)); // Cursor should be at the start of the cut
}

#[test]
fn test_copy_selection() {
    let mut editor = editor_with_clipboard_disabled();
    editor.document.lines = vec!["hello world".to_string()];
    editor.set_cursor_pos(11, 0); // Cursor at end of "world"
    editor.selection.marker_pos = Some((6, 0)); // Marker at 'w'

    // Copy "world"
    editor.process_input(Input::Character('w'), true).unwrap(); // Option-W
    assert_eq!(editor.document.lines[0], "hello world"); // Document unchanged
    assert_eq!(editor.clipboard.kill_buffer, "world");
    assert_eq!(editor.selection.marker_pos, None); // Marker should be cleared
    assert_eq!(editor.cursor_pos(), (11, 0)); // Cursor should remain
}

#[test]
fn test_highlight_selection() {
    let mut editor = Editor::new(None);
    editor.document.lines = vec!["hello world".to_string()];
    editor.set_cursor_pos(0, 0);
    editor.selection.marker_pos = Some((6, 0)); // Marker at 'w'

    // Check if selection is active
    assert!(editor.selection.is_selection_active());
    assert_eq!(
        editor.selection.get_selection_range(editor.cursor_pos()),
        Some(((0, 0), (6, 0)))
    ); // Cursor to marker
}

#[test]
fn test_cut_selection_from_start_of_line() {
    let mut editor = editor_with_clipboard_disabled();
    editor.document.lines = vec!["hello world".to_string()];
    editor.set_cursor_pos(5, 0); // Cursor at end of "hello"
    editor.selection.marker_pos = Some((0, 0)); // Marker at 'h'

    // Cut "hello"
    editor
        .process_input(Input::Character('\x17'), true)
        .unwrap(); // Ctrl-W
    assert_eq!(editor.document.lines[0], " world");
    assert_eq!(editor.clipboard.kill_buffer, "hello");
    assert_eq!(editor.selection.marker_pos, None);
    assert_eq!(editor.cursor_pos(), (0, 0));
}

#[test]
fn test_cut_selection_to_end_of_line() {
    let mut editor = editor_with_clipboard_disabled();
    editor.document.lines = vec!["hello world".to_string()];
    editor.set_cursor_pos(11, 0); // Cursor at end of "world"
    editor.selection.marker_pos = Some((6, 0)); // Marker at 'w'

    // Cut "world"
    editor
        .process_input(Input::Character('\x17'), true)
        .unwrap(); // Ctrl-W
    assert_eq!(editor.document.lines[0], "hello ");
    assert_eq!(editor.clipboard.kill_buffer, "world");
    assert_eq!(editor.selection.marker_pos, None);
    assert_eq!(editor.cursor_pos(), (6, 0));
}

#[test]
fn test_cut_entire_line() {
    let mut editor = editor_with_clipboard_disabled();
    editor.document.lines = vec!["hello world".to_string()];
    editor.set_cursor_pos(11, 0); // Cursor at end of line
    editor.selection.marker_pos = Some((0, 0)); // Marker at start of line

    // Cut "hello world"
    editor
        .process_input(Input::Character('\x17'), true)
        .unwrap(); // Ctrl-W
    assert_eq!(editor.document.lines, vec!["".to_string()]); // Line should be empty
    assert_eq!(editor.clipboard.kill_buffer, "hello world");
    assert_eq!(editor.selection.marker_pos, None);
    assert_eq!(editor.cursor_pos(), (0, 0));
}

#[test]
fn test_cut_multiple_lines() {
    let mut editor = editor_with_clipboard_disabled();
    editor.document.lines = vec![
        "line one".to_string(),
        "line two".to_string(),
        "line three".to_string(),
    ];
    editor.set_cursor_pos(10, 2); // Cursor at end of "line three"
    editor.selection.marker_pos = Some((0, 0)); // Marker at 'l' in "line one"

    // Cut "line one\nline two\nline three"
    editor
        .process_input(Input::Character('\x17'), true)
        .unwrap(); // Ctrl-W
    assert_eq!(editor.document.lines, vec!["".to_string()]); // All lines should be cut
    assert_eq!(
        editor.clipboard.kill_buffer,
        "line one\nline two\nline three"
    );
    assert_eq!(editor.selection.marker_pos, None);
    assert_eq!(editor.cursor_pos(), (0, 0));
}

#[test]
fn test_cut_selection_marker_after_cursor() {
    let mut editor = editor_with_clipboard_disabled();
    editor.document.lines = vec!["hello world".to_string()];
    editor.set_cursor_pos(6, 0); // Cursor at 'w'
    editor.selection.marker_pos = Some((11, 0)); // Marker at end of "world"

    // Cut "world"
    editor
        .process_input(Input::Character('\x17'), true)
        .unwrap(); // Ctrl-W
    assert_eq!(editor.document.lines[0], "hello ");
    assert_eq!(editor.clipboard.kill_buffer, "world");
    assert_eq!(editor.selection.marker_pos, None);
    assert_eq!(editor.cursor_pos(), (6, 0));
}
