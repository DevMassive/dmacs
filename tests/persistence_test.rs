use dmacs::persistence::{self, CursorPosition};
use filetime::{FileTime, set_file_mtime};
use serial_test::serial;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};
use uuid::Uuid;

const CLEANUP_THRESHOLD_DAYS: u64 = 3;

// Helper function to create a temporary directory for tests
fn setup_test_env() -> PathBuf {
    let temp_dir = PathBuf::from(format!("/tmp/dmacs_persistence_test_{}", Uuid::new_v4()));
    fs::create_dir_all(&temp_dir).expect("Failed to create temporary test directory");
    temp_dir
}

// Helper function to clean up the temporary directory
fn teardown_test_env(temp_dir: &PathBuf) {
    if temp_dir.exists() {
        fs::remove_dir_all(temp_dir).expect("Failed to remove temporary test directory");
    }
}

// Helper to get the expected cursor position file path within a test environment
fn get_test_cursor_pos_file_path(base_dir: &Path, file_path: &str) -> PathBuf {
    let config_dir = base_dir.join(".dmacs");
    let cursor_pos_dir = config_dir.join("cursor_positions");
    // This part needs to match the hashing logic in src/persistence.rs
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(file_path.as_bytes());
    let hash = hasher.finalize();
    let filename = format!("{hash:x}.json");
    cursor_pos_dir.join(filename)
}

#[test]
#[serial]
fn test_cleanup_old_cursor_position_files() {
    let temp_dir = setup_test_env();

    // Override the DMACS_CONFIG_DIR for this test to use the temporary directory
    // This is a bit tricky as DMACS_CONFIG_DIR is a const. We'll have to mock the get_config_dir function.
    // For now, I'll assume the persistence functions can be made to accept a base directory for testing.
    // If not, this test will require more significant refactoring of the persistence module.

    // For the purpose of this test, we'll manually create the directory structure
    // and then call the cleanup function, assuming it operates on the default location
    // or we can temporarily change the home directory for the test.

    // A better approach for testing would be to make `get_config_dir` configurable for tests.
    // Since it's not, we'll have to simulate the directory structure and then call the cleanup.

    // Create the .dmacs/cursor_positions directory within the temp_dir
    let test_dmacs_dir = temp_dir.join(".dmacs");
    let test_cursor_pos_dir = test_dmacs_dir.join("cursor_positions");
    fs::create_dir_all(&test_cursor_pos_dir).expect("Failed to create test cursor positions dir");

    // Create a recent cursor position file
    let recent_file_path = "/path/to/recent_file.txt";
    let recent_pos = CursorPosition {
        file_path: recent_file_path.to_string(),
        last_modified: SystemTime::now(),
        cursor_x: 10,
        cursor_y: 5,
        scroll_row_offset: 0,
        scroll_col_offset: 0,
    };
    let recent_hashed_path = get_test_cursor_pos_file_path(&temp_dir, recent_file_path);
    fs::write(
        &recent_hashed_path,
        serde_json::to_string_pretty(&recent_pos).unwrap(),
    )
    .unwrap();

    // Create an old cursor position file
    let old_file_path = "/path/to/old_file.txt";
    let old_pos = CursorPosition {
        file_path: old_file_path.to_string(),
        last_modified: SystemTime::now(),
        cursor_x: 20,
        cursor_y: 10,
        scroll_row_offset: 0,
        scroll_col_offset: 0,
    };
    let old_hashed_path = get_test_cursor_pos_file_path(&temp_dir, old_file_path);
    fs::write(
        &old_hashed_path,
        serde_json::to_string_pretty(&old_pos).unwrap(),
    )
    .unwrap();

    // Set the modification time of the old file to be older than the threshold
    let old_mtime =
        SystemTime::now() - Duration::from_secs(CLEANUP_THRESHOLD_DAYS * 24 * 60 * 60 + 1);
    set_file_mtime(&old_hashed_path, FileTime::from_system_time(old_mtime)).unwrap();

    // Call the cleanup function
    // This is the tricky part: persistence::cleanup_old_cursor_position_files() uses dirs::home_dir()
    // which is not easily mockable. For a proper unit test, get_config_dir() should be made to accept
    // an optional base directory for testing. Without that, this test will operate on the actual home directory
    // or require setting the HOME environment variable, which is not ideal for isolated tests.

    // For now, I'll assume the cleanup function will operate correctly on the default path.
    // If the persistence module cannot be made testable by passing a base directory, this test
    // will not be truly isolated and might affect the user's actual .dmacs directory.

    // A more robust solution would involve refactoring `get_config_dir` to allow injection of a base path.
    // For the purpose of adding a test as requested, I will proceed with calling the function directly,
    // but note this limitation.

    // Temporarily change the HOME environment variable for the test
    let original_home = std::env::var_os("HOME");
    unsafe {
        std::env::set_var("HOME", &temp_dir);
    }

    persistence::cleanup_old_cursor_position_files();

    // Restore original HOME environment variable
    if let Some(home) = original_home {
        unsafe {
            std::env::set_var("HOME", home);
        }
    } else {
        unsafe {
            std::env::remove_var("HOME");
        }
    }

    // Assertions
    assert!(
        recent_hashed_path.exists(),
        "Recent file should not be deleted"
    );
    assert!(!old_hashed_path.exists(), "Old file should be deleted");

    teardown_test_env(&temp_dir);
}

#[test]
#[serial]
fn test_get_cursor_position_with_scroll_restoration() {
    let temp_dir = setup_test_env();
    let file_path = "/path/to/test_file.txt";
    let last_modified = SystemTime::now();
    let expected_cursor_x = 15;
    let expected_cursor_y = 25;
    let expected_scroll_row_offset = 5;
    let expected_scroll_col_offset = 10;

    let pos = CursorPosition {
        file_path: file_path.to_string(),
        last_modified,
        cursor_x: expected_cursor_x,
        cursor_y: expected_cursor_y,
        scroll_row_offset: expected_scroll_row_offset,
        scroll_col_offset: expected_scroll_col_offset,
    };

    // Temporarily change the HOME environment variable for the test
    let original_home = std::env::var_os("HOME");
    unsafe {
        std::env::set_var("HOME", &temp_dir);
    }

    // Save the cursor position
    persistence::save_cursor_position(pos).unwrap();

    // Retrieve the cursor position
    let retrieved_pos = persistence::get_cursor_position(file_path, last_modified);

    // Restore original HOME environment variable
    if let Some(home) = original_home {
        unsafe {
            std::env::set_var("HOME", home);
        }
    } else {
        unsafe {
            std::env::remove_var("HOME");
        }
    }

    // Assertions
    assert!(retrieved_pos.is_some());
    let (x, y, scroll_row, scroll_col) = retrieved_pos.unwrap();
    assert_eq!(x, expected_cursor_x);
    assert_eq!(y, expected_cursor_y);
    assert_eq!(scroll_row, expected_scroll_row_offset);
    assert_eq!(scroll_col, expected_scroll_col_offset);

    teardown_test_env(&temp_dir);
}
