use dmacs::editor::Editor;
use pancurses::Input;

#[test]
fn test_editor_move_cursor() {
    let mut editor = Editor::new(None);
    editor.document.lines = vec!["one".to_string(), "two".to_string()];
    editor.handle_keypress(Input::KeyDown);
    assert_eq!(editor.cursor_pos(), (0, 1));
    editor.handle_keypress(Input::KeyRight);
    assert_eq!(editor.cursor_pos(), (1, 1));
    editor.handle_keypress(Input::KeyUp);
    assert_eq!(editor.cursor_pos(), (1, 0));
    editor.handle_keypress(Input::KeyLeft);
    assert_eq!(editor.cursor_pos(), (0, 0));
}

#[test]
fn test_editor_go_to_line_boundaries() {
    let mut editor = Editor::new(None);
    editor.document.lines[0] = "hello".to_string();
    editor.handle_keypress(Input::KeyRight);
    editor.handle_keypress(Input::KeyRight);
    assert_eq!(editor.cursor_pos(), (2, 0));
    editor.handle_keypress(Input::Character('\x01')); // Ctrl-A
    assert_eq!(editor.cursor_pos(), (0, 0));
    editor.handle_keypress(Input::Character('\x05')); // Ctrl-E
    assert_eq!(editor.cursor_pos(), (5, 0));
}

#[test]
fn test_editor_move_cursor_up_at_top_line() {
    let mut editor = Editor::new(None);
    editor.document.lines = vec!["line1".to_string(), "line2".to_string()];
    editor.set_cursor_pos(3, 0); // Set cursor to (3, 0)
    editor.handle_keypress(Input::KeyUp);
    assert_eq!(editor.cursor_pos(), (0, 0)); // Should move to (0, 0)
}

#[test]
fn test_editor_move_cursor_down_at_bottom_line() {
    let mut editor = Editor::new(None);
    editor.document.lines = vec!["line1".to_string(), "line2".to_string()];
    editor.set_cursor_pos(0, 1); // Set cursor to (0, 1)
    editor.handle_keypress(Input::KeyDown);
    assert_eq!(editor.cursor_pos(), (5, 1)); // Should move to (end of line, 1)
}

#[test]
fn test_editor_move_cursor_left_across_lines() {
    let mut editor = Editor::new(None);
    editor.document.lines = vec!["line1".to_string(), "line2".to_string()];
    editor.set_cursor_pos(0, 1); // Start at beginning of line2
    editor.handle_keypress(Input::KeyLeft);
    assert_eq!(editor.cursor_pos(), (5, 0)); // Should move to end of line1
}

#[test]
fn test_editor_move_cursor_right_across_lines() {
    let mut editor = Editor::new(None);
    editor.document.lines = vec!["line1".to_string(), "line2".to_string()];
    editor.set_cursor_pos(5, 0); // Start at end of line1
    editor.handle_keypress(Input::KeyRight);
    assert_eq!(editor.cursor_pos(), (0, 1)); // Should move to beginning of line2
}

#[test]
fn test_editor_move_cursor_word_left() {
    let mut editor = Editor::new(None);
    editor.document.lines = vec!["word1 word2 word3".to_string()];
    editor.set_cursor_pos(17, 0); // End of "word3"

    editor.handle_keypress(Input::Character('\x02')); // Ctrl-B
    assert_eq!(editor.cursor_pos(), (12, 0)); // Should move to "word2 "

    editor.handle_keypress(Input::Character('\x02')); // Ctrl-B
    assert_eq!(editor.cursor_pos(), (6, 0)); // Should move to "word1 "

    editor.handle_keypress(Input::Character('\x02')); // Ctrl-B
    assert_eq!(editor.cursor_pos(), (0, 0)); // Should move to beginning of line

    // Test with leading/trailing spaces
    editor.document.lines = vec!["  word1  word2  ".to_string()];
    editor.set_cursor_pos(16, 0); // End of line

    editor.handle_keypress(Input::Character('\x02')); // Ctrl-B
    assert_eq!(editor.cursor_pos(), (9, 0)); // Should move to "word2"

    editor.handle_keypress(Input::Character('\x02')); // Ctrl-B
    assert_eq!(editor.cursor_pos(), (2, 0)); // Should move to "word1"
}

#[test]
fn test_editor_move_cursor_word_right() {
    let mut editor = Editor::new(None);
    editor.document.lines = vec!["word1 word2 word3".to_string()];
    editor.set_cursor_pos(0, 0); // Beginning of "word1"

    editor.handle_keypress(Input::Character('\x06')); // Ctrl-F
    assert_eq!(editor.cursor_pos(), (5, 0)); // Should move to "word2"

    editor.handle_keypress(Input::Character('\x06')); // Ctrl-F
    assert_eq!(editor.cursor_pos(), (11, 0)); // Should move to "word3"

    editor.handle_keypress(Input::Character('\x06')); // Ctrl-F
    assert_eq!(editor.cursor_pos(), (17, 0)); // Should move to end of line

    // Test with leading/trailing spaces
    editor.document.lines = vec!["  word1  word2  ".to_string()];
    editor.set_cursor_pos(0, 0); // Beginning of line

    editor.handle_keypress(Input::Character('\x06')); // Ctrl-F
    assert_eq!(editor.cursor_pos(), (7, 0)); // Should move to "word1"

    editor.handle_keypress(Input::Character('\x06')); // Ctrl-F
    assert_eq!(editor.cursor_pos(), (14, 0)); // Should move to "word2"

    editor.handle_keypress(Input::Character('\x06')); // Ctrl-F
    assert_eq!(editor.cursor_pos(), (16, 0)); // Should move to end of line
}
