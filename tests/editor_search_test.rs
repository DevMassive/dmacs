use dmacs::editor::state::Editor;
use pancurses::Input;

#[test]
fn test_editor_search_mode_enter_and_exit() {
    let mut editor = Editor::new(None);
    editor.document.lines = vec!["test line one".to_string(), "test line two".to_string()];
    editor.set_message("Initial message.");

    // Enter search mode
    editor.process_input(Input::Character('\x13'), None, None); // Ctrl+S
    assert!(editor.search.mode);
    assert_eq!(editor.status_message, "Search: ");
    assert_eq!(editor.previous_status_message, "Initial message.");

    // Type a query
    editor.process_input(Input::Character('t'), None, None);
    assert_eq!(editor.search.query, "t");
    assert_eq!(editor.status_message, "Search: t");
    editor.process_input(Input::Character('e'), None, None);
    assert_eq!(editor.search.query, "te");
    assert_eq!(editor.status_message, "Search: te");

    // Exit with Enter
    editor.process_input(Input::Character('\n'), None, None);
    assert!(!editor.search.mode);
    assert_eq!(editor.status_message, "Initial message."); // Should restore previous message
    assert_eq!(editor.search.query, ""); // Query should be cleared
}

#[test]
fn test_editor_search_mode_escape_exit() {
    let mut editor = Editor::new(None);
    editor.document.lines = vec!["test line one".to_string()];
    editor.set_message("Another initial message.");

    // Enter search mode
    editor.process_input(Input::Character('\x13'), None, None); // Ctrl+S
    assert!(editor.search.mode);
    assert_eq!(editor.status_message, "Search: ");
    assert_eq!(editor.previous_status_message, "Another initial message.");

    // Type a query
    editor.process_input(Input::Character('e'), None, None);
    assert_eq!(editor.search.query, "e");
    assert_eq!(editor.status_message, "Search: e");

    // Exit with Escape
    editor.process_input(Input::Character('\x1b'), None, None); // Escape
    assert!(!editor.search.mode);
    assert_eq!(editor.status_message, "Another initial message."); // Should restore previous message
    assert_eq!(editor.search.query, ""); // Query should be cleared
}

#[test]
fn test_editor_search_next_and_previous_match() {
    let mut editor = Editor::new(None);
    editor.document.lines = vec![
        "apple banana apple".to_string(),
        "orange apple grape".to_string(),
    ];
    editor.set_message("Ready.");

    // Enter search mode and search for "apple"
    editor.process_input(Input::Character('\x13'), None, None); // Ctrl+S
    editor.process_input(Input::Character('a'), None, None);
    editor.process_input(Input::Character('p'), None, None);
    editor.process_input(Input::Character('p'), None, None);
    editor.process_input(Input::Character('l'), None, None);
    editor.process_input(Input::Character('e'), None, None);

    // Initial match should be the first "apple" (0,0)
    assert_eq!(editor.cursor_pos(), (0, 0));

    // Move to next match (Ctrl+S)
    editor.process_input(Input::Character('\x13'), None, None); // Ctrl+S
    assert_eq!(editor.cursor_pos(), (13, 0)); // Second "apple" on first line

    // Move to next match (Ctrl+S)
    editor.process_input(Input::Character('\x13'), None, None); // Ctrl+S
    assert_eq!(editor.cursor_pos(), (7, 1)); // First "apple" on second line

    // Move to next match (Ctrl+S) - should wrap around
    editor.process_input(Input::Character('\x13'), None, None); // Ctrl+S
    assert_eq!(editor.cursor_pos(), (0, 0)); // First "apple" on first line again

    // Move to previous match (Ctrl+R)
    editor.process_input(Input::Character('\x12'), None, None); // Ctrl+R
    assert_eq!(editor.cursor_pos(), (7, 1)); // First "apple" on second line

    // Move to previous match (Ctrl+R)
    editor.process_input(Input::Character('\x12'), None, None); // Ctrl+R
    assert_eq!(editor.cursor_pos(), (13, 0)); // Second "apple" on first line

    // Exit search mode
    editor.process_input(Input::Character('\n'), None, None); // Enter
    assert!(!editor.search.mode);
    assert_eq!(editor.status_message, "Ready.");
}

#[test]
fn test_editor_search_no_match() {
    let mut editor = Editor::new(None);
    editor.document.lines = vec!["line one".to_string(), "line two".to_string()];
    editor.set_message("Ready.");

    // Enter search mode and search for "xyz" (no match)
    editor.process_input(Input::Character('\x13'), None, None); // Ctrl+S
    editor.process_input(Input::Character('x'), None, None);
    editor.process_input(Input::Character('y'), None, None);
    editor.process_input(Input::Character('z'), None, None);

    assert_eq!(editor.search.query, "xyz");
    assert_eq!(editor.status_message, "Search: xyz (No match)");
    assert!(editor.search.results.is_empty());
    assert_eq!(editor.search.current_match_index, None);

    // Exit search mode
    editor.process_input(Input::Character('\n'), None, None); // Enter
    assert!(!editor.search.mode);
    assert_eq!(editor.status_message, "Ready.");
}

#[test]
fn test_editor_search_empty_query() {
    let mut editor = Editor::new(None);
    editor.document.lines = vec!["some text".to_string()];
    editor.set_message("Ready.");

    // Enter search mode
    editor.process_input(Input::Character('\x13'), None, None); // Ctrl+S
    assert_eq!(editor.status_message, "Search: ");

    // Backspace to empty query
    editor.process_input(Input::Character('e'), None, None);
    assert_eq!(editor.search.query, "e");
    assert_eq!(editor.status_message, "Search: e");
    editor.process_input(Input::Character('\x7f'), None, None); // Backspace
    assert_eq!(editor.search.query, "");
    assert_eq!(editor.status_message, "Search: ");
    assert!(editor.search.results.is_empty());
    assert_eq!(editor.search.current_match_index, None);

    // Exit search mode
    editor.process_input(Input::Character('\n'), None, None); // Enter
    assert!(!editor.search.mode);
    assert_eq!(editor.status_message, "Ready.");
}
