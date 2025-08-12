use crate::error::{DmacsError, Result};
use chrono::{DateTime, Duration, Local, NaiveDateTime, TimeZone};
use log::debug;
use std::fs;
use std::path::PathBuf;

pub struct BackupManager {
    backup_dir: PathBuf,
}

impl BackupManager {
    pub fn new() -> Result<Self> {
        let home_dir = dirs::home_dir().ok_or(DmacsError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Home directory not found",
        )))?;
        let backup_dir = home_dir.join(".dmacs").join("backup");
        fs::create_dir_all(&backup_dir).map_err(DmacsError::Io)?;
        Ok(Self { backup_dir })
    }

    pub fn save_backup(&self, filename: &str, content: &str) -> Result<()> {
        if content.is_empty() {
            return Ok(());
        }

        let original_path = PathBuf::from(filename);
        let file_stem = original_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unnamed");
        let file_extension = original_path
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("");

        let now: DateTime<Local> = Local::now();
        let timestamp = now.format("%Y%m%d%H%M%S").to_string();

        let backup_filename = if file_extension.is_empty() {
            format!("{file_stem}.{timestamp}.bak")
        } else {
            format!("{file_stem}.{file_extension}.{timestamp}.bak")
        };
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
}
