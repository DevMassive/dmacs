use dmacs::editor::Editor;
use pancurses::Input;

#[test]
fn test_editor_undo() {
    let mut editor = Editor::new(None);
    editor.process_input(Input::Character('a'), false).unwrap();
    editor.process_input(Input::Character('b'), false).unwrap();
    editor.process_input(Input::Character('c'), false).unwrap();
    assert_eq!(editor.document.lines[0], "abc");
    assert_eq!(editor.cursor_pos(), (3, 0));

    editor
        .process_input(Input::Character('\x1f'), false)
        .unwrap(); // Ctrl + _ (undo)
    assert_eq!(editor.document.lines[0], "ab");
    assert_eq!(editor.cursor_pos(), (2, 0));

    editor
        .process_input(Input::Character('\x1f'), false)
        .unwrap(); // Ctrl + _ (undo)
    assert_eq!(editor.document.lines[0], "a");
    assert_eq!(editor.cursor_pos(), (1, 0));

    editor
        .process_input(Input::Character('\x1f'), false)
        .unwrap(); // Ctrl + _ (undo)
    assert_eq!(editor.document.lines[0], "");
    assert_eq!(editor.cursor_pos(), (0, 0));

    // Test undo after newline
    editor.process_input(Input::Character('x'), false).unwrap();
    editor.process_input(Input::Character('\n'), false).unwrap();
    editor.process_input(Input::Character('y'), false).unwrap();
    assert_eq!(editor.document.lines[0], "x");
    assert_eq!(editor.document.lines[1], "y");
    assert_eq!(editor.cursor_pos(), (1, 1));

    editor
        .process_input(Input::Character('\x1f'), false)
        .unwrap(); // Undo 'y'
    assert_eq!(editor.document.lines[0], "x");
    assert_eq!(editor.document.lines[1], "");
    assert_eq!(editor.cursor_pos(), (0, 1));

    editor
        .process_input(Input::Character('\x1f'), false)
        .unwrap(); // Undo newline
    assert_eq!(editor.document.lines[0], "x");
    assert_eq!(editor.document.lines.len(), 1);
    assert_eq!(editor.cursor_pos(), (1, 0));

    editor
        .process_input(Input::Character('\x1f'), false)
        .unwrap(); // Undo 'x'
    assert_eq!(editor.document.lines[0], "");
    assert_eq!(editor.cursor_pos(), (0, 0));

    // Test undo after backspace
    editor.process_input(Input::Character('a'), false).unwrap();
    editor.process_input(Input::Character('b'), false).unwrap();
    editor.process_input(Input::KeyBackspace, false).unwrap();
    assert_eq!(editor.document.lines[0], "a");
    assert_eq!(editor.cursor_pos(), (1, 0));

    editor
        .process_input(Input::Character('\x1f'), false)
        .unwrap(); // Undo backspace
    assert_eq!(editor.document.lines[0], "ab");
    assert_eq!(editor.cursor_pos(), (2, 0));
}
