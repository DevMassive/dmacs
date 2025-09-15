use crate::error::{DmacsError, Result};
use chrono::{DateTime, Duration, Local, NaiveDateTime, TimeZone};
use log::debug;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::PathBuf;

pub struct BackupManager {
    backup_dir: PathBuf,
}

impl BackupManager {
    pub fn new() -> Result<Self> {
        Self::new_with_base_dir(None)
    }

    pub fn new_with_base_dir(base_dir: Option<PathBuf>) -> Result<Self> {
        let base = if let Some(dir) = base_dir {
            dir
        } else {
            dirs::home_dir().ok_or(DmacsError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Home directory not found",
            )))?
        };
        let backup_dir = base.join(".dmacs").join("backup");
        fs::create_dir_all(&backup_dir).map_err(DmacsError::Io)?;
        Ok(Self { backup_dir })
    }

    pub fn save_backup(&self, filename: &str, content: &str) -> Result<()> {
        if content.is_empty() {
            return Ok(());
        }

        if let Some(latest_backup_path) = self.find_latest_backup(filename)? {
            if let Ok(latest_content) = fs::read_to_string(&latest_backup_path) {
                if latest_content == content {
                    debug!("Content for {} has not changed, skipping backup.", filename);
                    return Ok(());
                }
            }
        }

        let prefix = self.get_backup_file_prefix(filename);
        let now: DateTime<Local> = Local::now();
        let timestamp = now.format("%Y%m%d%H%M%S").to_string();

        let backup_filename = format!("{prefix}.{timestamp}.bak");
        let backup_path = self.backup_dir.join(backup_filename);

        fs::write(&backup_path, content).map_err(DmacsError::Io)?;
        debug!("Backed up {} to {}", filename, backup_path.display());
        Ok(())
    }

    pub fn clean_old_backups(&self) -> Result<()> {
        let now: DateTime<Local> = Local::now();
        let three_days_ago = now - Duration::days(3);

        for entry in fs::read_dir(&self.backup_dir).map_err(DmacsError::Io)? {
            let entry = entry.map_err(DmacsError::Io)?;
            let path = entry.path();

            if path.is_file() {
                if let Some(filename_str) = path.file_name().and_then(|s| s.to_str()) {
                    // Expected format: file_stem.extension.timestamp.bak or file_stem.timestamp.bak
                    let parts: Vec<&str> = filename_str.split('.').collect();
                    let num_parts = parts.len();

                    if num_parts >= 3 && parts[num_parts - 1] == "bak" {
                        let timestamp_str = parts[num_parts - 2];

                        if let Ok(naive_datetime) =
                            NaiveDateTime::parse_from_str(timestamp_str, "%Y%m%d%H%M%S")
                        {
                            if let Some(backup_timestamp) =
                                Local.from_local_datetime(&naive_datetime).single()
                            {
                                if backup_timestamp < three_days_ago {
                                    fs::remove_file(&path).map_err(DmacsError::Io)?;
                                    debug!("Deleted old backup: {}", path.display());
                                }
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }

    pub fn restore_backup(&self, filename: &str) -> Result<()> {
        if let Some(backup_to_restore) = self.find_latest_backup(filename)? {
            let content = fs::read_to_string(&backup_to_restore).map_err(DmacsError::Io)?;
            fs::write(filename, content).map_err(DmacsError::Io)?;
            debug!(
                "Restored {} from {}",
                filename,
                backup_to_restore.display()
            );
            fs::remove_file(&backup_to_restore).map_err(DmacsError::Io)?;
            debug!("Deleted backup file: {}", backup_to_restore.display());
            Ok(())
        } else {
            Err(DmacsError::BackupNotFound(filename.to_string()))
        }
    }

    fn find_latest_backup(&self, filename: &str) -> Result<Option<PathBuf>> {
        let prefix = self.get_backup_file_prefix(filename);
        let mut latest_backup: Option<PathBuf> = None;
        let mut latest_timestamp: Option<NaiveDateTime> = None;

        for entry in fs::read_dir(&self.backup_dir).map_err(DmacsError::Io)? {
            let entry = entry.map_err(DmacsError::Io)?;
            let path = entry.path();
            if path.is_file() {
                if let Some(backup_filename_str) = path.file_name().and_then(|s| s.to_str()) {
                    if backup_filename_str.starts_with(&prefix)
                        && backup_filename_str.ends_with(".bak")
                    {
                        let timestamp_part = backup_filename_str
                            .trim_start_matches(&prefix)
                            .trim_start_matches('.') // The timestamp is preceded by a dot
                            .trim_end_matches(".bak");

                        if let Ok(timestamp) =
                            NaiveDateTime::parse_from_str(timestamp_part, "%Y%m%d%H%M%S")
                        {
                            if latest_timestamp.is_none() || timestamp > latest_timestamp.unwrap() {
                                latest_timestamp = Some(timestamp);
                                latest_backup = Some(path.clone());
                            }
                        }
                    }
                }
            }
        }

        Ok(latest_backup)
    }

    fn get_backup_file_prefix(&self, filename: &str) -> String {
        let original_path = PathBuf::from(filename);
        let file_name = original_path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("unnamed")
            .to_string();

        // To ensure consistency, use the canonical path for hashing.
        let canonical_path = std::fs::canonicalize(&original_path).unwrap_or(original_path);

        let mut hasher = Sha256::new();
        hasher.update(canonical_path.to_string_lossy().as_bytes());
        let result = hasher.finalize();
        let hash_str = format!("{:x}", result);
        let short_hash = &hash_str[..8];

        format!("{}-{}", file_name, short_hash)
    }
}