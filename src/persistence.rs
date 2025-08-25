use log::{debug, error};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;
use std::time::{Duration, SystemTime};

const DMACS_CONFIG_DIR: &str = ".dmacs";
const CURSOR_POSITIONS_SUBDIR: &str = "cursor_positions";
const CLEANUP_THRESHOLD_DAYS: u64 = 3;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CursorPosition {
    pub file_path: String,
    pub last_modified: SystemTime,
    pub cursor_x: usize,
    pub cursor_y: usize,
    pub scroll_row_offset: usize,
    pub scroll_col_offset: usize,
}

fn get_config_dir() -> Result<PathBuf, io::Error> {
    let home_dir = dirs::home_dir()
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Home directory not found"))?;
    let config_dir = home_dir.join(DMACS_CONFIG_DIR);
    if !config_dir.exists() {
        fs::create_dir_all(&config_dir)?;
    }
    Ok(config_dir)
}

fn get_cursor_pos_dir() -> Result<PathBuf, io::Error> {
    let config_dir = get_config_dir()?;
    let cursor_pos_dir = config_dir.join(CURSOR_POSITIONS_SUBDIR);
    if !cursor_pos_dir.exists() {
        fs::create_dir_all(&cursor_pos_dir)?;
    }
    Ok(cursor_pos_dir)
}

fn get_cursor_pos_file_path(file_path: &str) -> Result<PathBuf, io::Error> {
    let cursor_pos_dir = get_cursor_pos_dir()?;

    let mut hasher = Sha256::new();
    hasher.update(file_path.as_bytes());
    let hash = hasher.finalize();
    let filename = format!("{hash:x}.json");

    Ok(cursor_pos_dir.join(filename))
}

fn load_cursor_position(file_path: &str) -> Option<CursorPosition> {
    let file_path_hashed = match get_cursor_pos_file_path(file_path) {
        Ok(path) => path,
        Err(e) => {
            error!("Failed to get cursor position file path for {file_path}: {e}");
            return None;
        }
    };

    if file_path_hashed.exists() {
        debug!(
            "Loading cursor position from {}",
            file_path_hashed.display()
        );
        match fs::read_to_string(&file_path_hashed) {
            Ok(content) => match serde_json::from_str::<CursorPosition>(&content) {
                Ok(position) => {
                    debug!("Successfully loaded cursor position for {file_path}.");
                    Some(position)
                }
                Err(e) => {
                    error!(
                        "Failed to deserialize cursor position from {}: {}",
                        file_path_hashed.display(),
                        e
                    );
                    None
                }
            },
            Err(e) => {
                error!(
                    "Failed to read cursor position file {}: {}",
                    file_path_hashed.display(),
                    e
                );
                None
            }
        }
    } else {
        debug!(
            "Cursor position file not found at {}. Starting with no position.",
            file_path_hashed.display()
        );
        None
    }
}

pub fn save_cursor_position(pos: CursorPosition) -> Result<(), io::Error> {
    debug!(
        "Attempting to save cursor position for file: {}",
        pos.file_path
    );
    let file_path_hashed = get_cursor_pos_file_path(&pos.file_path)?;
    let content = serde_json::to_string_pretty(&pos)?;
    let mut file = fs::File::create(&file_path_hashed)?;
    file.write_all(content.as_bytes())?;
    debug!(
        "Saved cursor position for {} to {}.",
        pos.file_path,
        file_path_hashed.display()
    );
    Ok(())
}

pub fn get_cursor_position(
    file_path: &str,
    last_modified: SystemTime,
) -> Option<(usize, usize, usize, usize)> {
    debug!("Looking for cursor position for file: {file_path}");
    if let Some(pos) = load_cursor_position(file_path) {
        if pos.last_modified != last_modified {
            debug!(
                "Last modified date for {file_path} has changed. Not restoring cursor position."
            );
            return None;
        }
        debug!(
            "Found record for {}. Restoring cursor position: ({}, {}), scroll: ({}, {}).",
            file_path, pos.cursor_x, pos.cursor_y, pos.scroll_row_offset, pos.scroll_col_offset
        );
        return Some((
            pos.cursor_x,
            pos.cursor_y,
            pos.scroll_row_offset,
            pos.scroll_col_offset,
        ));
    } else {
        debug!("No record found for {file_path}.");
    }
    None
}

pub fn cleanup_old_cursor_position_files() {
    debug!("Starting cleanup of old cursor position files.");
    let cursor_pos_dir = match get_cursor_pos_dir() {
        Ok(path) => path,
        Err(e) => {
            error!("Failed to get cursor positions directory for cleanup: {e}");
            return;
        }
    };

    let now = SystemTime::now();
    let threshold = now - Duration::from_secs(CLEANUP_THRESHOLD_DAYS * 24 * 60 * 60);

    match fs::read_dir(&cursor_pos_dir) {
        Ok(entries) => {
            for entry in entries {
                let entry = match entry {
                    Ok(e) => e,
                    Err(e) => {
                        error!(
                            "Error reading directory entry in {}: {}",
                            cursor_pos_dir.display(),
                            e
                        );
                        continue;
                    }
                };
                let path = entry.path();
                if path.is_file() {
                    match fs::metadata(&path) {
                        Ok(metadata) => match metadata.modified() {
                            Ok(modified_time) => {
                                if modified_time < threshold {
                                    match fs::remove_file(&path) {
                                        Ok(_) => debug!(
                                            "Deleted old cursor position file: {}",
                                            path.display()
                                        ),
                                        Err(e) => error!(
                                            "Failed to delete old cursor position file {}: {}",
                                            path.display(),
                                            e
                                        ),
                                    }
                                }
                            }
                            Err(e) => {
                                error!("Failed to get modified time for {}: {}", path.display(), e)
                            }
                        },
                        Err(e) => error!("Failed to get metadata for {}: {}", path.display(), e),
                    }
                }
            }
        }
        Err(e) => error!(
            "Failed to read cursor positions directory {}: {}",
            cursor_pos_dir.display(),
            e
        ),
    }
    debug!("Finished cleanup of old cursor position files.");
}
