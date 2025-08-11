use dmacs::document::Document;
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
fn test_document_save() {
    let filename = "test_save.txt";
    let mut doc = Document::new_empty();
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
    doc.insert(0, 0, 'h').unwrap();
    doc.insert(1, 0, 'i').unwrap();
    assert_eq!(doc.lines[0], "hi");
}

#[test]
fn test_document_delete() {
    let mut doc = Document::default();
    doc.insert(0, 0, 'h').unwrap();
    doc.insert(1, 0, 'i').unwrap();
    doc.delete(0, 0).unwrap();
    assert_eq!(doc.lines[0], "i");
}

#[test]
fn test_insert_newline() {
    let mut doc = Document::default();
    doc.lines[0] = "abcdef".to_string();
    doc.insert_newline(3, 0, false).unwrap();
    assert_eq!(doc.lines.len(), 2);
    assert_eq!(doc.lines[0], "abc");
    assert_eq!(doc.lines[1], "def");
}

#[test]
fn test_document_insert_string() {
    let mut doc = Document::default();
    doc.lines[0] = "hello".to_string();
    doc.insert_string(2, 0, "X").unwrap();
    assert_eq!(doc.lines[0], "heXllo");

    doc.insert_string(0, 0, "YY").unwrap();
    assert_eq!(doc.lines[0], "YYheXllo");

    doc.insert_string(doc.lines[0].len(), 0, "ZZ").unwrap();
    assert_eq!(doc.lines[0], "YYheXlloZZ");

    // Test inserting into an empty document
    let mut doc2 = Document::default();
    doc2.insert_string(0, 0, "test").unwrap();
    assert_eq!(doc2.lines[0], "test");

    // Test inserting at an invalid line index (should do nothing)
    let mut doc3 = Document::default();
    doc3.lines[0] = "line".to_string();
    assert!(doc3.insert_string(0, 1, "invalid").is_err());
    assert_eq!(doc3.lines[0], "line");
}

#[test]
fn test_document_swap_lines() {
    let mut doc = Document::new_empty();
    doc.lines = vec![
        "line1".to_string(),
        "line2".to_string(),
        "line3".to_string(),
    ];

    // Swap line1 and line2
    doc.swap_lines(0, 1);
    assert_eq!(doc.lines[0], "line2");
    assert_eq!(doc.lines[1], "line1");
    assert_eq!(doc.lines[2], "line3");

    // Swap line1 (now at index 1) and line3
    doc.swap_lines(1, 2);
    assert_eq!(doc.lines[0], "line2");
    assert_eq!(doc.lines[1], "line3");
    assert_eq!(doc.lines[2], "line1");

    // Try swapping out of bounds (should do nothing)
    doc.swap_lines(0, 100);
    assert_eq!(doc.lines[0], "line2");
    assert_eq!(doc.lines[1], "line3");
    assert_eq!(doc.lines[2], "line1");

    doc.swap_lines(100, 0);
    assert_eq!(doc.lines[0], "line2");
    assert_eq!(doc.lines[1], "line3");
    assert_eq!(doc.lines[2], "line1");
}

#[test]
fn test_is_dirty_after_opening_file() {
    let filename = "test_dirty_check.txt";
    let content = "line1\nline2\n";
    fs::write(filename, content).unwrap();

    let doc = Document::open(filename).unwrap();
    assert!(
        !doc.is_dirty(),
        "Document should not be dirty after opening a clean file."
    );

    fs::remove_file(filename).unwrap();
}

#[test]
fn test_is_dirty_after_modification() {
    let filename = "test_dirty_modification.txt";
    let content = "line1\nline2\n";
    fs::write(filename, content).unwrap();

    let mut doc = Document::open(filename).unwrap();
    doc.insert(0, 0, 'X').unwrap(); // Modify the document
    assert!(
        doc.is_dirty(),
        "Document should be dirty after modification."
    );

    fs::remove_file(filename).unwrap();
}

#[test]
fn test_is_dirty_after_save() {
    let filename = "test_dirty_save.txt";
    let content = "line1\nline2\n";
    fs::write(filename, content).unwrap();

    let mut doc = Document::open(filename).unwrap();
    doc.insert(0, 0, 'X').unwrap(); // Modify the document
    assert!(doc.is_dirty(), "Document should be dirty before saving.");
    doc.save().unwrap();
    assert!(
        !doc.is_dirty(),
        "Document should not be dirty after saving."
    );

    fs::remove_file(filename).unwrap();
}

#[test]
fn test_is_dirty_new_file() {
    let doc = Document::new_empty();
    assert!(doc.is_dirty(), "New document should be dirty.");
}

#[test]
fn test_is_dirty_after_opening_file_no_trailing_newline() {
    let filename = "test_dirty_check_no_newline.txt";
    let content = "line1\nline2";
    fs::write(filename, content).unwrap();

    let doc = Document::open(filename).unwrap();
    assert!(
        !doc.is_dirty(),
        "Document should not be dirty after opening a clean file with no trailing newline."
    );

    fs::remove_file(filename).unwrap();
}

#[test]
fn test_document_insert_string_with_newlines() {
    let mut doc = Document::default();
    doc.lines[0] = "start".to_string();
    doc.insert_string(5, 0, "a\nbc").unwrap();
    assert_eq!(doc.lines.len(), 2);
    assert_eq!(doc.lines[0], "starta");
    assert_eq!(doc.lines[1], "bc");

    let mut doc2 = Document::default();
    doc2.lines[0] = "line1".to_string();
    doc2.lines.push("line2".to_string());
    doc2.insert_string(2, 0, "X\nY").unwrap(); // Insert in the middle of line1
    assert_eq!(doc2.lines.len(), 3);
    assert_eq!(doc2.lines[0], "liX");
    assert_eq!(doc2.lines[1], "Yne1");
    assert_eq!(doc2.lines[2], "line2");
}
