use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;
use std::time::{SystemTime, Duration};
use log::{debug, error};

const DMACS_CONFIG_DIR: &str = ".dmacs";
const CURSOR_POS_FILE: &str = "cursor_positions.json";
const CLEANUP_THRESHOLD_DAYS: u64 = 3;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CursorPosition {
    pub file_path: String,
    pub last_modified: SystemTime,
    pub cursor_x: usize,
    pub cursor_y: usize,
    pub timestamp: SystemTime, // When this record was last updated/saved
}

#[derive(Serialize, Deserialize, Debug, Default)]
struct CursorPositions {
    positions: HashMap<String, CursorPosition>,
}

fn get_config_dir() -> Result<PathBuf, io::Error> {
    let home_dir = dirs::home_dir().ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Home directory not found"))?;
    let config_dir = home_dir.join(DMACS_CONFIG_DIR);
    if !config_dir.exists() {
        fs::create_dir_all(&config_dir)?;
    }
    Ok(config_dir)
}

fn get_cursor_pos_file_path() -> Result<PathBuf, io::Error> {
    Ok(get_config_dir()?.join(CURSOR_POS_FILE))
}

fn load_cursor_positions() -> CursorPositions {
    let file_path = match get_cursor_pos_file_path() {
        Ok(path) => path,
        Err(e) => {
            error!("Failed to get cursor position file path: {}", e);
            return CursorPositions::default();
        }
    };

    if file_path.exists() {
        debug!("Loading cursor positions from {}", file_path.display());
        match fs::read_to_string(&file_path) {
            Ok(content) => match serde_json::from_str::<CursorPositions>(&content) {
                Ok(positions) => {
                    debug!("Successfully loaded {} cursor positions.", positions.positions.len());
                    positions
                },
                Err(e) => {
                    error!("Failed to deserialize cursor positions from {}: {}", file_path.display(), e);
                    CursorPositions::default()
                }
            },
            Err(e) => {
                error!("Failed to read cursor positions file {}: {}", file_path.display(), e);
                CursorPositions::default()
            }
        }
    } else {
        debug!("Cursor positions file not found at {}. Starting with empty positions.", file_path.display());
        CursorPositions::default()
    }
}

pub fn save_cursor_position(pos: CursorPosition) -> Result<(), io::Error> {
    debug!("Attempting to save cursor position for file: {}", pos.file_path);
    let mut all_positions = load_cursor_positions();
    all_positions.positions.insert(pos.file_path.clone(), pos);
    cleanup_old_records(&mut all_positions);

    let file_path = get_cursor_pos_file_path()?;
    let content = serde_json::to_string_pretty(&all_positions)?;
    let mut file = fs::File::create(&file_path)?;
    file.write_all(content.as_bytes())?;
    debug!("Saved {} cursor positions to {}.", all_positions.positions.len(), file_path.display());
    Ok(())
}

fn cleanup_old_records(positions: &mut CursorPositions) {
    let now = SystemTime::now();
    let threshold = now - Duration::from_secs(CLEANUP_THRESHOLD_DAYS * 24 * 60 * 60);
    let initial_count = positions.positions.len();
    positions.positions.retain(|_, pos| {
        pos.timestamp >= threshold
    });
    debug!("Cleaned up {} old cursor position records. Remaining: {}.", initial_count - positions.positions.len(), positions.positions.len());
}

pub fn get_cursor_position(file_path: &str) -> Option<(usize, usize)> {
    debug!("Looking for cursor position for file: {}", file_path);
    let all_positions = load_cursor_positions();
    if let Some(pos) = all_positions.positions.get(file_path) {
        debug!("Found record for {}. Restoring cursor position: ({}, {}).", file_path, pos.cursor_x, pos.cursor_y);
        return Some((pos.cursor_x, pos.cursor_y));
    } else {
        debug!("No record found for {}.", file_path);
    }
    None
}
