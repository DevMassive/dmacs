use chrono::Local;
use dmacs::editor::Editor;

#[test]
fn test_today_command() {
    let mut editor = Editor::new(None, None, None);
    editor.insert_text("/today").unwrap();
    editor.insert_newline().unwrap();

    let expected_date = Local::now().format("%Y-%m-%d").to_string();
    assert_eq!(editor.document.lines.len(), 2);
    assert_eq!(editor.document.lines[0], expected_date);
    assert_eq!(editor.document.lines[1], "");
    assert_eq!(editor.status_message, "/today");
    assert_eq!(editor.cursor_y, 1);
    assert_eq!(editor.cursor_x, 0);
}

#[test]
fn test_now_command() {
    let mut editor = Editor::new(None, None, None);
    editor.insert_text("/now").unwrap();
    editor.insert_newline().unwrap();

    let expected_date = Local::now().format("%Y-%m-%d %H:%M").to_string();
    assert_eq!(editor.document.lines.len(), 2);
    assert_eq!(editor.document.lines[0], expected_date);
    assert_eq!(editor.document.lines[1], "");
    assert_eq!(editor.status_message, "/now");
    assert_eq!(editor.cursor_y, 1);
    assert_eq!(editor.cursor_x, 0);
}
