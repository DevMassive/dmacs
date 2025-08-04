use std::io::{self, Write};

// Document being edited
#[derive(Clone)]
pub struct Document {
    pub lines: Vec<String>,
    pub filename: Option<String>,
    original_content: Option<String>,
}

impl Document {
    pub fn open(filename: &str) -> io::Result<Self> {
        let content = std::fs::read_to_string(filename)?;
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

    pub fn save(&mut self) -> io::Result<()> {
        if let Some(filename) = &self.filename {
            let mut file = std::fs::File::create(filename)?;
            for line in &self.lines {
                writeln!(file, "{line}")?;
            }
            let mut file = std::fs::File::create(filename)?;
            for line in &self.lines {
                writeln!(file, "{line}")?;
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
        let current_content = self.lines.join("\n");
        let current_content_with_newline =
            if !self.lines.is_empty() && !current_content.ends_with('\n') {
                current_content + "\n"
            } else {
                current_content
            };
        self.original_content
            .as_ref()
            .is_none_or(|orig| *orig != current_content_with_newline)
    }

    pub fn insert(&mut self, at_x: usize, at_y: usize, c: char) {
        if at_y > self.lines.len() {
            return;
        }
        if at_y == self.lines.len() {
            self.lines.push(String::new());
        }
        let line = self.lines.get_mut(at_y).unwrap();
        if at_x > line.len() {
            line.push(c);
        } else {
            line.insert(at_x, c);
        }
    }

    pub fn delete(&mut self, at_x: usize, at_y: usize) {
        if at_y >= self.lines.len() {
            return;
        }
        let line = self.lines.get_mut(at_y).unwrap();
        if at_x >= line.len() {
            return;
        }
        line.remove(at_x);
    }

    pub fn insert_newline(&mut self, at_x: usize, at_y: usize) {
        if at_y > self.lines.len() {
            return;
        }
        if at_y == self.lines.len() {
            self.lines.push(String::new());
            return;
        }
        let current_line = self.lines.get_mut(at_y).unwrap();
        let new_line = current_line.split_off(at_x);
        self.lines.insert(at_y + 1, new_line);
    }

    pub fn insert_string(&mut self, at_x: usize, at_y: usize, s: &str) {
        if at_y >= self.lines.len() {
            return;
        }
        let line = &mut self.lines[at_y];
        line.insert_str(at_x, s);
    }

    pub fn swap_lines(&mut self, y1: usize, y2: usize) {
        if y1 < self.lines.len() && y2 < self.lines.len() {
            self.lines.swap(y1, y2);
        }
    }
}

impl Default for Document {
    fn default() -> Self {
        Self::new_empty()
    }
}
