use dmacs::editor::Editor;
use pancurses::Input;

#[test]
fn test_editor_move_cursor() {
    let mut editor = Editor::new(None);
    editor.document.lines = vec!["one".to_string(), "two".to_string()];
    editor.process_input(Input::KeyDown, false).unwrap();
    assert_eq!(editor.cursor_pos(), (0, 1));
    editor.process_input(Input::KeyRight, false).unwrap();
    assert_eq!(editor.cursor_pos(), (1, 1));
    editor.process_input(Input::KeyUp, false).unwrap();
    assert_eq!(editor.cursor_pos(), (1, 0));
    editor.process_input(Input::KeyLeft, false).unwrap();
    assert_eq!(editor.cursor_pos(), (0, 0));
}

#[test]
fn test_editor_go_to_line_boundaries() {
    let mut editor = Editor::new(None);
    editor.document.lines[0] = "hello".to_string();
    editor.process_input(Input::KeyRight, false).unwrap();
    editor.process_input(Input::KeyRight, false).unwrap();
    assert_eq!(editor.cursor_pos(), (2, 0));
    editor
        .process_input(Input::Character('\x01'), false)
        .unwrap(); // Ctrl-A
    assert_eq!(editor.cursor_pos(), (0, 0));
    editor
        .process_input(Input::Character('\x05'), false)
        .unwrap(); // Ctrl-E
    assert_eq!(editor.cursor_pos(), (5, 0));
}

#[test]
fn test_editor_move_cursor_up_at_top_line() {
    let mut editor = Editor::new(None);
    editor.document.lines = vec!["line1".to_string(), "line2".to_string()];
    editor.set_cursor_pos(3, 0); // Set cursor to (3, 0)
    editor.process_input(Input::KeyUp, false).unwrap();
    assert_eq!(editor.cursor_pos(), (0, 0)); // Should move to (0, 0)
}

#[test]
fn test_editor_move_cursor_down_at_bottom_line() {
    let mut editor = Editor::new(None);
    editor.document.lines = vec!["line1".to_string(), "line2".to_string()];
    editor.set_cursor_pos(0, 1); // Set cursor to (0, 1)
    editor.process_input(Input::KeyDown, false).unwrap();
    assert_eq!(editor.cursor_pos(), (5, 1)); // Should move to (end of line, 1)
}

#[test]
fn test_editor_move_cursor_left_across_lines() {
    let mut editor = Editor::new(None);
    editor.document.lines = vec!["line1".to_string(), "line2".to_string()];
    editor.set_cursor_pos(0, 1); // Start at beginning of line2
    editor.process_input(Input::KeyLeft, false).unwrap();
    assert_eq!(editor.cursor_pos(), (5, 0)); // Should move to end of line1
}

#[test]
fn test_editor_move_cursor_right_across_lines() {
    let mut editor = Editor::new(None);
    editor.document.lines = vec!["line1".to_string(), "line2".to_string()];
    editor.set_cursor_pos(5, 0); // Start at end of line1
    editor.process_input(Input::KeyRight, false).unwrap();
    assert_eq!(editor.cursor_pos(), (0, 1)); // Should move to beginning of line2
}

#[test]
fn test_editor_move_cursor_word_left() {
    let mut editor = Editor::new(None);
    editor.document.lines = vec!["word1 word2 word3".to_string()];
    editor.set_cursor_pos(17, 0); // End of "word3"

    editor
        .process_input(Input::Character('\x02'), false)
        .unwrap(); // Ctrl-B
    assert_eq!(editor.cursor_pos(), (12, 0)); // Should move to "word2 "

    editor
        .process_input(Input::Character('\x02'), false)
        .unwrap(); // Ctrl-B
    assert_eq!(editor.cursor_pos(), (6, 0)); // Should move to "word1 "

    editor
        .process_input(Input::Character('\x02'), false)
        .unwrap(); // Ctrl-B
    assert_eq!(editor.cursor_pos(), (0, 0)); // Should move to beginning of line

    // Test with leading/trailing spaces
    editor.document.lines = vec!["  word1  word2  ".to_string()];
    editor.set_cursor_pos(16, 0); // End of line

    editor
        .process_input(Input::Character('\x02'), false)
        .unwrap(); // Ctrl-B
    assert_eq!(editor.cursor_pos(), (9, 0)); // Should move to "word2"

    editor
        .process_input(Input::Character('\x02'), false)
        .unwrap(); // Ctrl-B
    assert_eq!(editor.cursor_pos(), (2, 0)); // Should move to "word1"
}

#[test]
fn test_editor_move_cursor_word_right() {
    let mut editor = Editor::new(None);
    editor.document.lines = vec!["word1 word2 word3".to_string()];
    editor.set_cursor_pos(0, 0); // Beginning of "word1"

    editor.process_input(Input::Character('f'), true).unwrap(); // Alt-f
    assert_eq!(editor.cursor_pos(), (5, 0)); // Should move to "word2"

    editor.process_input(Input::Character('f'), true).unwrap(); // Alt-f
    assert_eq!(editor.cursor_pos(), (11, 0)); // Should move to "word3"

    editor.process_input(Input::Character('f'), true).unwrap(); // Alt-f
    assert_eq!(editor.cursor_pos(), (17, 0)); // Should move to end of line

    // Test with leading/trailing spaces
    editor.document.lines = vec!["  word1  word2  ".to_string()];
    editor.set_cursor_pos(0, 0); // Beginning of line

    editor.process_input(Input::Character('f'), true).unwrap(); // Alt-f
    assert_eq!(editor.cursor_pos(), (7, 0)); // Should move to "word1"

    editor.process_input(Input::Character('f'), true).unwrap(); // Alt-f
    assert_eq!(editor.cursor_pos(), (14, 0)); // Should move to "word2"

    editor.process_input(Input::Character('f'), true).unwrap(); // Alt-f
    assert_eq!(editor.cursor_pos(), (16, 0)); // Should move to end of line
}

#[test]
fn test_go_to_start_of_file() {
    let mut editor = Editor::new(None);
    editor.process_input(Input::Character('a'), false).unwrap();
    editor.insert_newline().unwrap();
    editor.process_input(Input::Character('b'), false).unwrap();
    editor.insert_newline().unwrap();
    editor.process_input(Input::Character('c'), false).unwrap();
    editor.go_to_end_of_file(); // Move to end first to ensure it's not already at start
    assert_eq!(editor.cursor_pos(), (1, 2));
    editor.go_to_start_of_file();
    assert_eq!(editor.cursor_pos(), (0, 0));
    assert_eq!(editor.scroll.row_offset, 0);
    assert_eq!(editor.scroll.col_offset, 0);
}

#[test]
fn test_go_to_end_of_file() {
    let mut editor = Editor::new(None);
    editor.process_input(Input::Character('a'), false).unwrap();
    editor.insert_newline().unwrap();
    editor.process_input(Input::Character('b'), false).unwrap();
    editor.insert_newline().unwrap();
    editor.process_input(Input::Character('c'), false).unwrap();
    editor.go_to_end_of_file();
    assert_eq!(editor.cursor_pos(), (1, 2)); // Line 2 (0-indexed), cursor at end of 'c'
}

#[test]
fn test_editor_move_cursor_word_right_japanese() {
    let mut editor = Editor::new(None);
    editor.document.lines = vec!["漢字とひらがなとカタカナと英字123。".to_string()];
    editor.set_cursor_pos(0, 0);

    // Move to end of "漢字"
    editor.process_input(Input::Character('f'), true).unwrap(); // Alt-f
    assert_eq!(editor.cursor_pos(), (6, 0));

    // Move to end of "とひらがなと"
    editor.process_input(Input::Character('f'), true).unwrap(); // Alt-f
    assert_eq!(editor.cursor_pos(), (24, 0));

    // Move to end of "カタカナ"
    editor.process_input(Input::Character('f'), true).unwrap(); // Alt-f
    assert_eq!(editor.cursor_pos(), (36, 0));

    // Move to end of "と"
    editor.process_input(Input::Character('f'), true).unwrap(); // Alt-f
    assert_eq!(editor.cursor_pos(), (39, 0));

    // Move to end of "英字"
    editor.process_input(Input::Character('f'), true).unwrap(); // Alt-f
    assert_eq!(editor.cursor_pos(), (45, 0));

    // Move to end of "123"
    editor.process_input(Input::Character('f'), true).unwrap(); // Alt-f
    assert_eq!(editor.cursor_pos(), (48, 0));

    // Move to end of "。"
    editor.process_input(Input::Character('f'), true).unwrap(); // Alt-f
    assert_eq!(editor.cursor_pos(), (51, 0));

    // Stays at the end
    editor.process_input(Input::Character('f'), true).unwrap(); // Alt-f
    assert_eq!(editor.cursor_pos(), (51, 0));
}

#[test]
fn test_editor_move_cursor_word_left_japanese() {
    let mut editor = Editor::new(None);
    let line = "漢字とひらがなとカタカナと英字123。".to_string();
    let line_len = line.len();
    editor.document.lines = vec![line];
    editor.set_cursor_pos(line_len, 0);

    // Move to beginning of "。"
    editor
        .process_input(Input::Character('\x02'), false)
        .unwrap(); // Ctrl-B
    assert_eq!(editor.cursor_pos(), (48, 0));

    // Move to beginning of "123"
    editor
        .process_input(Input::Character('\x02'), false)
        .unwrap(); // Ctrl-B
    assert_eq!(editor.cursor_pos(), (45, 0));

    // Move to beginning of "英字"
    editor
        .process_input(Input::Character('\x02'), false)
        .unwrap(); // Ctrl-B
    assert_eq!(editor.cursor_pos(), (39, 0));

    // Move to beginning of "と"
    editor
        .process_input(Input::Character('\x02'), false)
        .unwrap(); // Ctrl-B
    assert_eq!(editor.cursor_pos(), (36, 0));

    // Move to beginning of "カタカナ"
    editor
        .process_input(Input::Character('\x02'), false)
        .unwrap(); // Ctrl-B
    assert_eq!(editor.cursor_pos(), (24, 0));

    // Move to beginning of "とひらがなと"
    editor
        .process_input(Input::Character('\x02'), false)
        .unwrap(); // Ctrl-B
    assert_eq!(editor.cursor_pos(), (6, 0));

    // Move to beginning of "漢字"
    editor
        .process_input(Input::Character('\x02'), false)
        .unwrap(); // Ctrl-B
    assert_eq!(editor.cursor_pos(), (0, 0));

    // Stays at the beginning
    editor
        .process_input(Input::Character('\x02'), false)
        .unwrap(); // Ctrl-B
    assert_eq!(editor.cursor_pos(), (0, 0));
}
