use dmacs::editor::Editor;
use pancurses::Input;

fn create_editor_with_content(content: &str) -> Editor {
    let mut editor = Editor::new(None);
    editor.document.lines = content.lines().map(|s| s.to_string()).collect();
    if editor.document.lines.is_empty() {
        editor.document.lines.push(String::new());
    }
    editor
}

#[test]
fn test_indent_single_line() {
    let mut editor = create_editor_with_content("hello\nworld");
    editor.cursor_y = 0;
    editor.cursor_x = 2;
    editor.indent_line().unwrap();
    assert_eq!(editor.document.lines, vec!["  hello", "world"]);
    assert_eq!(editor.cursor_x, 4);
}

#[test]
fn test_outdent_single_line() {
    let mut editor = create_editor_with_content("  hello\nworld");
    editor.cursor_y = 0;
    editor.cursor_x = 4;
    editor.outdent_line().unwrap();
    assert_eq!(editor.document.lines, vec!["hello", "world"]);
    assert_eq!(editor.cursor_x, 2);
}

#[test]
fn test_indent_selection() {
    let mut editor = create_editor_with_content("line1\nline2\nline3");
    editor.selection.set_marker((0, 0)); // Start of line1
    editor.set_cursor_pos(5, 2); // End of line3
    editor.indent_line().unwrap();
    assert_eq!(editor.document.lines, vec!["  line1", "  line2", "  line3"]);
    assert!(editor.selection.is_selection_active());
    assert_eq!(editor.cursor_pos(), (7, 2)); // 5 + 2
    assert_eq!(editor.selection.marker_pos, Some((2, 0))); // 0 + 2
}

#[test]
fn test_outdent_selection() {
    let mut editor = create_editor_with_content("  line1\n  line2\n  line3");
    editor.selection.set_marker((0, 0));
    editor.set_cursor_pos(7, 2);
    editor.outdent_line().unwrap();
    assert_eq!(editor.document.lines, vec!["line1", "line2", "line3"]);
    assert!(editor.selection.is_selection_active());
    assert_eq!(editor.cursor_pos(), (5, 2)); // 7 - 2
    assert_eq!(editor.selection.marker_pos, Some((0, 0))); // 2 - 2
}

#[test]
fn test_indent_selection_skips_empty_line() {
    let mut editor = create_editor_with_content("line1\n\nline3");
    editor.selection.set_marker((0, 0));
    editor.set_cursor_pos(5, 2);
    editor.indent_line().unwrap();
    assert_eq!(editor.document.lines, vec!["  line1", "", "  line3"]);
}

#[test]
fn test_indent_selection_skips_last_line_if_cursor_at_x0() {
    let mut editor = create_editor_with_content("line1\nline2\nline3");
    editor.selection.set_marker((0, 0));
    editor.set_cursor_pos(0, 2); // End selection at start of line3
    editor.indent_line().unwrap();
    assert_eq!(editor.document.lines, vec!["  line1", "  line2", "line3"]);
}

#[test]
fn test_outdent_selection_skips_last_line_if_cursor_at_x0() {
    let mut editor = create_editor_with_content("  line1\n  line2\n  line3");
    editor.selection.set_marker((0, 0));
    editor.set_cursor_pos(0, 2);
    editor.outdent_line().unwrap();
    assert_eq!(editor.document.lines, vec!["line1", "line2", "  line3"]);
}

#[test]
fn test_undo_indent_selection() {
    let mut editor = create_editor_with_content("line1\nline2");
    editor.selection.set_marker((0, 0));
    editor.set_cursor_pos(5, 1);
    editor.indent_line().unwrap();
    assert_eq!(editor.document.lines, vec!["  line1", "  line2"]);
    editor.undo();
    assert_eq!(editor.document.lines, vec!["line1", "line2"]);
}

#[test]
fn test_undo_outdent_selection() {
    let mut editor = create_editor_with_content("  line1\n  line2");
    editor.selection.set_marker((0, 0));
    editor.set_cursor_pos(7, 1);
    editor.outdent_line().unwrap();
    assert_eq!(editor.document.lines, vec!["line1", "line2"]);
    editor.undo();
    assert_eq!(editor.document.lines, vec!["  line1", "  line2"]);
}

#[test]
fn test_indent_on_tab_press() {
    let mut editor = create_editor_with_content("");
    editor.process_input(Input::Character('\t'), false).unwrap();
    editor.process_input(Input::Character('a'), false).unwrap();
    assert_eq!(editor.document.lines[0], "  a");
    assert_eq!(editor.cursor_pos(), (3, 0));
}

#[test]
fn test_outdent_on_shift_tab_press() {
    let mut editor = create_editor_with_content("  a");
    editor.set_cursor_pos(3, 0);
    editor.process_input(Input::KeySTab, false).unwrap();
    assert_eq!(editor.document.lines[0], "a");
    assert_eq!(editor.cursor_pos(), (1, 0));
}

#[test]
fn test_outdent_with_one_space() {
    let mut editor = create_editor_with_content(" a");
    editor.set_cursor_pos(2, 0);
    editor.process_input(Input::KeySTab, false).unwrap();
    assert_eq!(editor.document.lines[0], "a");
    assert_eq!(editor.cursor_pos(), (1, 0));
}

#[test]
fn test_outdent_with_no_space() {
    let mut editor = create_editor_with_content("a");
    editor.set_cursor_pos(1, 0);
    editor.process_input(Input::KeySTab, false).unwrap();
    assert_eq!(editor.document.lines[0], "a");
    assert_eq!(editor.cursor_pos(), (1, 0));
}

#[test]
fn test_indent_cursor_position() {
    let mut editor = create_editor_with_content("a");
    editor.set_cursor_pos(1, 0);
    editor.process_input(Input::Character('\t'), false).unwrap();
    assert_eq!(editor.document.lines[0], "  a");
    assert_eq!(editor.cursor_pos(), (3, 0));
}
