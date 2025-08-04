use dmacs::editor::Editor;
use pancurses::Input;

#[test]
fn test_editor_search_mode_enter_and_exit() {
    let mut editor = Editor::new(None);
    editor.document.lines = vec!["test line one".to_string(), "test line two".to_string()];

    // Enter search mode
    editor
        .process_input(Input::Character('\x13'), false)
        .unwrap(); // Ctrl+S
    assert!(editor.search.mode);
    assert_eq!(editor.status_message, "Search: ");

    // Type a query
    editor.process_input(Input::Character('t'), false).unwrap();
    assert_eq!(editor.search.query, "t");
    assert_eq!(editor.status_message, "Search: t");
    editor.process_input(Input::Character('e'), false).unwrap();
    assert_eq!(editor.search.query, "te");
    assert_eq!(editor.status_message, "Search: te");

    // Exit with Enter
    editor.process_input(Input::Character('\n'), false).unwrap();
    assert!(!editor.search.mode);
    assert_eq!(editor.status_message, ""); // Should be empty
    assert_eq!(editor.search.query, ""); // Query should be cleared
}

#[test]
fn test_editor_search_mode_escape_exit() {
    let mut editor = Editor::new(None);
    editor.document.lines = vec!["test line one".to_string()];

    // Enter search mode
    editor
        .process_input(Input::Character('\x13'), false)
        .unwrap(); // Ctrl+S
    assert!(editor.search.mode);
    assert_eq!(editor.status_message, "Search: ");

    // Type a query
    editor.process_input(Input::Character('e'), false).unwrap();
    assert_eq!(editor.search.query, "e");
    assert_eq!(editor.status_message, "Search: e");

    // Exit with Escape
    editor
        .process_input(Input::Character('\x1b'), false)
        .unwrap(); // Escape
    assert!(!editor.search.mode);
    assert_eq!(editor.status_message, ""); // Should be empty
    assert_eq!(editor.search.query, ""); // Query should be cleared
}

#[test]
fn test_editor_search_next_and_previous_match() {
    let mut editor = Editor::new(None);
    editor.document.lines = vec![
        "apple banana apple".to_string(),
        "orange apple grape".to_string(),
    ];

    // Enter search mode and search for "apple"
    editor
        .process_input(Input::Character('\x13'), false)
        .unwrap(); // Ctrl+S
    editor.process_input(Input::Character('a'), false).unwrap();
    editor.process_input(Input::Character('p'), false).unwrap();
    editor.process_input(Input::Character('p'), false).unwrap();
    editor.process_input(Input::Character('l'), false).unwrap();
    editor.process_input(Input::Character('e'), false).unwrap();

    // Initial match should be the first "apple" (0,0)
    assert_eq!(editor.cursor_pos(), (0, 0));

    // Move to next match (Ctrl+S)
    editor
        .process_input(Input::Character('\x13'), false)
        .unwrap(); // Ctrl+S
    assert_eq!(editor.cursor_pos(), (13, 0)); // Second "apple" on first line

    // Move to next match (Ctrl+S)
    editor
        .process_input(Input::Character('\x13'), false)
        .unwrap(); // Ctrl+S
    assert_eq!(editor.cursor_pos(), (7, 1)); // First "apple" on second line

    // Move to next match (Ctrl+S) - should wrap around
    editor
        .process_input(Input::Character('\x13'), false)
        .unwrap(); // Ctrl+S
    assert_eq!(editor.cursor_pos(), (0, 0)); // First "apple" on first line again

    // Move to previous match (Ctrl+R)
    editor
        .process_input(Input::Character('\x12'), false)
        .unwrap(); // Ctrl+R
    assert_eq!(editor.cursor_pos(), (7, 1)); // First "apple" on second line

    // Move to previous match (Ctrl+R)
    editor
        .process_input(Input::Character('\x12'), false)
        .unwrap(); // Ctrl+R
    assert_eq!(editor.cursor_pos(), (13, 0)); // Second "apple" on first line

    // Exit search mode
    editor.process_input(Input::Character('\n'), false).unwrap(); // Enter
    assert!(!editor.search.mode);
    assert_eq!(editor.status_message, "");
}

#[test]
fn test_editor_search_no_match() {
    let mut editor = Editor::new(None);
    editor.document.lines = vec!["line one".to_string(), "line two".to_string()];

    // Enter search mode and search for "xyz" (no match)
    editor
        .process_input(Input::Character('\x13'), false)
        .unwrap(); // Ctrl+S
    editor.process_input(Input::Character('x'), false).unwrap();
    editor.process_input(Input::Character('y'), false).unwrap();
    editor.process_input(Input::Character('z'), false).unwrap();

    assert_eq!(editor.search.query, "xyz");
    assert_eq!(editor.status_message, "Search: xyz (No match)");
    assert!(editor.search.results.is_empty());
    assert_eq!(editor.search.current_match_index, None);

    // Exit search mode
    editor.process_input(Input::Character('\n'), false).unwrap(); // Enter
    assert!(!editor.search.mode);
    assert_eq!(editor.status_message, "");
}

#[test]
fn test_editor_search_empty_query() {
    let mut editor = Editor::new(None);
    editor.document.lines = vec!["some text".to_string()];

    // Enter search mode
    editor
        .process_input(Input::Character('\x13'), false)
        .unwrap(); // Ctrl+S
    assert_eq!(editor.status_message, "Search: ");

    // Backspace to empty query
    editor.process_input(Input::Character('e'), false).unwrap();
    assert_eq!(editor.search.query, "e");
    assert_eq!(editor.status_message, "Search: e");
    editor
        .process_input(Input::Character('\x7f'), false)
        .unwrap();
    assert_eq!(editor.search.query, "");
    assert_eq!(editor.status_message, "Search: ");
    assert!(editor.search.results.is_empty());
    assert_eq!(editor.search.current_match_index, None);

    // Exit search mode
    editor.process_input(Input::Character('\n'), false).unwrap(); // Enter
    assert!(!editor.search.mode);
    assert_eq!(editor.status_message, "");
}
