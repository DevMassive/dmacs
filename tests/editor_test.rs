use dmacs::{Document, Editor};
use pancurses::Input;
use std::fs;

#[test]
fn test_open_document() {
    let filename = "test_doc.txt";
    fs::write(filename, "hello\nworld").unwrap();

    let doc = Document::open(filename).unwrap();
    assert_eq!(doc.lines.len(), 2);
    assert_eq!(doc.lines[0], "hello");
    assert_eq!(doc.lines[1], "world");

    fs::remove_file(filename).unwrap();
}

#[test]
fn test_editor_initial_state_no_file() {
    let editor = Editor::new(None);
    assert!(!editor.should_quit);
    assert_eq!(editor.document.lines.len(), 1);
    assert_eq!(editor.document.lines[0], "");
}

#[test]
fn test_document_save() {
    let filename = "test_save.txt";
    let mut doc = Document::default();
    doc.filename = Some(filename.to_string());
    doc.lines = vec!["line1".to_string(), "line2".to_string()];
    doc.save().unwrap();

    let content = fs::read_to_string(filename).unwrap();
    assert_eq!(content, "line1\nline2\n");

    fs::remove_file(filename).unwrap();
}

#[test]
fn test_document_insert() {
    let mut doc = Document::default();
    doc.insert(0, 0, 'h');
    doc.insert(1, 0, 'i');
    assert_eq!(doc.lines[0], "hi");
}

#[test]
fn test_document_delete() {
    let mut doc = Document::default();
    doc.insert(0, 0, 'h');
    doc.insert(1, 0, 'i');
    doc.delete(0, 0);
    assert_eq!(doc.lines[0], "i");
}

#[test]
fn test_insert_newline() {
    let mut doc = Document::default();
    doc.lines[0] = "abcdef".to_string();
    doc.insert_newline(3, 0);
    assert_eq!(doc.lines.len(), 2);
    assert_eq!(doc.lines[0], "abc");
    assert_eq!(doc.lines[1], "def");
}

#[test]
fn test_editor_move_cursor() {
    let mut editor = Editor::new(None);
    editor.document.lines = vec!["one".to_string(), "two".to_string()];
    editor.handle_keypress(Input::KeyRight);
    assert_eq!(editor.cursor_pos(), (1, 0));
    editor.handle_keypress(Input::KeyDown);
    assert_eq!(editor.cursor_pos(), (1, 1));
    editor.handle_keypress(Input::KeyLeft);
    assert_eq!(editor.cursor_pos(), (0, 1));
    editor.handle_keypress(Input::KeyUp);
    assert_eq!(editor.cursor_pos(), (0, 0));
}

#[test]
fn test_editor_insert_char() {
    let mut editor = Editor::new(None);
    editor.handle_keypress(Input::Character('a'));
    assert_eq!(editor.document.lines[0], "a");
    assert_eq!(editor.cursor_pos(), (1, 0));
}

#[test]
fn test_editor_delete_char() {
    let mut editor = Editor::new(None);
    editor.handle_keypress(Input::Character('a'));
    editor.handle_keypress(Input::KeyBackspace);
    assert_eq!(editor.document.lines[0], "");
    assert_eq!(editor.cursor_pos(), (0, 0));
}

#[test]
fn test_editor_delete_forward_char() {
    let mut editor = Editor::new(None);
    editor.handle_keypress(Input::Character('a'));
    editor.handle_keypress(Input::KeyLeft);
    editor.handle_keypress(Input::Character('\x04')); // Ctrl-D
    assert_eq!(editor.document.lines[0], "");
    assert_eq!(editor.cursor_pos(), (0, 0));
}

#[test]
fn test_editor_insert_newline() {
    let mut editor = Editor::new(None);
    editor.handle_keypress(Input::Character('a'));
    editor.handle_keypress(Input::Character('\x0A'));
    assert_eq!(editor.document.lines.len(), 2);
    assert_eq!(editor.document.lines[0], "a");
    assert_eq!(editor.document.lines[1], "");
    assert_eq!(editor.cursor_pos(), (0, 1));
}

#[test]
fn test_go_to_line_boundaries() {
    let mut editor = Editor::new(None);
    editor.document.lines[0] = "hello".to_string();
    editor.handle_keypress(Input::KeyRight);
    editor.handle_keypress(Input::KeyRight);
    assert_eq!(editor.cursor_pos(), (2, 0));
    editor.handle_keypress(Input::Character('\x01')); // Ctrl-A
    assert_eq!(editor.cursor_pos(), (0, 0));
    editor.handle_keypress(Input::Character('\x05')); // Ctrl-E
    assert_eq!(editor.cursor_pos(), (5, 0));
}

#[test]
fn test_editor_backspace_line_join() {
    let mut editor = Editor::new(None);
    editor.document.lines = vec!["hello".to_string(), "world".to_string()];
    editor.handle_keypress(Input::KeyDown);
    editor.handle_keypress(Input::KeyLeft);
    editor.handle_keypress(Input::KeyLeft);
    editor.handle_keypress(Input::KeyLeft);
    editor.handle_keypress(Input::KeyLeft);
    editor.handle_keypress(Input::KeyLeft);
    editor.handle_keypress(Input::KeyBackspace);
    assert_eq!(editor.document.lines.len(), 1);
    assert_eq!(editor.document.lines[0], "helloworld");
    assert_eq!(editor.cursor_pos(), (5, 0));
}

#[test]
fn test_editor_delete_to_end_of_line() {
    let mut editor = Editor::new(None);
    editor.document.lines[0] = "hello world".to_string();
    editor.handle_keypress(Input::KeyRight);
    editor.handle_keypress(Input::KeyRight);
    editor.handle_keypress(Input::KeyRight);
    editor.handle_keypress(Input::KeyRight);
    editor.handle_keypress(Input::KeyRight);
    editor.handle_keypress(Input::Character('\x0b')); // Ctrl-K
    assert_eq!(editor.document.lines[0], "hello");
    assert_eq!(editor.cursor_pos(), (5, 0));
}

#[test]
fn test_editor_delete_to_end_of_line_at_end() {
    let mut editor = Editor::new(None);
    editor.document.lines = vec!["hello".to_string(), "world".to_string()];
    editor.handle_keypress(Input::Character('\x05')); // Ctrl-E
    editor.handle_keypress(Input::Character('\x0b')); // Ctrl-K
    assert_eq!(editor.document.lines.len(), 1);
    assert_eq!(editor.document.lines[0], "helloworld");
    assert_eq!(editor.cursor_pos(), (5, 0));
}

#[test]
fn test_editor_del_key() {
    let mut editor = Editor::new(None);
    editor.handle_keypress(Input::Character('a'));
    editor.handle_keypress(Input::Character('\x7f'));
    assert_eq!(editor.document.lines[0], "");
    assert_eq!(editor.cursor_pos(), (0, 0));
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
    assert_eq!(editor.document.lines[0], "\ta");
    // display width of tab is 8, plus 'a' is 1 = 9
    // cursor byte position is 1 (tab) + 1 (a) = 2
    assert_eq!(editor.cursor_pos(), (2, 0)); 
}