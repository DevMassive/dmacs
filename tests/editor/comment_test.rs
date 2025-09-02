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

fn simulate_alt_slash(editor: &mut Editor) {
    editor
        .process_input(Input::Character('/'), true)
        .unwrap();
}

#[test]
fn test_toggle_comment_on_single_line() {
    let mut editor = create_editor_with_content("hello world");
    editor.set_cursor_pos(5, 0);
    simulate_alt_slash(&mut editor);
    assert_eq!(editor.document.lines[0], "# hello world");
    assert_eq!(editor.cursor_pos(), (7, 0));
    assert_eq!(editor.status_message, "Commented line.");
}

#[test]
fn test_toggle_comment_off_single_line() {
    let mut editor = create_editor_with_content("# hello world");
    editor.set_cursor_pos(7, 0);
    simulate_alt_slash(&mut editor);
    assert_eq!(editor.document.lines[0], "hello world");
    assert_eq!(editor.cursor_pos(), (5, 0));
    assert_eq!(editor.status_message, "Uncommented line.");
}

#[test]
fn test_toggle_comment_on_indented_line() {
    let mut editor = create_editor_with_content("  hello world");
    editor.set_cursor_pos(7, 0);
    simulate_alt_slash(&mut editor);
    assert_eq!(editor.document.lines[0], "  # hello world");
    assert_eq!(editor.cursor_pos(), (9, 0));
}

#[test]
fn test_toggle_comment_off_indented_line() {
    let mut editor = create_editor_with_content("  # hello world");
    editor.set_cursor_pos(9, 0);
    simulate_alt_slash(&mut editor);
    assert_eq!(editor.document.lines[0], "  hello world");
    assert_eq!(editor.cursor_pos(), (7, 0));
}

#[test]
fn test_toggle_comment_on_selection() {
    let mut editor = create_editor_with_content("line1
line2
line3");
    editor.selection.set_marker((0, 0));
    editor.set_cursor_pos(5, 2);
    simulate_alt_slash(&mut editor);
    assert_eq!(
        editor.document.lines,
        vec!["# line1", "# line2", "# line3"]
    );
    assert!(editor.selection.is_selection_active());
}

#[test]
fn test_toggle_comment_off_selection() {
    let mut editor = create_editor_with_content("# line1
# line2
# line3");
    editor.selection.set_marker((0, 0));
    editor.set_cursor_pos(7, 2);
    simulate_alt_slash(&mut editor);
    assert_eq!(editor.document.lines, vec!["line1", "line2", "line3"]);
    assert!(editor.selection.is_selection_active());
}

#[test]
fn test_toggle_comment_on_mixed_selection() {
    let mut editor = create_editor_with_content("line1
# line2
line3");
    editor.selection.set_marker((0, 0));
    editor.set_cursor_pos(5, 2);
    simulate_alt_slash(&mut editor);
    assert_eq!(
        editor.document.lines,
        vec!["# line1", "# # line2", "# line3"]
    );
}

#[test]
fn test_toggle_comment_selection_ignores_empty_lines() {
    let mut editor = create_editor_with_content("line1

line3");
    editor.selection.set_marker((0, 0));
    editor.set_cursor_pos(5, 2);
    simulate_alt_slash(&mut editor);
    assert_eq!(editor.document.lines, vec!["# line1", "", "# line3"]);
}

#[test]
fn test_toggle_comment_selection_excludes_last_line_if_cursor_x_is_zero() {
    let mut editor = create_editor_with_content("line1
line2");
    editor.selection.set_marker((0, 0));
    editor.set_cursor_pos(0, 1);
    simulate_alt_slash(&mut editor);
    assert_eq!(editor.document.lines, vec!["# line1", "line2"]);
}

#[test]
fn test_toggle_comment_undo_redo() {
    let mut editor = create_editor_with_content("hello");
    let original_content = editor.document.lines.clone();
    let original_cursor = editor.cursor_pos();

    simulate_alt_slash(&mut editor);
    let commented_content = editor.document.lines.clone();
    let commented_cursor = editor.cursor_pos();
    assert_eq!(commented_content[0], "# hello");

    editor.undo();
    assert_eq!(editor.document.lines, original_content);
    assert_eq!(editor.cursor_pos(), original_cursor);

    editor.redo();
    assert_eq!(editor.document.lines, commented_content);
    assert_eq!(editor.cursor_pos(), commented_cursor);
}
