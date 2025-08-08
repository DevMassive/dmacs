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
        self.modify(at_x, at_y, &c.to_string(), "", false)
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
        self.modify(at_x, at_y, "", &char_to_delete, false)
            .map(|_| ())
    }

    pub fn insert_newline(&mut self, at_x: usize, at_y: usize) -> Result<()> {
        if at_y > self.lines.len() {
            return Err(DmacsError::Document(format!("Invalid line index: {at_y}")));
        }
        if at_y == self.lines.len() {
            self.lines.push(String::new());
            return Ok(());
        }
        let current_line = self
            .lines
            .get_mut(at_y)
            .ok_or(DmacsError::Document(format!("Invalid line index: {at_y}")))?;
        let new_line = current_line.split_off(at_x);
        self.lines.insert(at_y + 1, new_line);
        Ok(())
    }

    pub fn insert_string(&mut self, at_x: usize, at_y: usize, s: &str) -> Result<()> {
        self.modify(at_x, at_y, s, "", false).map(|_| ())
    }

    pub fn swap_lines(&mut self, y1: usize, y2: usize) {
        if y1 < self.lines.len() && y2 < self.lines.len() {
            self.lines.swap(y1, y2);
        }
    }

    pub fn join_line_with_previous(&mut self, at_y: usize) -> Result<()> {
        if at_y == 0 || at_y >= self.lines.len() {
            return Err(DmacsError::Document(format!(
                "Cannot join line {at_y} with previous line."
            )));
        }
        let current_line = self.lines.remove(at_y);
        self.lines[at_y - 1].push_str(&current_line);
        Ok(())
    }

    pub fn join_line_with_next(&mut self, at_y: usize) -> Result<()> {
        if at_y >= self.lines.len().saturating_sub(1) {
            return Err(DmacsError::Document(format!(
                "Cannot join line {at_y} with next line."
            )));
        }
        let next_line = self.lines.remove(at_y + 1);
        self.lines[at_y].push_str(&next_line);
        Ok(())
    }

    pub fn remove_line(&mut self, at_y: usize) -> Result<String> {
        if at_y >= self.lines.len() {
            return Err(DmacsError::Document(format!(
                "Invalid line index for removal: {at_y}"
            )));
        }
        Ok(self.lines.remove(at_y))
    }

    pub fn split_line_from(&mut self, at_x: usize, at_y: usize) -> Result<String> {
        if at_y >= self.lines.len() {
            return Err(DmacsError::Document(format!(
                "Invalid line index for split: {at_y}"
            )));
        }
        let line = &mut self.lines[at_y];
        if at_x > line.len() {
            return Err(DmacsError::Document(format!(
                "Invalid column index for split: {at_x}"
            )));
        }
        Ok(line.split_off(at_x))
    }

    pub fn delete_range(&mut self, at_x: usize, at_y: usize, end_x: usize) -> Result<()> {
        if at_y >= self.lines.len() {
            return Err(DmacsError::Document(format!(
                "Invalid line index for delete_range: {at_y}"
            )));
        }
        let line = &self.lines[at_y];
        if at_x > line.len() || end_x > line.len() || at_x > end_x {
            return Err(DmacsError::Document(format!(
                "Invalid range for delete_range: ({at_x}, {end_x}) on line length {}",
                line.len()
            )));
        }
        let deleted_text = line[at_x..end_x].to_string();
        self.modify(at_x, at_y, "", &deleted_text, false)
            .map(|_| ())
    }

    pub fn modify(
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
                    "Text to delete does not match: expected \"{}\", found \"{}\"",
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
}

impl Default for Document {
    fn default() -> Self {
        Self::new_empty()
    }
}
