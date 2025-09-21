use dmacs::config::Keymap;
use dmacs::editor::Editor;
use pancurses::Input;
use std::fs;
use tempfile::NamedTempFile;

#[test]
fn test_editor_initial_state_no_file() {
    let editor = Editor::new(None, None, None);
    assert!(!editor.should_quit);
    assert_eq!(editor.document.lines.len(), 1);
    assert_eq!(editor.document.lines[0], "");
}

#[test]
fn test_editor_with_wide_chars() {
    let mut editor = Editor::new(None, None, None);
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

#[test]
fn test_alt_s_saves_file() {
    let file = NamedTempFile::new().expect("Failed to create temp file");
    let path = file.path().to_path_buf();
    let initial_content = "Hello, world!";
    fs::write(&path, initial_content).expect("Failed to write to temp file");

    let mut editor = Editor::new(Some(path.to_str().unwrap().to_string()), None, None);

    // Insert some text
    editor.process_input(Input::Character('T'), false).unwrap();

    // Simulate Alt + S
    editor.process_input(Input::Character('s'), true).unwrap();

    // Read the file content and assert that the changes are saved
    let saved_content = fs::read_to_string(&path).expect("Failed to read saved file");
    assert_eq!(saved_content, "THello, world!\n");

    // Clean up the temporary file (done automatically by NamedTempFile drop)
}

#[test]
fn test_custom_keymap_overrides_default() {
    let mut editor = Editor::new(None, None, None);
    let custom_toml = r#"
        up = "Quit"
    "#;
    let keymap: Keymap = toml::from_str(custom_toml).unwrap();

    // The user config loader will extend the default map, so we simulate that.
    editor.keymap.bindings.extend(keymap.bindings);

    // Process the 'up' arrow key, which is now mapped to Quit
    editor.process_input(Input::KeyUp, false).unwrap();

    // Assert that the editor should quit
    assert!(editor.should_quit);

    // Let's also test that a default binding still works in a fresh editor
    let mut editor2 = Editor::new(None, None, None);
    // Process the 'down' arrow key, which should just move the cursor
    editor2.process_input(Input::KeyDown, false).unwrap();
    assert_eq!(editor2.cursor_pos(), (0, 0)); // Stays at 0,0 on a single line doc
    assert!(!editor2.should_quit);
}

#[test]
fn test_ctrl_x_no_exit_on_save() {
    let file = NamedTempFile::new().expect("Failed to create temp file");
    let path = file.path().to_path_buf();
    let initial_content = "Hello, world!";
    fs::write(&path, initial_content).expect("Failed to write to temp file");

    let mut editor = Editor::new(Some(path.to_str().unwrap().to_string()), None, None);
    editor.set_no_exit_on_save(true);

    // Insert some text
    editor.process_input(Input::Character('N'), false).unwrap();

    // Simulate Ctrl + X (save and exit, but with no_exit_on_save it should only save)
    editor
        .process_input(Input::Character('\x18'), false)
        .unwrap();

    // Assert that the file content is saved
    let saved_content = fs::read_to_string(&path).expect("Failed to read saved file");
    assert_eq!(saved_content, "NHello, world!\n");

    // Assert that the editor did NOT quit
    assert!(!editor.should_quit);

    // Clean up the temporary file (done automatically by NamedTempFile drop)
}
