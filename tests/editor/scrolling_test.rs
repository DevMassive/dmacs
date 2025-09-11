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

#[test]
#[ignore = "This test interacts with the terminal and is best run manually"]
fn test_horizontal_scroll_visual_and_cursor_pinning() {
    // This test initializes a pancurses window to check the actual visual output.
    // It might interfere with your terminal if it panics.

    // 1. Setup
    let window = pancurses::initscr();
    window.keypad(true);
    pancurses::noecho();
    pancurses::curs_set(0); // Hide cursor for stable testing

    let mut editor = Editor::new(None);
    let screen_rows = 10;
    let screen_cols = 40;
    let scroll_margin = 10;
    editor.update_screen_size(screen_rows, screen_cols);
    editor.document.lines[0] = "This is a very long line of text to test the horizontal scrolling behavior of the editor.".to_string();

    // 2. Action: Move cursor to trigger scrolling
    let move_count = 45;
    for _ in 0..move_count {
        editor.process_input(Input::KeyRight, false).unwrap();
    }
    editor.draw(&window);

    // 3. Assertions for scrolled state
    assert_eq!(editor.cursor_pos().0, move_count);
    let expected_cursor_x = screen_cols - scroll_margin;

    // Check cursor position on screen
    let (cury, curx) = window.get_cur_yx();
    assert_eq!(cury, STATUS_BAR_HEIGHT as i32, "Cursor Y position is incorrect");
    assert_eq!(curx, expected_cursor_x as i32, "Cursor should be pinned to the right margin");

    // Check displayed text on the line
    let line_y = STATUS_BAR_HEIGHT as i32;
    let mut displayed_line = String::new();
    for x in 0..screen_cols {
        let ch = window.mvinch(line_y, x as i32);
        displayed_line.push((ch & pancurses::A_CHARTEXT) as u8 as char);
    }
    
    let expected_col_offset = move_count - expected_cursor_x;
    assert_eq!(editor.scroll.col_offset, expected_col_offset);
    let expected_line_slice = &editor.document.lines[0][expected_col_offset..expected_col_offset + screen_cols];
    assert_eq!(displayed_line, expected_line_slice, "Displayed text is not scrolled correctly");

    // 4. Action: Move cursor left, still in scrolled mode
    editor.process_input(Input::KeyLeft, false).unwrap();
    editor.draw(&window);

    // 5. Assertions for continued scrolled state
    let (cury, curx) = window.get_cur_yx();
    assert_eq!(cury, STATUS_BAR_HEIGHT as i32, "Cursor Y should not change");
    assert_eq!(curx, expected_cursor_x as i32, "Cursor should remain pinned");

    // 6. Action: Move cursor left until scrolling stops
    // Current cursor_x is `move_count - 1`.
    // We need to move left until cursor_x becomes smaller than `expected_cursor_x`.
    let moves_to_unpin = (move_count - 1) - (expected_cursor_x - 1);
    for _ in 0..moves_to_unpin {
        editor.process_input(Input::KeyLeft, false).unwrap();
    }
    assert_eq!(editor.cursor_pos().0, expected_cursor_x - 1); // e.g. 29
    editor.draw(&window);

    // 7. Assertions for unscrolled state
    assert_eq!(editor.scroll.col_offset, 0, "col_offset should be 0");
    let (cury, curx) = window.get_cur_yx();
    assert_eq!(cury, STATUS_BAR_HEIGHT as i32);
    assert_eq!(curx, (expected_cursor_x - 1) as i32, "Cursor should now move freely");

    let mut displayed_line_unscrolled = String::new();
    for x in 0..screen_cols {
        let ch = window.mvinch(line_y, x as i32);
        displayed_line_unscrolled.push((ch & pancurses::A_CHARTEXT) as u8 as char);
    }
    let expected_line_slice_unscrolled = &editor.document.lines[0][0..screen_cols];
    assert_eq!(displayed_line_unscrolled, expected_line_slice_unscrolled);

    // 8. Teardown
    pancurses::endwin();
}