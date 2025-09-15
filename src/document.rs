use crate::backup::BackupManager;
use crate::error::{DmacsError, Result};
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::time::SystemTime;

#[derive(Clone, Debug)]
pub struct ActionDiff {
    pub cursor_start_x: usize,
    pub cursor_start_y: usize,
    pub cursor_end_x: usize,
    pub cursor_end_y: usize,
    pub start_x: usize,
    pub start_y: usize,
    pub end_x: usize,
    pub end_y: usize,
    pub old: Vec<String>,
    pub new: Vec<String>,
}

#[derive(Clone)]
pub struct Document {
    pub lines: Vec<String>,
    pub filename: Option<String>,
    original_content: Option<String>,
}

impl Document {
    pub fn open(filename: &str) -> Result<Self> {
        let content = std::fs::read_to_string(filename).map_err(DmacsError::Io)?;
        let lines = content.lines().map(|s| s.to_string()).collect();
        Ok(Self {
            lines,
            filename: Some(filename.to_string()),
            original_content: Some(content),
        })
    }

    pub fn new_empty() -> Self {
        Self {
            lines: vec!["".to_string()],
            filename: None,
            original_content: None,
        }
    }

    pub fn save(&mut self, base_dir: Option<PathBuf>) -> Result<()> {
        if let Some(filename) = &self.filename {
            let backup_manager = BackupManager::new_with_base_dir(base_dir)?;

            // Backup original content if it exists and the document is dirty
            if self.is_dirty() {
                if let Some(original_content) = &self.original_content {
                    backup_manager.save_backup(filename, original_content)?;
                }
            }

            let mut file = std::fs::File::create(filename).map_err(DmacsError::Io)?;
            for _line in &self.lines {
                writeln!(file, "{_line}").map_err(DmacsError::Io)?;
            }
            self.original_content = Some(self.lines.join("\n") + "\n");

            // Clean up old backups
            backup_manager.clean_old_backups()?;
        }
        Ok(())
    }

    pub fn is_dirty(&self) -> bool {
        if self.filename.is_none() {
            // New file, always dirty until saved
            return true;
        }
        let original_lines: Vec<String> = self
            .original_content
            .as_ref()
            .map(|s| s.lines().map(|line| line.to_string()).collect())
            .unwrap_or_default();

        self.lines != original_lines
    }

    pub fn last_modified(&self) -> Result<SystemTime> {
        if let Some(filename) = &self.filename {
            let metadata = fs::metadata(filename).map_err(DmacsError::Io)?;
            metadata.modified().map_err(DmacsError::Io)
        } else {
            Err(DmacsError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Document has no filename, cannot get last modified time.",
            )))
        }
    }

    pub fn apply_action_diff(
        &mut self,
        action_diff: &ActionDiff,
        is_undo: bool,
    ) -> Result<(usize, usize)> {
        let ActionDiff {
            cursor_start_x,
            cursor_start_y,
            cursor_end_x,
            cursor_end_y,
            start_x,
            start_y,
            end_x,
            end_y,
            ref old,
            ref new,
        } = *action_diff;

        let replacement = if is_undo { old } else { new };

        // Delete start..end
        // Do nothing if it is insertion or deletion
        let is_insertion = old.is_empty() && !replacement.is_empty();
        let is_deletion = new.is_empty() && !replacement.is_empty();
        if !is_insertion && !is_deletion {
            if start_y == end_y {
                if start_y < self.lines.len() {
                    self.lines[start_y].drain(start_x..end_x);
                }
            } else {
                let prefix = self.lines[start_y][..start_x].to_string();
                let suffix = self.lines[end_y][end_x..].to_string();
                self.lines[start_y] = format!("{prefix}{suffix}");

                for y in (start_y + 1..=end_y).rev() {
                    if y < self.lines.len() {
                        self.lines.remove(y);
                    }
                }
            }
        }

        // Insert replacement
        if !replacement.is_empty() {
            if replacement.len() == 1 {
                if start_y < self.lines.len() {
                    self.lines[start_y].insert_str(start_x, &replacement[0]);
                } else {
                    self.lines.insert(start_y, replacement[0].clone());
                }
            } else {
                let suffix = if start_y < self.lines.len() {
                    self.lines[start_y][start_x..].to_string()
                } else {
                    String::new()
                };
                self.lines[start_y] =
                    format!("{}{}", &self.lines[start_y][..start_x], replacement[0]);

                for (i, line) in replacement
                    .iter()
                    .enumerate()
                    .skip(1)
                    .take(replacement.len() - 2)
                {
                    self.lines.insert(start_y + i, line.clone());
                }

                let end_line_idx = start_y + replacement.len() - 1;
                self.lines.insert(
                    end_line_idx,
                    format!("{}{}", replacement.last().unwrap(), suffix),
                );
            }
        }

        // Adjust cursor position
        if is_undo {
            Ok((cursor_start_x, cursor_start_y))
        } else {
            Ok((cursor_end_x, cursor_end_y))
        }
    }
}

impl Default for Document {
    fn default() -> Self {
        Self::new_empty()
    }
}
