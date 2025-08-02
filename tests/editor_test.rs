use dmacs::editor::Editor;
use pancurses::Input;

#[test]
fn test_editor_initial_state_no_file() {
    let editor = Editor::new(None);
    assert!(!editor.should_quit);
    assert_eq!(editor.document.lines.len(), 1);
    assert_eq!(editor.document.lines[0], "");
}

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
fn test_go_to_line_boundaries() {
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
fn test_editor_with_wide_chars() {
    let mut editor = Editor::new(None);
    editor.handle_keypress(Input::Character('あ'));
    editor.handle_keypress(Input::Character('い'));
    assert_eq!(editor.document.lines[0], "あい");
    assert_eq!(editor.cursor_pos(), (6, 0)); // "あ" and "い" are 3 bytes each
    editor.handle_keypress(Input::KeyLeft);
    assert_eq!(editor.cursor_pos(), (3, 0));
    editor.handle_keypress(Input::KeyBackspace);
    assert_eq!(editor.document.lines[0], "い");
    assert_eq!(editor.cursor_pos(), (0, 0));
}

#[test]
fn test_editor_with_tabs() {
    let mut editor = Editor::new(None);
    editor.handle_keypress(Input::Character('\t'));
    editor.handle_keypress(Input::Character('a'));
    assert_eq!(editor.document.lines[0], "\ta");
    // display width of tab is 8, plus 'a' is 1 = 9
    // cursor byte position is 1 (tab) + 1 (a) = 2
    assert_eq!(editor.cursor_pos(), (2, 0));
}

#[test]
fn test_horizontal_scroll_right() {
    let mut editor = Editor::new(None);
    editor.document.lines[0] = "0123456789abcdef".to_string();
    let screen_width = 10;
    let screen_height = 20;

    // Move cursor to the right, beyond the screen width
    for i in 0..12 {
        editor.handle_keypress(Input::KeyRight);
        editor.scroll(screen_width, screen_height);

        let (x, _) = editor.cursor_pos();
        assert_eq!(x, i + 1);

        if i < 9 {
            // Still within the screen, no scroll
            assert_eq!(editor.col_offset, 0);
        } else {
            // Scrolled past the screen edge
            // display_cursor_x = i + 1
            // col_offset = display_cursor_x - screen_width + 1
            assert_eq!(editor.col_offset, (i + 1) - screen_width + 1);
        }
    }
    assert_eq!(editor.cursor_pos(), (12, 0));
    assert_eq!(editor.col_offset, 3); // 12 - 10 + 1 = 3
}

#[test]
fn test_horizontal_scroll_left() {
    let mut editor = Editor::new(None);
    editor.document.lines[0] = "0123456789abcdef".to_string();
    let screen_width = 10;
    let screen_height = 20;

    // First, scroll to the right
    for _ in 0..15 {
        editor.handle_keypress(Input::KeyRight);
    }
    editor.scroll(screen_width, screen_height);
    assert_eq!(editor.cursor_pos(), (15, 0));
    assert_eq!(editor.col_offset, 6); // 15 - 10 + 1 = 6

    // Now, move cursor to the left, back into the scrolled area
    for i in 0..10 {
        editor.handle_keypress(Input::KeyLeft);
        editor.scroll(screen_width, screen_height);

        let (x, _) = editor.cursor_pos();
        let display_x = x; // In this test, display_width is same as byte position
        assert_eq!(x, 14 - i);

        // if the cursor is scrolled off the left edge, the view should scroll with it
        if display_x < editor.col_offset {
            assert_eq!(editor.col_offset, display_x);
        }
    }
    assert_eq!(editor.cursor_pos(), (5, 0));
    assert_eq!(editor.col_offset, 5);
}

#[test]
fn test_horizontal_scroll_line_change() {
    let mut editor = Editor::new(None);
    editor.document.lines = vec![
        "a very long line to test scrolling".to_string(), // len = 34
        "short line".to_string(),                         // len = 10
    ];
    let screen_width = 15;
    let screen_height = 20;

    // Go to the end of the long line to force scrolling
    editor.handle_keypress(Input::Character('\x05')); // Ctrl-E (end of line)
    editor.scroll(screen_width, screen_height);
    assert_eq!(editor.cursor_pos(), (34, 0));
    assert_eq!(editor.col_offset, 20); // 34 - 15 + 1 = 20

    // Move down to the shorter line
    editor.handle_keypress(Input::KeyDown);
    editor.scroll(screen_width, screen_height);

    // Cursor should be clamped to the end of the shorter line
    assert_eq!(editor.cursor_pos(), (10, 1));
    // The view should scroll left so the cursor is visible
    assert_eq!(editor.col_offset, 10);
}

#[test]
fn test_editor_undo() {
    let mut editor = Editor::new(None);
    editor.handle_keypress(Input::Character('a'));
    editor.handle_keypress(Input::Character('b'));
    editor.handle_keypress(Input::Character('c'));
    assert_eq!(editor.document.lines[0], "abc");
    assert_eq!(editor.cursor_pos(), (3, 0));

    editor.handle_keypress(Input::Character('\x1f')); // Ctrl + _ (undo)
    assert_eq!(editor.document.lines[0], "ab");
    assert_eq!(editor.cursor_pos(), (2, 0));

    editor.handle_keypress(Input::Character('\x1f')); // Ctrl + _ (undo)
    assert_eq!(editor.document.lines[0], "a");
    assert_eq!(editor.cursor_pos(), (1, 0));

    editor.handle_keypress(Input::Character('\x1f')); // Ctrl + _ (undo)
    assert_eq!(editor.document.lines[0], "");
    assert_eq!(editor.cursor_pos(), (0, 0));

    // Test undo after newline
    editor.handle_keypress(Input::Character('x'));
    editor.handle_keypress(Input::Character('\n'));
    editor.handle_keypress(Input::Character('y'));
    assert_eq!(editor.document.lines[0], "x");
    assert_eq!(editor.document.lines[1], "y");
    assert_eq!(editor.cursor_pos(), (1, 1));

    editor.handle_keypress(Input::Character('\x1f')); // Undo 'y'
    assert_eq!(editor.document.lines[0], "x");
    assert_eq!(editor.document.lines[1], "");
    assert_eq!(editor.cursor_pos(), (0, 1));

    editor.handle_keypress(Input::Character('\x1f')); // Undo newline
    assert_eq!(editor.document.lines[0], "x");
    assert_eq!(editor.document.lines.len(), 1);
    assert_eq!(editor.cursor_pos(), (1, 0));

    editor.handle_keypress(Input::Character('\x1f')); // Undo 'x'
    assert_eq!(editor.document.lines[0], "");
    assert_eq!(editor.cursor_pos(), (0, 0));

    // Test undo after backspace
    editor.handle_keypress(Input::Character('a'));
    editor.handle_keypress(Input::Character('b'));
    editor.handle_keypress(Input::KeyBackspace);
    assert_eq!(editor.document.lines[0], "a");
    assert_eq!(editor.cursor_pos(), (1, 0));

    editor.handle_keypress(Input::Character('\x1f')); // Undo backspace
    assert_eq!(editor.document.lines[0], "ab");
    assert_eq!(editor.cursor_pos(), (2, 0));
}

#[test]
fn test_editor_hungry_delete() {
    let mut editor = Editor::new(None);

    // Test deleting word and preceding whitespace
    editor.document.lines[0] = "    hello".to_string();
    editor.set_cursor_pos(9, 0);
    editor.hungry_delete();
    assert_eq!(editor.document.lines[0], "");
    assert_eq!(editor.cursor_pos(), (0, 0));

    // Test deleting word
    editor.document.lines[0] = "hello world".to_string();
    editor.set_cursor_pos(11, 0);
    editor.hungry_delete();
    assert_eq!(editor.document.lines[0], "hello");
    assert_eq!(editor.cursor_pos(), (5, 0));

    // Test deleting across lines (joining lines)
    editor.document.lines = vec!["line1".to_string(), "    line2".to_string()];
    editor.set_cursor_pos(0, 1);
    editor.hungry_delete();
    assert_eq!(editor.document.lines.len(), 1);
    assert_eq!(editor.document.lines[0], "line1    line2");
    assert_eq!(editor.cursor_pos(), (5, 0));

    // Test deleting word with leading whitespace
    editor.document.lines[0] = "  foo bar".to_string();
    editor.set_cursor_pos(9, 0);
    editor.hungry_delete();
    assert_eq!(editor.document.lines[0], "  foo");
    assert_eq!(editor.cursor_pos(), (5, 0));

    // Test deleting only whitespace
    editor.document.lines[0] = "  ".to_string();
    editor.set_cursor_pos(2, 0);
    editor.hungry_delete();
    assert_eq!(editor.document.lines[0], "");
    assert_eq!(editor.cursor_pos(), (0, 0));
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

    editor.handle_keypress(Input::Character('')); // Ctrl-B
    assert_eq!(editor.cursor_pos(), (12, 0)); // Should move to "word2 "

    editor.handle_keypress(Input::Character('')); // Ctrl-B
    assert_eq!(editor.cursor_pos(), (6, 0)); // Should move to "word1 "

    editor.handle_keypress(Input::Character('')); // Ctrl-B
    assert_eq!(editor.cursor_pos(), (0, 0)); // Should move to beginning of line

    // Test with leading/trailing spaces
    editor.document.lines = vec!["  word1  word2  ".to_string()];
    editor.set_cursor_pos(16, 0); // End of line

    editor.handle_keypress(Input::Character('')); // Ctrl-B
    assert_eq!(editor.cursor_pos(), (9, 0)); // Should move to "word2"

    editor.handle_keypress(Input::Character('')); // Ctrl-B
    assert_eq!(editor.cursor_pos(), (2, 0)); // Should move to "word1"
}

#[test]
fn test_editor_move_cursor_word_right() {
    let mut editor = Editor::new(None);
    editor.document.lines = vec!["word1 word2 word3".to_string()];
    editor.set_cursor_pos(0, 0); // Beginning of "word1"

    editor.handle_keypress(Input::Character('')); // Ctrl-F
    assert_eq!(editor.cursor_pos(), (5, 0)); // Should move to "word2"

    editor.handle_keypress(Input::Character('')); // Ctrl-F
    assert_eq!(editor.cursor_pos(), (11, 0)); // Should move to "word3"

    editor.handle_keypress(Input::Character('')); // Ctrl-F
    assert_eq!(editor.cursor_pos(), (17, 0)); // Should move to end of line

    // Test with leading/trailing spaces
    editor.document.lines = vec!["  word1  word2  ".to_string()];
    editor.set_cursor_pos(0, 0); // Beginning of line

    editor.handle_keypress(Input::Character('')); // Ctrl-F
    assert_eq!(editor.cursor_pos(), (7, 0)); // Should move to "word1"

    editor.handle_keypress(Input::Character('')); // Ctrl-F
    assert_eq!(editor.cursor_pos(), (14, 0)); // Should move to "word2"

    editor.handle_keypress(Input::Character('')); // Ctrl-F
    assert_eq!(editor.cursor_pos(), (16, 0)); // Should move to end of line
}

#[test]
fn test_editor_kill_line_middle_of_line() {
    let mut editor = Editor::new(None);
    editor.document.lines = vec!["hello world".to_string()];
    editor.set_cursor_pos(6, 0); // Cursor at 'w' in "world"
    editor.kill_line();
    assert_eq!(editor.document.lines[0], "hello ");
    assert_eq!(editor.kill_buffer, "world");
    assert_eq!(editor.cursor_pos(), (6, 0));
}

#[test]
fn test_editor_kill_line_end_of_line_not_last_line() {
    let mut editor = Editor::new(None);
    editor.document.lines = vec!["hello".to_string(), "world".to_string()];
    editor.set_cursor_pos(5, 0); // Cursor at end of "hello"
    editor.kill_line();
    assert_eq!(editor.document.lines.len(), 1);
    assert_eq!(editor.document.lines[0], "helloworld");
    assert_eq!(editor.kill_buffer, "\nworld"); // Newline + content of next line
    assert_eq!(editor.cursor_pos(), (5, 0));
}

#[test]
fn test_editor_kill_line_empty_line_not_last_line() {
    let mut editor = Editor::new(None);
    editor.document.lines = vec!["line1".to_string(), "".to_string(), "line3".to_string()];
    editor.set_cursor_pos(0, 1); // Cursor at beginning of empty line
    editor.kill_line();
    assert_eq!(editor.document.lines.len(), 2);
    assert_eq!(editor.document.lines[0], "line1");
    assert_eq!(editor.document.lines[1], "line3");
    assert_eq!(editor.kill_buffer, "\n"); // Only newline killed
    assert_eq!(editor.cursor_pos(), (0, 1));
}

#[test]
fn test_editor_kill_line_last_line() {
    let mut editor = Editor::new(None);
    editor.document.lines = vec!["last line".to_string()];
    editor.set_cursor_pos(0, 0);
    editor.kill_line();
    assert_eq!(editor.document.lines[0], "");
    assert_eq!(editor.kill_buffer, "last line");
    assert_eq!(editor.cursor_pos(), (0, 0));
}

#[test]
fn test_editor_yank_single_line() {
    let mut editor = Editor::new(None);
    editor.kill_buffer = "yanked text".to_string();
    editor.document.lines = vec!["start ".to_string(), "end".to_string()];
    editor.set_cursor_pos(6, 0); // After "start "
    editor.yank();
    assert_eq!(editor.document.lines[0], "start yanked text");
    assert_eq!(editor.cursor_pos(), (17, 0)); // Cursor after yanked text
}

#[test]
fn test_editor_yank_multiple_lines() {
    let mut editor = Editor::new(None);
    editor.kill_buffer = "line1\nline2\nline3".to_string();
    editor.document.lines = vec!["start".to_string(), "end".to_string()];
    editor.set_cursor_pos(5, 0); // After "start"
    editor.yank();
    assert_eq!(editor.document.lines.len(), 4);
    assert_eq!(editor.document.lines[0], "startline1");
    assert_eq!(editor.document.lines[1], "line2");
    assert_eq!(editor.document.lines[2], "line3");
    assert_eq!(editor.document.lines[3], "end");
    assert_eq!(editor.cursor_pos(), (5, 2)); // Cursor at end of last yanked line
}

#[test]
fn test_editor_consecutive_kill_line() {
    let mut editor = Editor::new(None);
    editor.document.lines = vec![
        "line one".to_string(),
        "line two".to_string(),
        "line three".to_string(),
    ];

    // Kill "line one"
    editor.set_cursor_pos(0, 0);
    editor.handle_keypress(Input::Character('\x0b')); // Ctrl-K
    assert_eq!(editor.kill_buffer, "line one");
    assert_eq!(editor.document.lines.len(), 3);
    assert_eq!(editor.document.lines[0], ""); // "line one" should be removed

    editor.set_cursor_pos(0, 0);
    editor.handle_keypress(Input::Character('\x0b')); // Ctrl-K
    assert_eq!(editor.kill_buffer, "line one\n");
    assert_eq!(editor.document.lines.len(), 2);
    assert_eq!(editor.document.lines[0], "line two"); // "line one\n" should be removed

    // Kill "line two" immediately after
    editor.set_cursor_pos(0, 0); // Cursor is now at the start of "line two"
    editor.handle_keypress(Input::Character('\x0b')); // Ctrl-K
    assert_eq!(editor.kill_buffer, "line one\nline two"); // Should append
    assert_eq!(editor.document.lines.len(), 2);
    assert_eq!(editor.document.lines[0], ""); // "line two" should be removed

    editor.set_cursor_pos(0, 0);
    editor.handle_keypress(Input::Character('\x0b')); // Ctrl-K
    assert_eq!(editor.kill_buffer, "line one\nline two\n"); // Should append
    assert_eq!(editor.document.lines.len(), 1);
    assert_eq!(editor.document.lines[0], "line three"); // "line two" should be removed

    // Yank the accumulated content
    editor.set_cursor_pos(0, 0);
    editor.handle_keypress(Input::Character('\x19')); // Ctrl-Y
    assert_eq!(editor.document.lines.len(), 3);
    assert_eq!(editor.document.lines[0], "line one");
    assert_eq!(editor.document.lines[1], "line two");
    assert_eq!(editor.document.lines[2], "line three");
}

#[test]
fn test_editor_move_line_up() {
    let mut editor = Editor::new(None);
    editor.document.lines = vec![
        "line1".to_string(),
        "line2".to_string(),
        "line3".to_string(),
    ];
    editor.set_cursor_pos(0, 1); // Cursor on line2
    editor.move_line_up(); // Call the function directly
    assert_eq!(editor.document.lines[0], "line2");
    assert_eq!(editor.document.lines[1], "line1");
    assert_eq!(editor.document.lines[2], "line3");
    assert_eq!(editor.cursor_pos(), (0, 0)); // Cursor should move up with the line

    // Try moving up from the first line (should not change document, only status message)
    editor.move_line_up(); // Call the function directly
    assert_eq!(editor.document.lines[0], "line2");
    assert_eq!(editor.document.lines[1], "line1");
    assert_eq!(editor.document.lines[2], "line3");
    assert_eq!(editor.cursor_pos(), (0, 0));
}

#[test]
fn test_editor_move_line_down() {
    let mut editor = Editor::new(None);
    editor.document.lines = vec![
        "line1".to_string(),
        "line2".to_string(),
        "line3".to_string(),
    ];
    editor.set_cursor_pos(0, 1); // Cursor on line2
    editor.move_line_down(); // Call the function directly
    assert_eq!(editor.document.lines[0], "line1");
    assert_eq!(editor.document.lines[1], "line3");
    assert_eq!(editor.document.lines[2], "line2");
    assert_eq!(editor.cursor_pos(), (0, 2)); // Cursor should move down with the line

    // Try moving down from the last line (should not change document, only status message)
    editor.move_line_down(); // Call the function directly
    assert_eq!(editor.document.lines[0], "line1");
    assert_eq!(editor.document.lines[1], "line3");
    assert_eq!(editor.document.lines[2], "line2");
    assert_eq!(editor.cursor_pos(), (0, 2));
}

#[test]
fn test_editor_yank_empty_kill_buffer() {
    let mut editor = Editor::new(None);
    editor.kill_buffer = "".to_string();
    editor.document.lines = vec!["original".to_string()];
    editor.set_cursor_pos(0, 0);
    editor.yank();
    assert_eq!(editor.document.lines[0], "original"); // Document should be unchanged
    assert_eq!(editor.cursor_pos(), (0, 0));
}
