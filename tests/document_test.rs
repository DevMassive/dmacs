use dmacs::document::{ActionDiff, Diff, Document};
use dmacs::error::Result;
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

// Helper function for tests
fn insert_string_via_action_diff(
    doc: &mut Document,
    x: usize,
    y: usize,
    s: &str,
) -> Result<(usize, usize)> {
    let mut current_x = x;
    let mut current_y = y;
    for c in s.chars() {
        let (new_x, new_y) = if c == '\n' {
            doc.apply_action_diff(
                &ActionDiff::NewlineInsertion {
                    x: current_x,
                    y: current_y,
                },
                false,
            )?
        } else {
            doc.apply_action_diff(
                &ActionDiff::CharChange(Diff {
                    x: current_x,
                    y: current_y,
                    added_text: c.to_string(),
                    deleted_text: "".to_string(),
                }),
                false,
            )?
        };
        current_x = new_x;
        current_y = new_y;
    }
    Ok((current_x, current_y))
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
fn test_document_insert() {
    let mut doc = Document::default();
    doc.apply_action_diff(
        &ActionDiff::CharChange(Diff {
            x: 0,
            y: 0,
            added_text: 'h'.to_string(),
            deleted_text: "".to_string(),
        }),
        false,
    )
    .unwrap();
    doc.apply_action_diff(
        &ActionDiff::CharChange(Diff {
            x: 1,
            y: 0,
            added_text: 'i'.to_string(),
            deleted_text: "".to_string(),
        }),
        false,
    )
    .unwrap();
    assert_eq!(doc.lines[0], "hi");
}

#[test]
fn test_document_delete() {
    let mut doc = Document::default();
    doc.apply_action_diff(
        &ActionDiff::CharChange(Diff {
            x: 0,
            y: 0,
            added_text: 'h'.to_string(),
            deleted_text: "".to_string(),
        }),
        false,
    )
    .unwrap();
    doc.apply_action_diff(
        &ActionDiff::CharChange(Diff {
            x: 1,
            y: 0,
            added_text: 'i'.to_string(),
            deleted_text: "".to_string(),
        }),
        false,
    )
    .unwrap();
    // Before deleting, the line is "hi". Deleting 'h' at (0,0)
    let char_to_delete = doc.lines[0].chars().next().unwrap().to_string(); // This is 'h'
    doc.apply_action_diff(
        &ActionDiff::CharChange(Diff {
            x: 0,
            y: 0,
            added_text: "".to_string(),
            deleted_text: char_to_delete,
        }),
        false,
    )
    .unwrap();
    assert_eq!(doc.lines[0], "i");
}

#[test]
fn test_insert_newline() {
    let mut doc = Document::default();
    doc.lines[0] = "abcdef".to_string();
    doc.apply_action_diff(&ActionDiff::NewlineInsertion { x: 3, y: 0 }, false)
        .unwrap();
    assert_eq!(doc.lines.len(), 2);
    assert_eq!(doc.lines[0], "abc");
    assert_eq!(doc.lines[1], "def");
}

#[test]
fn test_document_insert_string() {
    let mut doc = Document::default();
    doc.lines[0] = "hello".to_string();
    insert_string_via_action_diff(&mut doc, 2, 0, "X").unwrap();
    assert_eq!(doc.lines[0], "heXllo");

    insert_string_via_action_diff(&mut doc, 0, 0, "YY").unwrap();
    assert_eq!(doc.lines[0], "YYheXllo");

    let current_len = doc.lines[0].len();
    insert_string_via_action_diff(&mut doc, current_len, 0, "ZZ").unwrap();
    assert_eq!(doc.lines[0], "YYheXlloZZ");

    // Test inserting into an empty document
    let mut doc2 = Document::default();
    insert_string_via_action_diff(&mut doc2, 0, 0, "test").unwrap();
    assert_eq!(doc2.lines[0], "test");

    // Test inserting at an invalid line index (should do nothing)
    let mut doc3 = Document::default();
    doc3.lines[0] = "line".to_string();
    assert!(insert_string_via_action_diff(&mut doc3, 0, 1, "invalid").is_err());
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
    doc.apply_action_diff(
        &ActionDiff::LineSwap {
            y1: 0,
            y2: 1,
            original_cursor_x: 0,
            original_cursor_y: 0,
            new_cursor_x: 0,
            new_cursor_y: 1,
        },
        false,
    )
    .unwrap();
    assert_eq!(doc.lines[0], "line2");
    assert_eq!(doc.lines[1], "line1");
    assert_eq!(doc.lines[2], "line3");

    // Swap line1 (now at index 1) and line3
    doc.apply_action_diff(
        &ActionDiff::LineSwap {
            y1: 1,
            y2: 2,
            original_cursor_x: 0,
            original_cursor_y: 0,
            new_cursor_x: 0,
            new_cursor_y: 2,
        },
        false,
    )
    .unwrap();
    assert_eq!(doc.lines[0], "line2");
    assert_eq!(doc.lines[1], "line3");
    assert_eq!(doc.lines[2], "line1");

    // Try swapping out of bounds (should do nothing)
    doc.apply_action_diff(
        &ActionDiff::LineSwap {
            y1: 0,
            y2: 100,
            original_cursor_x: 0,
            original_cursor_y: 0,
            new_cursor_x: 0,
            new_cursor_y: 100,
        },
        false,
    )
    .unwrap();
    assert_eq!(doc.lines[0], "line2");
    assert_eq!(doc.lines[1], "line3");
    assert_eq!(doc.lines[2], "line1");

    doc.apply_action_diff(
        &ActionDiff::LineSwap {
            y1: 100,
            y2: 0,
            original_cursor_x: 0,
            original_cursor_y: 0,
            new_cursor_x: 0,
            new_cursor_y: 0,
        },
        false,
    )
    .unwrap();
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
    doc.apply_action_diff(
        &ActionDiff::CharChange(Diff {
            x: 0,
            y: 0,
            added_text: 'X'.to_string(),
            deleted_text: "".to_string(),
        }),
        false,
    )
    .unwrap(); // Modify the document
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
    doc.apply_action_diff(
        &ActionDiff::CharChange(Diff {
            x: 0,
            y: 0,
            added_text: 'X'.to_string(),
            deleted_text: "".to_string(),
        }),
        false,
    )
    .unwrap(); // Modify the document
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

#[test]
fn test_document_insert_string_with_newlines() {
    let mut doc = Document::default();
    doc.lines[0] = "start".to_string();
    insert_string_via_action_diff(&mut doc, 5, 0, "a\nbc").unwrap();
    assert_eq!(doc.lines.len(), 2);
    assert_eq!(doc.lines[0], "starta");
    assert_eq!(doc.lines[1], "bc");

    let mut doc2 = Document::default();
    doc2.lines[0] = "line1".to_string();
    doc2.lines.push("line2".to_string());
    insert_string_via_action_diff(&mut doc2, 2, 0, "X\nY").unwrap(); // Insert in the middle of line1
    assert_eq!(doc2.lines.len(), 3);
    assert_eq!(doc2.lines[0], "liX");
    assert_eq!(doc2.lines[1], "Yne1");
    assert_eq!(doc2.lines[2], "line2");
}

#[test]
fn test_delete_range_undo_redo() {
    let mut doc = Document::new_empty();
    doc.lines = vec![
        "Line One".to_string(),
        "Line Two".to_string(),
        "Line Three".to_string(),
    ];

    // Simulate a cut selection from "Line Two" (0,1) to "Line Thre" (9,2)
    // This means deleting "Line Two\nLine Thre"
    let deleted_content = vec!["Line Two".to_string(), "Line Thre".to_string()];

    let action_diff = ActionDiff::DeleteRange {
        start_x: 0,
        start_y: 1,
        end_x: 9,
        end_y: 2,
        content: deleted_content.clone(),
    };

    // Apply the deletion (redo)
    let (new_x, new_y) = doc.apply_action_diff(&action_diff, false).unwrap();
    assert_eq!(doc.lines.len(), 2);
    assert_eq!(doc.lines[0], "Line One");
    assert_eq!(doc.lines[1], "e"); // "Line Three" becomes "e" after "Line Thre" is removed
    assert_eq!(new_x, 0);
    assert_eq!(new_y, 1);

    // Undo the deletion
    let (new_x, new_y) = doc.apply_action_diff(&action_diff, true).unwrap();
    assert_eq!(doc.lines.len(), 3);
    assert_eq!(doc.lines[0], "Line One");
    assert_eq!(doc.lines[1], "Line Two");
    assert_eq!(doc.lines[2], "Line Three");
    assert_eq!(new_x, 9); // Cursor should be at the end of the re-inserted text
    assert_eq!(new_y, 2);

    // Redo the deletion
    let (new_x, new_y) = doc.apply_action_diff(&action_diff, false).unwrap();
    assert_eq!(doc.lines.len(), 2);
    assert_eq!(doc.lines[0], "Line One");
    assert_eq!(doc.lines[1], "e");
    assert_eq!(new_x, 0);
    assert_eq!(new_y, 1);
}
