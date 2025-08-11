use crate::error::{DmacsError, Result};
use std::io::Write;

// Document being edited
#[derive(Clone, Debug)]
pub struct Diff {
    pub x: usize,
    pub y: usize,
    pub added_text: String,
    pub deleted_text: String,
}

#[derive(Clone, Debug)]
pub enum ActionDiff {
    CharChange(Diff),
    NewlineInsertion {
        x: usize,
        y: usize,
    },
    NewlineDeletion {
        x: usize,
        y: usize,
    },
    LineSwap {
        y1: usize,
        y2: usize,
    },
    DeleteRange {
        start_x: usize,
        start_y: usize,
        end_x: usize,
        end_y: usize,
        content: Vec<String>,
    },
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

    pub fn save(&mut self) -> Result<()> {
        if let Some(filename) = &self.filename {
            let mut file = std::fs::File::create(filename).map_err(DmacsError::Io)?;
            for _line in &self.lines {
                writeln!(file, "{_line}").map_err(DmacsError::Io)?;
            }
            self.original_content = Some(self.lines.join("\n") + "\n");
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

    pub fn swap_lines(&mut self, y1: usize, y2: usize) {
        if y1 < self.lines.len() && y2 < self.lines.len() {
            self.lines.swap(y1, y2);
        }
    }

    pub fn apply_action_diff(
        &mut self,
        action_diff: &ActionDiff,
        is_undo: bool,
    ) -> Result<(usize, usize)> {
        match action_diff {
            ActionDiff::CharChange(diff) => self.modify_single_char(diff, is_undo),
            ActionDiff::NewlineInsertion { x, y } => {
                if is_undo {
                    self.delete_newline(*x, *y) // Undo insertion is deletion
                } else {
                    self.insert_newline(*x, *y) // Redo insertion is insertion
                }
            }
            ActionDiff::NewlineDeletion { x, y } => {
                if is_undo {
                    self.insert_newline(*x, *y) // Undo deletion is insertion
                } else {
                    self.delete_newline(*x, *y) // Redo deletion is deletion
                }
            }
            ActionDiff::LineSwap { y1, y2 } => {
                self.swap_lines(*y1, *y2);
                // For LineSwap, the cursor position logic is handled by the caller (Editor)
                // This function just performs the swap.
                // Return a dummy cursor position, as it's not used by Editor for LineSwap.
                Ok((0, 0))
            }
            ActionDiff::DeleteRange {
                start_x,
                start_y,
                end_x,
                end_y,
                content,
            } => {
                if is_undo {
                    // Undo: Re-insert the content
                    let mut current_y = *start_y;
                    let mut current_x = *start_x;

                    if content.is_empty() {
                        return Ok((current_x, current_y));
                    }

                    // If single line deletion
                    if *start_y == *end_y {
                        if *start_y < self.lines.len() {
                            self.lines[*start_y].insert_str(*start_x, &content[0]);
                        } else {
                            self.lines.insert(*start_y, content[0].clone());
                        }
                    } else {
                        // Multi-line deletion
                        // The current line at start_y contains the prefix of the original start_y line
                        // and the suffix of the original end_y line.
                        // We need to split it and insert the deleted lines.

                        let original_start_line_prefix =
                            self.lines[*start_y][0..*start_x].to_string();
                        let original_end_line_suffix = self.lines[*start_y][*start_x..].to_string();

                        // Reconstruct the start line
                        self.lines[*start_y] =
                            format!("{}{}", original_start_line_prefix, content[0]);

                        // Insert the intermediate lines
                        for (i, line) in content.iter().enumerate().skip(1).take(content.len() - 2)
                        {
                            self.lines.insert(*start_y + i, line.clone());
                        }

                        // Reconstruct the end line
                        let end_line_idx = *start_y + content.len() - 1;
                        self.lines.insert(
                            end_line_idx,
                            format!("{}{}", content.last().unwrap(), original_end_line_suffix),
                        );
                    }

                    // Adjust cursor position
                    current_x = *end_x;
                    current_y = *end_y;

                    Ok((current_x, current_y))
                } else {
                    // Redo: Perform the deletion again
                    if *start_y == *end_y {
                        // Single line deletion
                        if *start_y < self.lines.len() {
                            self.lines[*start_y].drain(*start_x..*end_x);
                        }
                    } else {
                        // Multi-line deletion
                        let mut remaining_start_line_prefix = String::new();
                        if *start_y < self.lines.len() {
                            let start_line = &mut self.lines[*start_y];
                            remaining_start_line_prefix = start_line[0..*start_x].to_string();
                            start_line.drain(*start_x..); // Remove from start_x to end of line
                        }

                        let mut remaining_end_line_suffix = String::new();
                        if *end_y < self.lines.len() {
                            let end_line = &mut self.lines[*end_y];
                            remaining_end_line_suffix = end_line[*end_x..].to_string();
                            end_line.drain(0..*end_x); // Remove from beginning to end_x
                        }

                        // Join the remaining parts
                        if *start_y < self.lines.len() {
                            self.lines[*start_y] =
                                format!("{remaining_start_line_prefix}{remaining_end_line_suffix}");
                        }

                        // Remove intermediate lines and the end_y line if it's different from start_y
                        // Iterate backwards to avoid index issues
                        for y_idx in (*start_y + 1..=*end_y).rev() {
                            // Iterate from end_y down to start_y + 1
                            if y_idx < self.lines.len() {
                                self.lines.remove(y_idx);
                            }
                        }
                    }
                    Ok((*start_x, *start_y))
                }
            }
        }
    }

    pub fn modify_single_char(&mut self, diff: &Diff, is_undo: bool) -> Result<(usize, usize)> {
        let (add, delete) = if is_undo {
            (&diff.deleted_text, &diff.added_text)
        } else {
            (&diff.added_text, &diff.deleted_text)
        };

        if diff.y >= self.lines.len() {
            return Err(DmacsError::Document(format!(
                "Invalid line index: {}",
                diff.y
            )));
        }

        // Calculate new cursor position (initial values)
        let new_x = diff.x + add.len();
        let new_y = diff.y; // new_y is not mutable anymore as it's not changed here

        let line = &mut self.lines[diff.y]; // Get mutable reference to the line

        // Handle deletion
        if !delete.is_empty() {
            let delete_len = delete.len();
            if diff.x + delete_len > line.len() {
                return Err(DmacsError::Document(format!(
                    "Deletion out of bounds: x={}, delete_len={}, line_len={}",
                    diff.x,
                    delete_len,
                    line.len()
                )));
            }
            if line[diff.x..].starts_with(delete) {
                line.replace_range(diff.x..(diff.x + delete_len), "");
            } else {
                let found_text = &line[diff.x..(diff.x + delete_len)];
                return Err(DmacsError::Document(format!(
                    "Text to delete does not match: expected \"{delete}\", found \"{found_text}\""
                )));
            }
        }

        // Handle insertion
        if !add.is_empty() {
            if diff.x > line.len() {
                return Err(DmacsError::Document(format!(
                    "Insertion out of bounds: x={}, line_len={}",
                    diff.x,
                    line.len()
                )));
            }
            line.insert_str(diff.x, add);
        }

        Ok((new_x, new_y))
    }

    pub fn insert_newline(&mut self, x: usize, y: usize) -> Result<(usize, usize)> {
        // Handle newline insertion (splitting a line)
        if y > self.lines.len() {
            return Err(DmacsError::Document(format!(
                "Invalid line index for newline insertion: {y}"
            )));
        }
        if y == self.lines.len() {
            self.lines.push(String::new());
        } else {
            let current_line = self
                .lines
                .get_mut(y)
                .ok_or(DmacsError::Document(format!("Invalid line index: {y}")))?;
            let new_line = current_line.split_off(x);
            self.lines.insert(y + 1, new_line);
        }
        Ok((0, y + 1))
    }
    pub fn delete_newline(&mut self, x: usize, y: usize) -> Result<(usize, usize)> {
        // Handle newline deletion (joining lines)
        if x == 0 {
            // Backspace at the beginning of a line, join with previous
            if y == 0 {
                // If it's the first line and we're deleting a newline at x=0,
                // it means we're effectively removing the first line.
                // This happens when you delete the newline *after* the first line.
                if self.lines.len() > 1 {
                    self.lines.remove(y); // Remove the first line
                    Ok((0, 0)) // Cursor moves to the beginning of the new first line
                } else {
                    // If it's the only line, just clear it.
                    self.lines[y].clear();
                    Ok((0, 0))
                }
            } else {
                let current_line = self.lines.remove(y);
                let prev_line_len = self.lines[y - 1].len();
                self.lines[y - 1].push_str(&current_line);
                Ok((prev_line_len, y - 1))
            }
        } else {
            // Delete at the end of a line, join with next
            if y >= self.lines.len().saturating_sub(1) {
                return Err(DmacsError::Document(format!(
                    "Cannot join line {y} with next line."
                )));
            }
            let next_line = self.lines.remove(y + 1);
            let current_line_len = self.lines[y].len();
            self.lines[y].push_str(&next_line);
            Ok((current_line_len, y))
        }
    }
}

impl Default for Document {
    fn default() -> Self {
        Self::new_empty()
    }
}
