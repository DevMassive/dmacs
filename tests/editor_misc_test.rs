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
    assert_eq!(editor.document.lines[0], "	a");
    // display width of tab is 8, plus 'a' is 1 = 9
    // cursor byte position is 1 (tab) + 1 (a) = 2
    assert_eq!(editor.cursor_pos(), (2, 0));
}
