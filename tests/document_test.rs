use dmacs::document::Document;
use std::fs;
use std::path::PathBuf;

// Helper function to create a temporary directory for tests
fn setup_test_env() -> PathBuf {
    let temp_dir = PathBuf::from(format!("/tmp/dmacs_test_{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(&temp_dir).expect("Failed to create temporary test directory");
    temp_dir
}

// Helper function to clean up the temporary directory
fn teardown_test_env(temp_dir: &PathBuf) {
    if temp_dir.exists() {
        fs::remove_dir_all(temp_dir).expect("Failed to remove temporary test directory");
    }
}

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
    let temp_dir = setup_test_env();
    let filename = temp_dir.join("test_save.txt");
    let mut doc = Document::new_empty();
    doc.filename = Some(filename.to_str().unwrap().to_string());
    doc.lines = vec!["line1".to_string(), "line2".to_string()];
    doc.save(Some(temp_dir.clone())).unwrap();

    let content = fs::read_to_string(&filename).unwrap();
    assert_eq!(content, "line1\nline2\n");

    teardown_test_env(&temp_dir);
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
    doc.lines = vec!["line1".to_string()];
    assert!(
        doc.is_dirty(),
        "Document should be dirty after modification."
    );

    fs::remove_file(filename).unwrap();
}

#[test]
fn test_is_dirty_after_save() {
    let temp_dir = setup_test_env();
    let filename = temp_dir.join("test_dirty_save.txt");
    let content = "line1\nline2\n";
    fs::write(&filename, content).unwrap();

    let mut doc = Document::open(filename.to_str().unwrap()).unwrap();
    doc.lines = vec!["line1".to_string()];
    assert!(doc.is_dirty(), "Document should be dirty before saving.");
    doc.save(Some(temp_dir.clone())).unwrap();
    assert!(
        !doc.is_dirty(),
        "Document should not be dirty after saving."
    );

    teardown_test_env(&temp_dir);
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
