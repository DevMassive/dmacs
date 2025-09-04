use dmacs::editor::Editor;
use pancurses::Input;

fn editor_with_clipboard_disabled() -> Editor {
    let mut editor = Editor::new(None);
    editor._set_clipboard_enabled_for_test(false);
    editor
}

#[test]
fn test_editor_kill_line_middle_of_line() {
    let mut editor = editor_with_clipboard_disabled();
    editor.document.lines = vec!["hello world".to_string()];
    editor.set_cursor_pos(6, 0); // Cursor at 'w' in "world"
    editor
        .process_input(Input::Character('\x0b'), false)
        .unwrap();
    assert_eq!(editor.document.lines[0], "hello ");
    assert_eq!(editor.kill_buffer, "world");
    assert_eq!(editor.cursor_pos(), (6, 0));
}

#[test]
fn test_editor_kill_line_end_of_line_not_last_line() {
    let mut editor = editor_with_clipboard_disabled();
    editor.document.lines = vec!["hello".to_string(), "world".to_string()];
    editor.set_cursor_pos(5, 0); // Cursor at end of "hello"
    editor
        .process_input(Input::Character('\x0b'), false)
        .unwrap();
    assert_eq!(editor.document.lines.len(), 1);
    assert_eq!(editor.document.lines[0], "helloworld");
    assert_eq!(editor.kill_buffer, "\n"); // Newline
    assert_eq!(editor.cursor_pos(), (5, 0));
}

#[test]
fn test_editor_kill_line_empty_line_not_last_line() {
    let mut editor = editor_with_clipboard_disabled();
    editor.document.lines = vec!["line1".to_string(), "".to_string(), "line3".to_string()];
    editor.set_cursor_pos(0, 1); // Cursor at beginning of empty line
    editor
        .process_input(Input::Character('\x0b'), false)
        .unwrap();
    assert_eq!(editor.document.lines.len(), 2);
    assert_eq!(editor.document.lines[0], "line1");
    assert_eq!(editor.document.lines[1], "line3");
    assert_eq!(editor.kill_buffer, "\n"); // Only newline killed
    assert_eq!(editor.cursor_pos(), (0, 1));
}

#[test]
fn test_editor_kill_line_last_line() {
    let mut editor = editor_with_clipboard_disabled();
    editor.document.lines = vec!["last line".to_string()];
    editor.set_cursor_pos(0, 0);
    editor
        .process_input(Input::Character('\x0b'), false)
        .unwrap();
    assert_eq!(editor.document.lines[0], "");
    assert_eq!(editor.kill_buffer, "last line");
    assert_eq!(editor.cursor_pos(), (0, 0));
}

#[test]
fn test_editor_yank_single_line() {
    let mut editor = editor_with_clipboard_disabled();
    editor.kill_buffer = "yanked text".to_string();
    editor.document.lines = vec!["start ".to_string(), "end".to_string()];
    editor.set_cursor_pos(6, 0); // After "start "
    editor
        .process_input(Input::Character('\x19'), false)
        .unwrap();
    assert_eq!(editor.document.lines[0], "start yanked text");
    assert_eq!(editor.cursor_pos(), (17, 0)); // Cursor after yanked text
}

#[test]
fn test_editor_yank_multiple_lines() {
    let mut editor = editor_with_clipboard_disabled();
    editor.kill_buffer = "line1\nline2\nline3".to_string();
    editor.document.lines = vec!["start".to_string(), "end".to_string()];
    editor.set_cursor_pos(5, 0); // After "start"
    editor
        .process_input(Input::Character('\x19'), false)
        .unwrap();
    assert_eq!(editor.document.lines.len(), 4);
    assert_eq!(editor.document.lines[0], "startline1");
    assert_eq!(editor.document.lines[1], "line2");
    assert_eq!(editor.document.lines[2], "line3");
    assert_eq!(editor.document.lines[3], "end");
    assert_eq!(editor.cursor_pos(), (5, 2)); // Cursor at end of last yanked line
}

#[test]
fn test_editor_consecutive_kill_line() {
    let mut editor = editor_with_clipboard_disabled();
    editor.document.lines = vec![
        "line one".to_string(),
        "line two".to_string(),
        "line three".to_string(),
    ];

    // Kill "line one"
    editor.set_cursor_pos(0, 0);
    editor
        .process_input(Input::Character('\x0b'), false)
        .unwrap(); // Ctrl-K
    assert_eq!(editor.kill_buffer, "line one");
    assert_eq!(editor.document.lines.len(), 3);
    assert_eq!(editor.document.lines[0], ""); // "line one" should be removed

    editor.set_cursor_pos(0, 0);
    editor
        .process_input(Input::Character('\x0b'), false)
        .unwrap(); // Ctrl-K
    assert_eq!(editor.kill_buffer, "line one\n");
    assert_eq!(editor.document.lines.len(), 2);
    assert_eq!(editor.document.lines[0], "line two"); // "line one\n" should be removed

    // Kill "line two" immediately after
    editor.set_cursor_pos(0, 0); // Cursor is now at the start of "line two"
    editor
        .process_input(Input::Character('\x0b'), false)
        .unwrap(); // Ctrl-K
    assert_eq!(editor.kill_buffer, "line one\nline two"); // Should append
    assert_eq!(editor.document.lines.len(), 2);
    assert_eq!(editor.document.lines[0], ""); // "line two" should be removed

    editor.set_cursor_pos(0, 0);
    editor
        .process_input(Input::Character('\x0b'), false)
        .unwrap(); // Ctrl-K
    assert_eq!(editor.kill_buffer, "line one\nline two\n"); // Should append
    assert_eq!(editor.document.lines.len(), 1);
    assert_eq!(editor.document.lines[0], "line three"); // "line two" should be removed

    // Yank the accumulated content
    editor.set_cursor_pos(0, 0);
    editor
        .process_input(Input::Character('\x19'), false)
        .unwrap(); // Ctrl-Y
    assert_eq!(editor.document.lines.len(), 3);
    assert_eq!(editor.document.lines[0], "line one");
    assert_eq!(editor.document.lines[1], "line two");
    assert_eq!(editor.document.lines[2], "line three");
}

#[test]
fn test_editor_yank_empty_kill_buffer() {
    let mut editor = editor_with_clipboard_disabled();
    editor.kill_buffer = "".to_string();
    editor.document.lines = vec!["original".to_string()];
    editor.set_cursor_pos(0, 0);
    editor
        .process_input(Input::Character('\x19'), false)
        .unwrap();
    assert_eq!(editor.document.lines[0], "original"); // Document should be unchanged
    assert_eq!(editor.cursor_pos(), (0, 0));
}
