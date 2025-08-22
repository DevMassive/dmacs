use dmacs::editor::Editor;
use pancurses::Input;

fn simulate_ctrl_t(editor: &mut Editor) {
    editor
        .process_input(Input::Character('\x14'), false)
        .unwrap();
}

#[test]
fn test_toggle_checkbox_add() {
    let mut editor = Editor::new(None);
    editor.insert_text("Hello world").unwrap();
    editor.go_to_start_of_line();
    simulate_ctrl_t(&mut editor);
    assert_eq!(editor.document.lines[0], "- [ ] Hello world");
    assert_eq!(editor.cursor_pos(), (6, 0));
    assert_eq!(editor.status_message, "Checkbox added.");
}

#[test]
fn test_toggle_checkbox_check() {
    let mut editor = Editor::new(None);
    editor.insert_text("- [ ] Hello world").unwrap();
    editor.go_to_start_of_line();
    simulate_ctrl_t(&mut editor);
    assert_eq!(editor.document.lines[0], "- [x] Hello world");
    assert_eq!(editor.cursor_pos(), (0, 0));
    assert_eq!(editor.status_message, "Checkbox checked.");
}

#[test]
fn test_toggle_checkbox_uncheck() {
    let mut editor = Editor::new(None);
    editor.insert_text("- [x] Hello world").unwrap();
    editor.go_to_start_of_line();
    simulate_ctrl_t(&mut editor);
    assert_eq!(editor.document.lines[0], "Hello world");
    assert_eq!(editor.cursor_pos(), (0, 0));
    assert_eq!(editor.status_message, "Checkbox removed.");
}

#[test]
fn test_toggle_checkbox_undo_redo() {
    let mut editor = Editor::new(None);
    editor.insert_text("Hello world").unwrap();
    editor.go_to_start_of_line();

    // Add checkbox
    simulate_ctrl_t(&mut editor);
    assert_eq!(editor.document.lines[0], "- [ ] Hello world");
    assert_eq!(editor.status_message, "Checkbox added.");

    // Check it
    simulate_ctrl_t(&mut editor);
    assert_eq!(editor.document.lines[0], "- [x] Hello world");
    assert_eq!(editor.status_message, "Checkbox checked.");

    // Uncheck it
    simulate_ctrl_t(&mut editor);
    assert_eq!(editor.document.lines[0], "Hello world");
    assert_eq!(editor.status_message, "Checkbox removed.");

    // Undo uncheck
    editor.undo();
    assert_eq!(editor.document.lines[0], "- [x] Hello world");

    // Undo check
    editor.undo();
    assert_eq!(editor.document.lines[0], "- [ ] Hello world");

    // Undo add
    editor.undo();
    assert_eq!(editor.document.lines[0], "Hello world");

    // Redo add
    editor.redo();
    assert_eq!(editor.document.lines[0], "- [ ] Hello world");

    // Redo check
    editor.redo();
    assert_eq!(editor.document.lines[0], "- [x] Hello world");

    // Redo uncheck
    editor.redo();
    assert_eq!(editor.document.lines[0], "Hello world");
}
