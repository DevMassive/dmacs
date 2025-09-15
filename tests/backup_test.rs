use chrono::{Duration, Local};
use dmacs::backup::BackupManager;
use sha2::{Digest, Sha256};
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

    let filename = temp_dir.join("test_file.txt");
    fs::write(&filename, "content").unwrap(); // Ensure file exists for canonicalization
    let filename_str = filename.to_str().unwrap();
    let content = "This is some test content.";
    backup_manager.save_backup(filename_str, content).unwrap();

    let canonical_path = std::fs::canonicalize(&filename).unwrap();
    let mut hasher = Sha256::new();
    hasher.update(canonical_path.to_string_lossy().as_bytes());
    let result = hasher.finalize();
    let hash_str = format!("{:x}", result);
    let short_hash = &hash_str[..8];
    let expected_prefix = format!("{}-{}", filename.file_name().unwrap().to_str().unwrap(), short_hash);

    let backup_dir = temp_dir.join(".dmacs").join("backup");
    let mut found_backup = false;
    for entry in fs::read_dir(&backup_dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_file() {
            let backup_filename = path.file_name().unwrap().to_str().unwrap();
            if backup_filename.starts_with(&expected_prefix) && backup_filename.ends_with(".bak") {
                let saved_content = fs::read_to_string(&path).unwrap();
                assert_eq!(saved_content, content);
                found_backup = true;
                break;
            }
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
    let recent_backup_path = backup_dir.join(format!("recent_file.{recent_timestamp}.bak"));
    fs::write(&recent_backup_path, "recent content").unwrap();

    // Create an old backup (4 days ago)
    let old_timestamp = (Local::now() - Duration::days(4))
        .format("%Y%m%d%H%M%S")
        .to_string();
    let old_backup_path = backup_dir.join(format!("old_file.{old_timestamp}.bak"));
    fs::write(&old_backup_path, "old content").unwrap();

    backup_manager.clean_old_backups().unwrap();

    assert!(
        recent_backup_path.exists(),
        "Recent backup should not be deleted."
    );
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
        if path.is_file()
            && path
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .starts_with("empty_file.txt.")
        {
            found_backup = true;
            break;
        }
    }
    assert!(
        !found_backup,
        "Backup file should not be created for empty content."
    );
    teardown_test_env(&temp_dir);
}

#[test]
fn test_restore_backup() {
    let temp_dir = setup_test_env();
    let backup_manager = BackupManager::new_with_base_dir(Some(temp_dir.clone())).unwrap();

    let filename = temp_dir.join("test_file.txt");
    let filename_str = filename.to_str().unwrap();

    // Create and back up version 1
    let content_v1 = "version 1";
    fs::write(&filename, content_v1).unwrap();
    backup_manager.save_backup(filename_str, content_v1).unwrap();

    // Wait a moment to ensure a different timestamp
    std::thread::sleep(std::time::Duration::from_secs(1));

    // Create and back up version 2
    let content_v2 = "version 2";
    fs::write(&filename, content_v2).unwrap();
    backup_manager.save_backup(filename_str, content_v2).unwrap();

    // Modify the file to a different state
    fs::write(&filename, "latest content").unwrap();

    // Restore from backup
    backup_manager.restore_backup(filename_str).unwrap();

    // Check if the file is restored to version 2
    let restored_content = fs::read_to_string(&filename).unwrap();
    assert_eq!(restored_content, content_v2);

    teardown_test_env(&temp_dir);
}

#[test]
fn test_restore_backup_not_found() {
    let temp_dir = setup_test_env();
    let backup_manager = BackupManager::new_with_base_dir(Some(temp_dir.clone())).unwrap();

    let filename = "non_existent_file.txt";
    let result = backup_manager.restore_backup(filename);

    assert!(result.is_err());
    if let Some(dmacs::error::DmacsError::BackupNotFound(name)) = result.err() {
        assert_eq!(name, filename);
    } else {
        panic!("Expected BackupNotFound error");
    }

    teardown_test_env(&temp_dir);
}
