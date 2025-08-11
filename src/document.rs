use crate::error::{DmacsError, Result};
use std::io::Write;

// Document being edited
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
            for line in &self.lines {
                writeln!(file, "{line}").map_err(DmacsError::Io)?;
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
            return Err(DmacsError::Document(format!("Invalid line index: {at_y}")));
        }
        if at_y == self.lines.len() {
            self.lines.push(String::new());
        }
        self.modify_single_char(at_x, at_y, &c.to_string(), "", false)
            .map(|_| ())
    }

    pub fn delete(&mut self, at_x: usize, at_y: usize) -> Result<()> {
        if at_y >= self.lines.len() {
            return Err(DmacsError::Document(format!("Invalid line index: {at_y}")));
        }
        let line = &self.lines[at_y];
        if at_x >= line.len() {
            return Err(DmacsError::Document(format!(
                "Invalid column index: {at_x}"
            )));
        }
        let char_to_delete = line.chars().nth(at_x).unwrap().to_string();
        if char_to_delete == "\n" {
            self.delete_newline(at_x, at_y, false).map(|_| ())
        } else {
            self.modify_single_char(at_x, at_y, "", &char_to_delete, false)
                .map(|_| ())
        }
    }

    pub fn insert_string(&mut self, mut x: usize, mut y: usize, s: &str) -> Result<()> {
        for c in s.chars() {
            let (new_x, new_y) = if c == '\n' {
                self.insert_newline(x, y, false)?
            } else {
                self.modify_single_char(x, y, &c.to_string(), "", false)?
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

    pub fn modify_single_char(
        &mut self,
        x: usize,
        y: usize,
        added_text: &str,
        deleted_text: &str,
        is_undo: bool,
    ) -> Result<(usize, usize)> {
        let (add, delete) = if is_undo {
            (deleted_text, added_text)
        } else {
            (added_text, deleted_text)
        };

        if y >= self.lines.len() {
            return Err(DmacsError::Document(format!("Invalid line index: {y}")));
        }

        let line = &mut self.lines[y];

        // Handle deletion
        if !delete.is_empty() {
            let delete_len = delete.len();
            if x + delete_len > line.len() {
                return Err(DmacsError::Document(format!(
                    "Deletion out of bounds: x={x}, delete_len={delete_len}, line_len={}",
                    line.len()
                )));
            }
            if line[x..].starts_with(delete) {
                line.replace_range(x..(x + delete_len), "");
            } else {
                return Err(DmacsError::Document(format!(
                    "Text to delete does not match: expected \"{}}}\", found \"{}}}\"",
                    delete,
                    &line[x..(x + delete_len)]
                )));
            }
        }

        // Handle insertion
        if !add.is_empty() {
            if x > line.len() {
                // If inserting beyond the current line length, pad with spaces
                // This might not be the desired behavior for all cases, but it's a start.
                // Consider if this should be an error or if the line should be extended.
                line.push_str(&" ".repeat(x - line.len()));
            }
            line.insert_str(x, add);
        }

        // Calculate new cursor position
        let new_x = x + add.len();
        let new_y = y;

        Ok((new_x, new_y))
    }
    pub fn insert_newline(&mut self, x: usize, y: usize, is_undo: bool) -> Result<(usize, usize)> {
        let (new_x, new_y) = if is_undo {
            // Undo newline insertion means joining lines
            if y == 0 {
                // Cannot undo newline insertion at the very beginning of the document
                return Err(DmacsError::Document(
                    "Cannot undo newline insertion at the beginning of the document.".to_string(),
                ));
            }
            let current_line = self.lines.remove(y);
            let prev_line_len = self.lines[y - 1].len();
            self.lines[y - 1].push_str(&current_line);
            (prev_line_len, y - 1)
        } else {
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
            (0, y + 1)
        };
        Ok((new_x, new_y))
    }
    pub fn delete_newline(&mut self, x: usize, y: usize, is_undo: bool) -> Result<(usize, usize)> {
        let (new_x, new_y) = if is_undo {
            // Undo newline deletion means inserting a newline
            self.insert_newline(x, y, false)?
        } else {
            // Handle newline deletion (joining lines)
            if x == 0 {
                // Backspace at the beginning of a line, join with previous
                if y == 0 {
                    // If it's the first line and we're deleting a newline at x=0,
                    // it means we're effectively removing the first line.
                    // This happens when you delete the newline *after* the first line.
                    if self.lines.len() > 1 {
                        self.lines.remove(y); // Remove the first line
                        (0, 0) // Cursor moves to the beginning of the new first line
                    } else {
                        // If it's the only line, just clear it.
                        self.lines[y].clear();
                        (0, 0)
                    }
                } else {
                    let current_line = self.lines.remove(y);
                    let prev_line_len = self.lines[y - 1].len();
                    self.lines[y - 1].push_str(&current_line);
                    (prev_line_len, y - 1)
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
                (current_line_len, y)
            }
        };
        Ok((new_x, new_y))
    }
}

impl Default for Document {
    fn default() -> Self {
        Self::new_empty()
    }
}
