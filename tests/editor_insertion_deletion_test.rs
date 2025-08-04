use dmacs::editor::Editor;
use pancurses::Input;

#[test]
fn test_editor_insert_char() {
    let mut editor = Editor::new(None);
    editor.handle_keypress(Input::Character('a'));
    assert_eq!(editor.document.lines[0], "a");
    assert_eq!(editor.cursor_pos(), (1, 0));
}

#[test]
fn test_editor_delete_char() {
    let mut editor = Editor::new(None);
    editor.handle_keypress(Input::Character('a'));
    editor.handle_keypress(Input::KeyBackspace);
    assert_eq!(editor.document.lines[0], "");
    assert_eq!(editor.cursor_pos(), (0, 0));
}

#[test]
fn test_editor_delete_forward_char() {
    let mut editor = Editor::new(None);
    editor.handle_keypress(Input::Character('a'));
    editor.handle_keypress(Input::KeyLeft);
    editor.handle_keypress(Input::Character('\x04')); // Ctrl-D
    assert_eq!(editor.document.lines[0], "");
    assert_eq!(editor.cursor_pos(), (0, 0));
}

#[test]
fn test_editor_insert_newline() {
    let mut editor = Editor::new(None);
    editor.handle_keypress(Input::Character('a'));
    editor.handle_keypress(Input::Character('\x0A'));
    assert_eq!(editor.document.lines.len(), 2);
    assert_eq!(editor.document.lines[0], "a");
    assert_eq!(editor.document.lines[1], "");
    assert_eq!(editor.cursor_pos(), (0, 1));
}

#[test]
fn test_editor_backspace_line_join() {
    let mut editor = Editor::new(None);
    editor.document.lines = vec!["hello".to_string(), "world".to_string()];
    editor.set_cursor_pos(0, 1); // Set cursor to beginning of "world"
    editor.handle_keypress(Input::KeyBackspace);
    assert_eq!(editor.document.lines.len(), 1);
    assert_eq!(editor.document.lines[0], "helloworld");
    assert_eq!(editor.cursor_pos(), (5, 0));
}

#[test]
fn test_editor_delete_to_end_of_line() {
    let mut editor = Editor::new(None);
    editor.document.lines[0] = "hello world".to_string();
    editor.handle_keypress(Input::KeyRight);
    editor.handle_keypress(Input::KeyRight);
    editor.handle_keypress(Input::KeyRight);
    editor.handle_keypress(Input::KeyRight);
    editor.handle_keypress(Input::KeyRight);
    editor.handle_keypress(Input::Character('\x0b')); // Ctrl-K
    assert_eq!(editor.document.lines[0], "hello");
    assert_eq!(editor.cursor_pos(), (5, 0));
}

#[test]
fn test_editor_delete_to_end_of_line_at_end() {
    let mut editor = Editor::new(None);
    editor.document.lines = vec!["hello".to_string(), "world".to_string()];
    editor.handle_keypress(Input::Character('\x05')); // Ctrl-E
    editor.handle_keypress(Input::Character('\x0b')); // Ctrl-K
    assert_eq!(editor.document.lines.len(), 1);
    assert_eq!(editor.document.lines[0], "helloworld");
    assert_eq!(editor.cursor_pos(), (5, 0));
}

#[test]
fn test_editor_del_key() {
    let mut editor = Editor::new(None);
    editor.handle_keypress(Input::Character('a'));
    editor.handle_keypress(Input::Character('\x7f'));
    assert_eq!(editor.document.lines[0], "");
    assert_eq!(editor.cursor_pos(), (0, 0));
}

#[test]
fn test_editor_hungry_delete() {
    let mut editor = Editor::new(None);

    // Test deleting word and preceding whitespace
    editor.document.lines[0] = "    hello".to_string();
    editor.set_cursor_pos(9, 0);
    editor.process_input(Input::Character('\x1b'), Some(Input::KeyBackspace), None);
    assert_eq!(editor.document.lines[0], "");
    assert_eq!(editor.cursor_pos(), (0, 0));

    // Test deleting word
    editor.document.lines[0] = "hello world".to_string();
    editor.set_cursor_pos(11, 0);
    editor.process_input(Input::Character('\x1b'), Some(Input::KeyBackspace), None);
    assert_eq!(editor.document.lines[0], "hello");
    assert_eq!(editor.cursor_pos(), (5, 0));

    // Test deleting across lines (joining lines)
    editor.document.lines = vec!["line1".to_string(), "    line2".to_string()];
    editor.set_cursor_pos(0, 1);
    editor.process_input(Input::Character('\x1b'), Some(Input::KeyBackspace), None);
    assert_eq!(editor.document.lines.len(), 1);
    assert_eq!(editor.document.lines[0], "line1    line2");
    assert_eq!(editor.cursor_pos(), (5, 0));

    // Test deleting word with leading whitespace
    editor.document.lines[0] = "  foo bar".to_string();
    editor.set_cursor_pos(9, 0);
    editor.process_input(Input::Character('\x1b'), Some(Input::KeyBackspace), None);
    assert_eq!(editor.document.lines[0], "  foo");
    assert_eq!(editor.cursor_pos(), (5, 0));

    // Test deleting only whitespace
    editor.document.lines[0] = "  ".to_string();
    editor.set_cursor_pos(2, 0);
    editor.process_input(Input::Character('\x1b'), Some(Input::KeyBackspace), None);
    assert_eq!(editor.document.lines[0], "");
    assert_eq!(editor.cursor_pos(), (0, 0));
}
