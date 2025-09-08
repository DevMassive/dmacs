use dmacs::editor::Editor;
use dmacs::editor::ui::STATUS_BAR_HEIGHT;
use pancurses::Input;

#[test]
fn test_editor_horizontal_scroll_right() {
    let mut editor = Editor::new(None);
    editor.document.lines[0] = "0123456789abcdef".to_string();
    let screen_cols = 20;
    let scroll_margin = 10;
    editor.update_screen_size(10, screen_cols);

    // Move cursor to the right, beyond the screen width
    for i in 0..12 {
        editor.process_input(Input::KeyRight, false).unwrap();
        editor.scroll();

        let (x, _) = editor.cursor_pos();
        assert_eq!(x, i + 1);

        let display_x = i + 1;
        if display_x < screen_cols - scroll_margin {
            assert_eq!(editor.scroll.col_offset, 0);
        } else {
            let expected_offset = display_x.saturating_sub(screen_cols - scroll_margin);
            assert_eq!(editor.scroll.col_offset, expected_offset);
        }
    }
    assert_eq!(editor.cursor_pos(), (12, 0));
    assert_eq!(editor.scroll.col_offset, 12 - (screen_cols - scroll_margin));
}

#[test]
fn test_editor_horizontal_scroll_left() {
    let mut editor = Editor::new(None);
    editor.document.lines[0] = "0123456789abcdef".to_string();
    let screen_cols = 20;
    let scroll_margin = 10;
    editor.update_screen_size(10, screen_cols);

    // First, scroll to the right
    for _ in 0..15 {
        editor.process_input(Input::KeyRight, false).unwrap();
    }
    editor.scroll();
    assert_eq!(editor.cursor_pos(), (15, 0));
    assert_eq!(editor.scroll.col_offset, 15 - (screen_cols - scroll_margin));

    // Now, move cursor to the left, back into the scrolled area
    for i in 0..10 {
        editor.process_input(Input::KeyLeft, false).unwrap();
        editor.scroll();

        let (x, _) = editor.cursor_pos();
        let _display_x = x; // In this test, display_width is same as byte position
        assert_eq!(x, 14 - i);
    }
    assert_eq!(editor.cursor_pos(), (5, 0));
    assert_eq!(editor.scroll.col_offset, 0);
}

#[test]
fn test_editor_horizontal_scroll_line_change() {
    let mut editor = Editor::new(None);
    editor.document.lines = vec![
        "a very long line to test scrolling".to_string(), // len = 34
        "short line".to_string(),                         // len = 10
    ];
    let screen_cols = 20;
    let scroll_margin = 10;
    editor.update_screen_size(15, screen_cols);

    // Go to the end of the long line to force scrolling
    editor
        .process_input(Input::Character('\x05'), false)
        .unwrap(); // Ctrl-E (end of line)
    editor.scroll();
    assert_eq!(editor.cursor_pos(), (34, 0));
    assert_eq!(editor.scroll.col_offset, 34 - (screen_cols - scroll_margin));

    // Move down to the shorter line
    editor.process_input(Input::KeyDown, false).unwrap();
    editor.scroll();

    // Cursor should be clamped to the end of the shorter line
    assert_eq!(editor.cursor_pos(), (10, 1));
    // The view should scroll left so the cursor is visible
    assert_eq!(editor.scroll.col_offset, 0);
}

#[test]
fn test_editor_scroll_page_down() {
    let mut editor = Editor::new(None);
    for _ in 0..50 {
        // Create 50 lines
        editor.document.lines.push("test line".to_string());
    }
    editor.update_screen_size(25, 80); // screen_rows = 25, usable height = 25 - STATUS_BAR_HEIGHT

    // Initial state
    assert_eq!(editor.cursor_pos().1, 0);
    assert_eq!(editor.scroll.row_offset, 0);

    let usable_height = editor.scroll.screen_rows.saturating_sub(STATUS_BAR_HEIGHT);

    // Scroll down one page
    editor.scroll_page_down();
    assert_eq!(editor.cursor_pos().1, usable_height); // Should move to the top of the next page
    assert_eq!(editor.scroll.row_offset, usable_height);

    // Scroll down another page
    editor.scroll_page_down();
    assert_eq!(editor.cursor_pos().1, usable_height * 2);
    assert_eq!(editor.scroll.row_offset, usable_height * 2);

    // Scroll down beyond document end
    editor.scroll_page_down();
    assert_eq!(editor.cursor_pos().1, 50); // Clamped to last line
    assert_eq!(editor.scroll.row_offset, 50); // Clamped to last line

    // Test with cursor not at 0
    editor.set_cursor_pos(0, 10);
    editor.scroll.row_offset = 10;
    editor.scroll_page_down();
    assert_eq!(editor.cursor_pos().1, 10 + usable_height); // 10 + usable_height
    assert_eq!(editor.scroll.row_offset, 10 + usable_height);
}

#[test]
fn test_editor_scroll_page_up() {
    let mut editor = Editor::new(None);
    for _ in 0..50 {
        // Create 50 lines
        editor.document.lines.push("test line".to_string());
    }
    editor.update_screen_size(25, 80); // screen_rows = 25, usable height = 25 - STATUS_BAR_HEIGHT

    let usable_height = editor.scroll.screen_rows.saturating_sub(STATUS_BAR_HEIGHT);

    // First, scroll down to simulate being in the middle of the document
    editor.set_cursor_pos(0, usable_height * 2);
    editor.scroll.row_offset = usable_height * 2;

    // Scroll up one page
    editor.scroll_page_up();
    assert_eq!(editor.cursor_pos().1, usable_height); // Should move to the top of the previous page
    assert_eq!(editor.scroll.row_offset, usable_height);

    // Scroll up another page
    editor.scroll_page_up();
    assert_eq!(editor.cursor_pos().1, 0);
    assert_eq!(editor.scroll.row_offset, 0);

    // Scroll up beyond document start
    editor.scroll_page_up();
    assert_eq!(editor.cursor_pos().1, 0);
    assert_eq!(editor.scroll.row_offset, 0);

    // Test with cursor not at 0
    editor.set_cursor_pos(0, usable_height + 11);
    editor.scroll.row_offset = usable_height + 11;
    editor.scroll_page_up();
    assert_eq!(editor.cursor_pos().1, 11); // (usable_height + 11) - usable_height
    assert_eq!(editor.scroll.row_offset, 11);
}

#[test]
fn test_editor_vertical_scroll_down() {
    let mut editor = Editor::new(None);
    for _ in 0..50 {
        editor.document.lines.push("test line".to_string());
    }
    editor.update_screen_size(10, 80); // screen_rows = 10, usable height = 10 - STATUS_BAR_HEIGHT

    // Initial state
    assert_eq!(editor.cursor_pos().1, 0);
    assert_eq!(editor.scroll.row_offset, 0);

    let usable_height = editor.scroll.screen_rows.saturating_sub(STATUS_BAR_HEIGHT);

    // Move cursor down line by line, beyond the screen height
    for i in 0..15 {
        editor.process_input(Input::KeyDown, false).unwrap();
        editor.scroll();

        let (_, y) = editor.cursor_pos();
        assert_eq!(y, i + 1);

        if (i + 1) < usable_height {
            // Still within the screen, no scroll
            assert_eq!(editor.scroll.row_offset, 0);
        } else {
            // Scrolled past the screen edge
            assert_eq!(
                editor.scroll.row_offset,
                ((i + 1) as isize - usable_height as isize + 1).max(0) as usize
            );
        }
    }
    assert_eq!(editor.cursor_pos(), (0, 15));
    assert_eq!(editor.scroll.row_offset, 15 - usable_height + 1);
}

#[test]
fn test_editor_vertical_scroll_up() {
    let mut editor = Editor::new(None);
    for _ in 0..50 {
        editor.document.lines.push("test line".to_string());
    }
    editor.update_screen_size(10, 80); // screen_rows = 10, usable height = 10 - STATUS_BAR_HEIGHT

    let usable_height = editor.scroll.screen_rows.saturating_sub(STATUS_BAR_HEIGHT);

    // First, scroll down to simulate being in the middle of the document
    for _ in 0..20 {
        editor.process_input(Input::KeyDown, false).unwrap();
    }
    editor.scroll();
    assert_eq!(editor.cursor_pos(), (0, 20));
    assert_eq!(editor.scroll.row_offset, 20 - usable_height + 1);

    // Now, move cursor up line by line, back into the scrolled area
    for i in 0..10 {
        editor.process_input(Input::KeyUp, false).unwrap();
        editor.scroll();

        let (_, y) = editor.cursor_pos();
        assert_eq!(y, 19 - i);
    }
    assert_eq!(editor.cursor_pos(), (0, 10));
    assert_eq!(editor.scroll.row_offset, 10);
}
