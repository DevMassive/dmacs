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
    NewlineInsertion { x: usize, y: usize },
    NewlineDeletion { x: usize, y: usize },
    LineSwap { y1: usize, y2: usize },
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

    pub fn insert(&mut self, at_x: usize, at_y: usize, c: char) -> Result<()> {
        if at_y > self.lines.len() {
            return Err(DmacsError::Document(
                "Invalid line index: {at_y}".to_string(),
            ));
        }
        if at_y == self.lines.len() {
            self.lines.push(String::new());
        }
        let diff = Diff {
            x: at_x,
            y: at_y,
            added_text: c.to_string(),
            deleted_text: "".to_string(),
        };
        self.modify_single_char(&diff, false).map(|_| ())
    }

    pub fn delete(&mut self, at_x: usize, at_y: usize) -> Result<()> {
        if at_y >= self.lines.len() {
            return Err(DmacsError::Document(
                "Invalid line index: {at_y}".to_string(),
            ));
        }
        let line = &self.lines[at_y];
        if at_x >= line.len() {
            return Err(DmacsError::Document(
                "Invalid column index: {at_x}".to_string(),
            ));
        }
        let char_to_delete = line.chars().nth(at_x).unwrap().to_string();
        if char_to_delete == "\n" {
            self.delete_newline(at_x, at_y).map(|_| ())
        } else {
            let diff = Diff {
                x: at_x,
                y: at_y,
                added_text: "".to_string(),
                deleted_text: char_to_delete,
            };
            self.modify_single_char(&diff, false).map(|_| ())
        }
    }

    pub fn insert_string(&mut self, mut x: usize, mut y: usize, s: &str) -> Result<()> {
        for c in s.chars() {
            let (new_x, new_y) = if c == '\n' {
                self.insert_newline(x, y)?
            } else {
                let diff = Diff {
                    x,
                    y,
                    added_text: c.to_string(),
                    deleted_text: "".to_string(),
                };
                self.modify_single_char(&diff, false)?
            };
            x = new_x;
            y = new_y;
        }
        Ok(())
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

        let line = &mut self.lines[diff.y];

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
                // If inserting beyond the current line length, pad with spaces
                // This might not be the desired behavior for all cases, but it's a start.
                // Consider if this should be an error or if the line should be extended.
                line.push_str(&" ".repeat(diff.x - line.len()));
            }
            line.insert_str(diff.x, add);
        }

        // Calculate new cursor position
        let new_x = diff.x + add.len();
        let new_y = diff.y;

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
