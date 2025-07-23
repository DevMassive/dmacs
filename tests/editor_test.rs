use dmacs::{Document, Editor};
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