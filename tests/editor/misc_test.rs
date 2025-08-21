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
fn test_editor_with_wide_chars() {
    let mut editor = Editor::new(None);
    editor.process_input(Input::Character('あ'), false).unwrap();
    editor.process_input(Input::Character('い'), false).unwrap();
    assert_eq!(editor.document.lines[0], "あい");
    assert_eq!(editor.cursor_pos(), (6, 0)); // "あ" and "い" are 3 bytes each
    editor.process_input(Input::KeyLeft, false).unwrap();
    assert_eq!(editor.cursor_pos(), (3, 0));
    editor.process_input(Input::KeyBackspace, false).unwrap();
    assert_eq!(editor.document.lines[0], "い");
    assert_eq!(editor.cursor_pos(), (0, 0));
}

#[test]
fn test_editor_with_tabs() {
    let mut editor = Editor::new(None);
    editor.process_input(Input::Character('\t'), false).unwrap();
    editor.process_input(Input::Character('a'), false).unwrap();
    assert_eq!(editor.document.lines[0], "	a");
    // display width of tab is 8, plus 'a' is 1 = 9
    // cursor byte position is 1 (tab) + 1 (a) = 2
    assert_eq!(editor.cursor_pos(), (2, 0));
}

#[test]
fn test_is_separator_line() {
    // Test exact match
    assert!(Editor::is_separator_line("---"));

    // Test with leading/trailing whitespace (should be false)
    assert!(!Editor::is_separator_line(" ---"));
    assert!(!Editor::is_separator_line("--- "));
    assert!(!Editor::is_separator_line(" ---"));

    // Test with other characters
    assert!(!Editor::is_separator_line("-----"));
    assert!(!Editor::is_separator_line("--"));
    assert!(!Editor::is_separator_line("abc---"));
    assert!(!Editor::is_separator_line("---abc"));
    assert!(!Editor::is_separator_line(""));
    assert!(!Editor::is_separator_line("hello"));
}

#[test]
fn test_is_unchecked_checkbox() {
    assert!(Editor::is_unchecked_checkbox("- [ ] task"));
    assert!(Editor::is_unchecked_checkbox("  - [ ] task"));
    assert!(!Editor::is_unchecked_checkbox("- [] task"));
    assert!(!Editor::is_unchecked_checkbox("- [x] task"));
    assert!(!Editor::is_unchecked_checkbox("task - [ ]"));
}

#[test]
fn test_is_checked_checkbox() {
    assert!(Editor::is_checked_checkbox("- [x] task"));
    assert!(Editor::is_checked_checkbox("  - [x] task"));
    assert!(!Editor::is_checked_checkbox("- [] task"));
    assert!(!Editor::is_checked_checkbox("- [ ] task"));
    assert!(!Editor::is_checked_checkbox("task - [x]"));
}
