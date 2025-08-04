use dmacs::editor::Editor;
use pancurses::Input;

#[test]
fn test_editor_undo() {
    let mut editor = Editor::new(None);
    editor.handle_keypress(Input::Character('a')).unwrap();
    editor.handle_keypress(Input::Character('b')).unwrap();
    editor.handle_keypress(Input::Character('c')).unwrap();
    assert_eq!(editor.document.lines[0], "abc");
    assert_eq!(editor.cursor_pos(), (3, 0));

    editor.handle_keypress(Input::Character('\x1f')).unwrap(); // Ctrl + _ (undo)
    assert_eq!(editor.document.lines[0], "ab");
    assert_eq!(editor.cursor_pos(), (2, 0));

    editor.handle_keypress(Input::Character('\x1f')).unwrap(); // Ctrl + _ (undo)
    assert_eq!(editor.document.lines[0], "a");
    assert_eq!(editor.cursor_pos(), (1, 0));

    editor.handle_keypress(Input::Character('\x1f')).unwrap(); // Ctrl + _ (undo)
    assert_eq!(editor.document.lines[0], "");
    assert_eq!(editor.cursor_pos(), (0, 0));

    // Test undo after newline
    editor.handle_keypress(Input::Character('x')).unwrap();
    editor.handle_keypress(Input::Character('\n')).unwrap();
    editor.handle_keypress(Input::Character('y')).unwrap();
    assert_eq!(editor.document.lines[0], "x");
    assert_eq!(editor.document.lines[1], "y");
    assert_eq!(editor.cursor_pos(), (1, 1));

    editor.handle_keypress(Input::Character('\x1f')).unwrap(); // Undo 'y'
    assert_eq!(editor.document.lines[0], "x");
    assert_eq!(editor.document.lines[1], "");
    assert_eq!(editor.cursor_pos(), (0, 1));

    editor.handle_keypress(Input::Character('\x1f')).unwrap(); // Undo newline
    assert_eq!(editor.document.lines[0], "x");
    assert_eq!(editor.document.lines.len(), 1);
    assert_eq!(editor.cursor_pos(), (1, 0));

    editor.handle_keypress(Input::Character('\x1f')).unwrap(); // Undo 'x'
    assert_eq!(editor.document.lines[0], "");
    assert_eq!(editor.cursor_pos(), (0, 0));

    // Test undo after backspace
    editor.handle_keypress(Input::Character('a')).unwrap();
    editor.handle_keypress(Input::Character('b')).unwrap();
    editor.handle_keypress(Input::KeyBackspace).unwrap();
    assert_eq!(editor.document.lines[0], "a");
    assert_eq!(editor.cursor_pos(), (1, 0));

    editor.handle_keypress(Input::Character('\x1f')).unwrap(); // Undo backspace
    assert_eq!(editor.document.lines[0], "ab");
    assert_eq!(editor.cursor_pos(), (2, 0));
}
