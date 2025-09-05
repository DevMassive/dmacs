use dmacs::editor::Editor;
use pancurses::Input;

#[test]
fn test_editor_insert_char() {
    let mut editor = Editor::new(None);
    editor.process_input(Input::Character('a'), false).unwrap();
    assert_eq!(editor.document.lines[0], "a");
    assert_eq!(editor.cursor_pos(), (1, 0));
}

#[test]
fn test_editor_delete_char() {
    let mut editor = Editor::new(None);
    editor.process_input(Input::Character('a'), false).unwrap();
    editor.process_input(Input::KeyBackspace, false).unwrap();
    assert_eq!(editor.document.lines[0], "");
    assert_eq!(editor.cursor_pos(), (0, 0));
}

#[test]
fn test_editor_delete_forward_char() {
    let mut editor = Editor::new(None);
    editor.process_input(Input::Character('a'), false).unwrap();
    editor.process_input(Input::KeyLeft, false).unwrap();
    editor
        .process_input(Input::Character('\x04'), false)
        .unwrap(); // Ctrl-D
    assert_eq!(editor.document.lines[0], "");
    assert_eq!(editor.cursor_pos(), (0, 0));
}

#[test]
fn test_editor_insert_newline() {
    let mut editor = Editor::new(None);
    editor.process_input(Input::Character('a'), false).unwrap();
    editor
        .process_input(Input::Character('\x0A'), false)
        .unwrap();
    assert_eq!(editor.document.lines.len(), 2);
    assert_eq!(editor.document.lines[0], "a");
    assert_eq!(editor.document.lines[1], "");
    assert_eq!(editor.cursor_pos(), (0, 1));
}

#[test]
fn test_editor_insert_newline_with_indent() {
    let mut editor = Editor::new(None);
    editor.document.lines[0] = "  Hello".to_string();
    editor.set_cursor_pos(7, 0); // End of line
    editor.insert_newline().unwrap();

    assert_eq!(editor.document.lines.len(), 2);
    assert_eq!(editor.document.lines[0], "  Hello");
    assert_eq!(editor.document.lines[1], "  ");
    assert_eq!(editor.cursor_y, 1);
    assert_eq!(editor.cursor_x, 2);
}

#[test]
fn test_editor_insert_newline_with_list_marker() {
    let mut editor = Editor::new(None);
    editor.document.lines[0] = "  - Hello".to_string();
    editor.set_cursor_pos(9, 0); // End of line
    editor.insert_newline().unwrap();

    assert_eq!(editor.document.lines.len(), 2);
    assert_eq!(editor.document.lines[0], "  - Hello");
    assert_eq!(editor.document.lines[1], "  - ");
    assert_eq!(editor.cursor_y, 1);
    assert_eq!(editor.cursor_x, 4); // "  - "
}

#[test]
fn test_editor_insert_newline_with_task_marker() {
    let mut editor = Editor::new(None);
    editor.document.lines[0] = "  - [ ] Task 1".to_string();
    editor.set_cursor_pos(15, 0); // End of line
    editor.insert_newline().unwrap();

    assert_eq!(editor.document.lines.len(), 2);
    assert_eq!(editor.document.lines[0], "  - [ ] Task 1");
    assert_eq!(editor.document.lines[1], "  - [ ] ");
    assert_eq!(editor.cursor_y, 1);
    assert_eq!(editor.cursor_x, 8); // "  - [ ] "
}

#[test]
fn test_editor_insert_newline_with_checked_task_marker() {
    let mut editor = Editor::new(None);
    editor.document.lines[0] = "  - [x] Task 1".to_string();
    editor.set_cursor_pos(15, 0); // End of line
    editor.insert_newline().unwrap();

    assert_eq!(editor.document.lines.len(), 2);
    assert_eq!(editor.document.lines[0], "  - [x] Task 1");
    assert_eq!(editor.document.lines[1], "  - [ ] "); // Should be unchecked
    assert_eq!(editor.cursor_y, 1);
    assert_eq!(editor.cursor_x, 8); // "  - [ ] "
}

#[test]
fn test_editor_backspace_indentation() {
    let mut editor = Editor::new(None);
    editor.document.lines[0] = "    Hello".to_string();
    editor.set_cursor_pos(4, 0); // After indentation
    editor.delete_char().unwrap(); // Backspace
    assert_eq!(editor.document.lines[0], "  Hello");
    assert_eq!(editor.cursor_x, 2);

    editor.delete_char().unwrap(); // Backspace
    assert_eq!(editor.document.lines[0], "Hello");
    assert_eq!(editor.cursor_x, 0);

    // Should not delete 2 chars if not at end of indentation
    editor.document.lines[0] = "  Hello  World".to_string();
    editor.set_cursor_pos(9, 0); // After "  Hello  "
    editor.delete_char().unwrap(); // Backspace
    assert_eq!(editor.document.lines[0], "  Hello World");
    assert_eq!(editor.cursor_x, 8);
}

#[test]
fn test_editor_backspace_line_join() {
    let mut editor = Editor::new(None);
    editor.document.lines = vec!["hello".to_string(), "world".to_string()];
    editor.set_cursor_pos(0, 1); // Set cursor to beginning of "world"
    editor.process_input(Input::KeyBackspace, false).unwrap();
    assert_eq!(editor.document.lines.len(), 1);
    assert_eq!(editor.document.lines[0], "helloworld");
    assert_eq!(editor.cursor_pos(), (5, 0));
}

#[test]
fn test_editor_delete_to_end_of_line() {
    let mut editor = Editor::new(None);
    editor.document.lines[0] = "hello world".to_string();
    editor.process_input(Input::KeyRight, false).unwrap();
    editor.process_input(Input::KeyRight, false).unwrap();
    editor.process_input(Input::KeyRight, false).unwrap();
    editor.process_input(Input::KeyRight, false).unwrap();
    editor.process_input(Input::KeyRight, false).unwrap();
    editor
        .process_input(Input::Character('\x0b'), false)
        .unwrap(); // Ctrl-K
    assert_eq!(editor.document.lines[0], "hello");
    assert_eq!(editor.cursor_pos(), (5, 0));
}

#[test]
fn test_editor_delete_to_end_of_line_at_end() {
    let mut editor = Editor::new(None);
    editor.document.lines = vec!["hello".to_string(), "world".to_string()];
    editor
        .process_input(Input::Character('\x05'), false)
        .unwrap(); // Ctrl-E
    editor
        .process_input(Input::Character('\x0b'), false)
        .unwrap(); // Ctrl-K
    assert_eq!(editor.document.lines.len(), 1);
    assert_eq!(editor.document.lines[0], "helloworld");
    assert_eq!(editor.cursor_pos(), (5, 0));
}

#[test]
fn test_editor_del_key() {
    let mut editor = Editor::new(None);
    editor.process_input(Input::Character('a'), false).unwrap();
    editor
        .process_input(Input::Character('\x7f'), false)
        .unwrap();
    assert_eq!(editor.document.lines[0], "");
    assert_eq!(editor.cursor_pos(), (0, 0));
}

#[test]
fn test_editor_hungry_delete() {
    let mut editor = Editor::new(None);

    // Test deleting word and preceding whitespace
    editor.document.lines[0] = "    hello".to_string();
    editor.set_cursor_pos(9, 0);
    editor.process_input(Input::KeyBackspace, true).unwrap();
    assert_eq!(editor.document.lines[0], "");
    assert_eq!(editor.cursor_pos(), (0, 0));

    // Test deleting word
    editor.document.lines[0] = "hello world".to_string();
    editor.set_cursor_pos(11, 0);
    editor.process_input(Input::KeyBackspace, true).unwrap();
    assert_eq!(editor.document.lines[0], "hello");
    assert_eq!(editor.cursor_pos(), (5, 0));

    // Test deleting across lines (joining lines)
    editor.document.lines = vec!["line1".to_string(), "    line2".to_string()];
    editor.set_cursor_pos(0, 1);
    editor.process_input(Input::KeyBackspace, true).unwrap();
    assert_eq!(editor.document.lines.len(), 1);
    assert_eq!(editor.document.lines[0], "line1    line2");
    assert_eq!(editor.cursor_pos(), (5, 0));

    // Test deleting word with leading whitespace
    editor.document.lines[0] = "  foo bar".to_string();
    editor.set_cursor_pos(9, 0);
    editor.process_input(Input::KeyBackspace, true).unwrap();
    assert_eq!(editor.document.lines[0], "  foo");
    assert_eq!(editor.cursor_pos(), (5, 0));

    // Test deleting only whitespace
    editor.document.lines[0] = "  ".to_string();
    editor.set_cursor_pos(2, 0);
    editor.process_input(Input::KeyBackspace, true).unwrap();
    assert_eq!(editor.document.lines[0], "");
    assert_eq!(editor.cursor_pos(), (0, 0));
}

#[test]
fn test_editor_backspace_empty_list_item() {
    let mut editor = Editor::new(None);

    // Test deleting "- "
    editor.document.lines[0] = "- ".to_string();
    editor.set_cursor_pos(2, 0);
    editor.delete_char().unwrap();
    assert_eq!(editor.document.lines[0], "");
    assert_eq!(editor.cursor_pos(), (0, 0));

    // Test deleting "- [ ] "
    editor.document.lines[0] = "- [ ] ".to_string();
    editor.set_cursor_pos(6, 0);
    editor.delete_char().unwrap();
    assert_eq!(editor.document.lines[0], "");
    assert_eq!(editor.cursor_pos(), (0, 0));

    // Test deleting "- [x] "
    editor.document.lines[0] = "- [x] ".to_string();
    editor.set_cursor_pos(6, 0);
    editor.delete_char().unwrap();
    assert_eq!(editor.document.lines[0], "");
    assert_eq!(editor.cursor_pos(), (0, 0));

    // Test with indentation
    editor.document.lines[0] = "  - ".to_string();
    editor.set_cursor_pos(4, 0);
    editor.delete_char().unwrap();
    assert_eq!(editor.document.lines[0], "  ");
    assert_eq!(editor.cursor_pos(), (2, 0));

    // Test with extra whitespace
    editor.document.lines[0] = "- [ ]   ".to_string();
    editor.set_cursor_pos(9, 0);
    editor.delete_char().unwrap();
    assert_eq!(editor.document.lines[0], "");
    assert_eq!(editor.cursor_pos(), (0, 0));

    // Test with indentation and extra whitespace
    editor.document.lines[0] = "    - [x]  ".to_string();
    editor.set_cursor_pos(11, 0);
    editor.delete_char().unwrap();
    assert_eq!(editor.document.lines[0], "    ");
    assert_eq!(editor.cursor_pos(), (4, 0));

    // Test that it doesn't delete when not at the end of the line
    editor.document.lines[0] = "- [x] something".to_string();
    editor.set_cursor_pos(15, 0); // cursor at the very end
    editor.delete_char().unwrap();
    assert_eq!(editor.document.lines[0], "- [x] somethin"); // regular backspace
    assert_eq!(editor.cursor_pos(), (14, 0));
}

#[test]
fn test_editor_newline_empty_list_item() {
    let mut editor = Editor::new(None);

    // Test with "- "
    editor.document.lines[0] = "- ".to_string();
    editor.set_cursor_pos(2, 0);
    editor.insert_newline().unwrap();
    assert_eq!(editor.document.lines[0], "");
    assert_eq!(editor.cursor_pos(), (0, 0));

    // Test with "  - "
    editor.document.lines = vec!["  - ".to_string()];
    editor.set_cursor_pos(4, 0);
    editor.insert_newline().unwrap();
    assert_eq!(editor.document.lines[0], "");
    assert_eq!(editor.cursor_pos(), (0, 0));

    // Test with "- [ ] "
    editor.document.lines = vec!["- [ ] ".to_string()];
    editor.set_cursor_pos(6, 0);
    editor.insert_newline().unwrap();
    assert_eq!(editor.document.lines[0], "");
    assert_eq!(editor.cursor_pos(), (0, 0));

    // Test with "- [x] "
    editor.document.lines = vec!["- [x] ".to_string()];
    editor.set_cursor_pos(6, 0);
    editor.insert_newline().unwrap();
    assert_eq!(editor.document.lines[0], "");
    assert_eq!(editor.cursor_pos(), (0, 0));

    // Negative test: multiple spaces after marker
    editor.document.lines = vec!["-   ".to_string()];
    editor.set_cursor_pos(4, 0);
    editor.insert_newline().unwrap();
    assert_eq!(editor.document.lines.len(), 2);
    assert_eq!(editor.document.lines[0], "-   ");
    assert_eq!(editor.document.lines[1], "- ");

    // Negative test: multiple spaces after checkbox
    editor.document.lines = vec!["- [ ]   ".to_string()];
    editor.set_cursor_pos(9, 0);
    editor.insert_newline().unwrap();
    assert_eq!(editor.document.lines.len(), 2);
    assert_eq!(editor.document.lines[0], "- [ ]   ");
    assert_eq!(editor.document.lines[1], "- [ ] ");
}
