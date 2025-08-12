use dmacs::backup::BackupManager;
use std::fs;
use std::path::PathBuf;
use chrono::{Local, Duration};

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
fn test_backup_manager_new() {
    let temp_dir = setup_test_env();
    let _backup_manager = BackupManager::new_with_base_dir(Some(temp_dir.clone())).unwrap();
    let expected_backup_dir = temp_dir.join(".dmacs").join("backup");
    assert!(expected_backup_dir.exists());
    teardown_test_env(&temp_dir);
}

#[test]
fn test_save_backup() {
    let temp_dir = setup_test_env();
    let backup_manager = BackupManager::new_with_base_dir(Some(temp_dir.clone())).unwrap();

    let filename = "test_file.txt";
    let content = "This is some test content.";
    backup_manager.save_backup(filename, content).unwrap();

    let backup_dir = temp_dir.join(".dmacs").join("backup");
    let mut found_backup = false;
    for entry in fs::read_dir(&backup_dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_file() && path.file_name().unwrap().to_str().unwrap().starts_with("test_file.txt.") && path.extension().unwrap() == "bak" {
            let saved_content = fs::read_to_string(&path).unwrap();
            assert_eq!(saved_content, content);
            found_backup = true;
            break;
        }
    }
    assert!(found_backup, "Backup file not found or incorrect.");
    teardown_test_env(&temp_dir);
}

#[test]
fn test_clean_old_backups() {
    let temp_dir = setup_test_env();
    let backup_manager = BackupManager::new_with_base_dir(Some(temp_dir.clone())).unwrap();
    let backup_dir = temp_dir.join(".dmacs").join("backup");

    // Create a recent backup
    let recent_timestamp = Local::now().format("%Y%m%d%H%M%S").to_string();
    let recent_backup_path = backup_dir.join(format!("recent_file.{}.bak", recent_timestamp));
    fs::write(&recent_backup_path, "recent content").unwrap();

    // Create an old backup (4 days ago)
    let old_timestamp = (Local::now() - Duration::days(4)).format("%Y%m%d%H%M%S").to_string();
    let old_backup_path = backup_dir.join(format!("old_file.{}.bak", old_timestamp));
    fs::write(&old_backup_path, "old content").unwrap();

    backup_manager.clean_old_backups().unwrap();

    assert!(recent_backup_path.exists(), "Recent backup should not be deleted.");
    assert!(!old_backup_path.exists(), "Old backup should be deleted.");

    teardown_test_env(&temp_dir);
}

#[test]
fn test_save_backup_empty_content() {
    let temp_dir = setup_test_env();
    let backup_manager = BackupManager::new_with_base_dir(Some(temp_dir.clone())).unwrap();

    let filename = "empty_file.txt";
    let content = "";
    backup_manager.save_backup(filename, content).unwrap();

    let backup_dir = temp_dir.join(".dmacs").join("backup");
    let mut found_backup = false;
    for entry in fs::read_dir(&backup_dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_file() && path.file_name().unwrap().to_str().unwrap().starts_with("empty_file.txt.") {
            found_backup = true;
            break;
        }
    }
    assert!(!found_backup, "Backup file should not be created for empty content.");
    teardown_test_env(&temp_dir);
}