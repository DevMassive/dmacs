use dmacs::editor::{Editor, EditorMode};
use pancurses::Input;

#[test]
fn test_enter_and_exit_fuzzy_search_mode() {
    let mut editor = Editor::new(None);
    editor.document.lines = vec!["line one".to_string(), "line two".to_string()];

    // Enter fuzzy search mode
    editor
        .process_input(Input::Character('\x06'), false)
        .unwrap(); // Ctrl+F
    assert_eq!(editor.mode, EditorMode::FuzzySearch);

    // Exit with Esc
    editor
        .process_input(Input::Character('\x1b'), false)
        .unwrap(); // Esc
    assert_eq!(editor.mode, EditorMode::Normal);
}

#[test]
fn test_fuzzy_search_and_select() {
    let mut editor = Editor::new(None);
    editor.document.lines = vec![
        "apple".to_string(),
        "banana".to_string(),
        "apricot".to_string(),
    ];

    // Enter fuzzy search mode
    editor
        .process_input(Input::Character('\x06'), false)
        .unwrap(); // Ctrl+F

    // Type a query
    editor.process_input(Input::Character('a'), false).unwrap();
    editor.process_input(Input::Character('p'), false).unwrap();

    // Check matches
    assert_eq!(editor.fuzzy_search.matches.len(), 2);
    assert_eq!(editor.fuzzy_search.matches[0].0, "apple");
    assert_eq!(editor.fuzzy_search.matches[1].0, "apricot");

    // Select the second match
    editor.process_input(Input::KeyDown, false).unwrap();
    assert_eq!(editor.fuzzy_search.selected_index, 1);

    // Press Enter to jump to the line
    editor
        .process_input(Input::Character('\x0a'), false)
        .unwrap();

    // Check cursor position
    assert_eq!(editor.cursor_y, 2); // apricot is on line 2
    assert_eq!(editor.mode, EditorMode::Normal);
}

#[test]
fn test_fuzzy_search_navigation() {
    let mut editor = Editor::new(None);
    editor.document.lines = vec!["one".to_string(), "two".to_string(), "three".to_string()];

    // Enter fuzzy search mode
    editor
        .process_input(Input::Character('\x06'), false)
        .unwrap(); // Ctrl+F

    // Navigate down
    editor.process_input(Input::KeyDown, false).unwrap();
    assert_eq!(editor.fuzzy_search.selected_index, 1);

    editor.process_input(Input::KeyDown, false).unwrap();
    assert_eq!(editor.fuzzy_search.selected_index, 2);

    // Wrap around
    editor.process_input(Input::KeyDown, false).unwrap();
    assert_eq!(editor.fuzzy_search.selected_index, 0);

    // Navigate up
    editor.process_input(Input::KeyUp, false).unwrap();
    assert_eq!(editor.fuzzy_search.selected_index, 2);
}

#[test]
fn test_fuzzy_search_no_match() {
    let mut editor = Editor::new(None);
    editor.document.lines = vec!["abc".to_string(), "def".to_string()];

    // Enter fuzzy search mode
    editor
        .process_input(Input::Character('\x06'), false)
        .unwrap(); // Ctrl+F

    // Type a query with no match
    editor.process_input(Input::Character('x'), false).unwrap();
    editor.process_input(Input::Character('y'), false).unwrap();
    editor.process_input(Input::Character('z'), false).unwrap();

    assert!(editor.fuzzy_search.matches.is_empty());

    // Press Enter
    editor
        .process_input(Input::Character('\x0a'), false)
        .unwrap();

    // Cursor should not have moved
    assert_eq!(editor.cursor_y, 0);
    assert_eq!(editor.mode, EditorMode::Normal);
}

#[test]
fn test_fuzzy_search_backspace() {
    let mut editor = Editor::new(None);
    editor.document.lines = vec![
        "apple".to_string(),
        "banana".to_string(),
        "apricot".to_string(),
    ];

    // Enter fuzzy search mode
    editor
        .process_input(Input::Character('\x06'), false)
        .unwrap(); // Ctrl+F

    // Type a query
    editor.process_input(Input::Character('a'), false).unwrap();
    editor.process_input(Input::Character('p'), false).unwrap();
    assert_eq!(editor.fuzzy_search.query, "ap");
    assert_eq!(editor.fuzzy_search.matches.len(), 2);

    // Use backspace
    editor.process_input(Input::KeyBackspace, false).unwrap();
    assert_eq!(editor.fuzzy_search.query, "a");
    assert_eq!(editor.fuzzy_search.matches.len(), 3);
}

#[test]
fn test_fuzzy_search_reset() {
    let mut editor = Editor::new(None);
    editor.document.lines = vec![
        "apple".to_string(),
        "banana".to_string(),
        "apricot".to_string(),
    ];

    // Enter fuzzy search mode
    editor
        .process_input(Input::Character('\x06'), false)
        .unwrap(); // Ctrl+F

    // Type a query
    editor.process_input(Input::Character('a'), false).unwrap();
    assert!(!editor.fuzzy_search.query.is_empty());

    // Exit fuzzy search mode
    editor
        .process_input(Input::Character('\x1b'), false)
        .unwrap(); // Esc

    // Check that the search state is reset
    assert!(editor.fuzzy_search.query.is_empty());
    assert!(editor.fuzzy_search.matches.is_empty());
}
