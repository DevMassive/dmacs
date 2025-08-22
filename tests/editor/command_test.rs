use chrono::Local;
use dmacs::editor::Editor;

#[test]
fn test_today_command() {
    let mut editor = Editor::new(None);
    editor.insert_text("/today").unwrap();
    editor.insert_newline().unwrap();

    let expected_date = Local::now().format("%Y-%m-%d").to_string();
    assert_eq!(editor.document.lines[0], expected_date);
    assert_eq!(editor.status_message, "/today");
}
