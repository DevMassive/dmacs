use pancurses::{Input, Window};
use std::io::{self, Write};

// Document being edited
pub struct Document {
    pub lines: Vec<String>,
    pub filename: Option<String>,
}

impl Document {
    pub fn open(filename: &str) -> io::Result<Self> {
        let content = std::fs::read_to_string(filename)?;
        let lines = content.lines().map(|s| s.to_string()).collect();
        Ok(Self {
            lines,
            filename: Some(filename.to_string()),
        })
    }

    pub fn default() -> Self {
        Self {
            lines: vec!["".to_string()],
            filename: None,
        }
    }

    pub fn save(&self) -> io::Result<()> {
        if let Some(filename) = &self.filename {
            let mut file = std::fs::File::create(filename)?;
            for line in &self.lines {
                writeln!(file, "{}", line)?;
            }
        }
        Ok(())
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
}

// Editor state
pub struct Editor {
    pub should_quit: bool,
    pub document: Document,
    cursor_x: i32,
    cursor_y: i32,
    desired_cursor_x: i32,
    status_message: String,
}

impl Editor {
    pub fn new(filename: Option<String>) -> Self {
        let document = match filename {
            Some(fname) => {
                if let Ok(doc) = Document::open(&fname) {
                    doc
                } else {
                    Document {
                        lines: vec!["".to_string()],
                        filename: Some(fname),
                    }
                }
            }
            None => Document::default(),
        };

        Self {
            should_quit: false,
            document,
            cursor_x: 0,
            cursor_y: 0,
            desired_cursor_x: 0,
            status_message: "".to_string(),
        }
    }

    pub fn handle_keypress(&mut self, key: Input) {
        match key {
            Input::Character(c) => match c {
                '' => self.quit(),
                '' => self.save_document(),
                '' => self.go_to_start_of_line(),
                '' => self.go_to_end_of_line(),
                '' => self.delete_forward_char(),
                '\x0A' => self.insert_newline(),
                _ => self.insert_char(c),
            },
            Input::KeyBackspace => self.delete_char(),
            Input::KeyUp => self.move_cursor_up(),
            Input::KeyDown => self.move_cursor_down(),
            Input::KeyLeft => self.move_cursor_left(),
            Input::KeyRight => self.move_cursor_right(),
            _ => {}
        }
        self.clamp_cursor_x();
    }

    pub fn draw(&self, window: &Window) {
        window.erase();

        // Draw text
        for (index, line) in self.document.lines.iter().enumerate() {
            window.mvaddstr(index as i32, 0, line);
        }

        // Draw status bar
        let status_bar = format!(
            "{} - {} lines | {}",
            self.document.filename.as_deref().unwrap_or("[No Name]"),
            self.document.lines.len(),
            self.status_message
        );
        window.mvaddstr(window.get_max_y() - 1, 0, &status_bar);

        // Move cursor
        window.mv(self.cursor_y, self.cursor_x);
        window.refresh();
    }

    pub fn move_cursor_up(&mut self) {
        if self.cursor_y > 0 {
            self.cursor_y -= 1;
        }
    }

    pub fn move_cursor_down(&mut self) {
        if self.cursor_y < self.document.lines.len() as i32 - 1 {
            self.cursor_y += 1;
        }
    }

    pub fn move_cursor_left(&mut self) {
        if self.cursor_x > 0 {
            self.cursor_x -= 1;
            self.desired_cursor_x = self.cursor_x;
        }
    }

    pub fn move_cursor_right(&mut self) {
        let line_len = self.document.lines[self.cursor_y as usize].len() as i32;
        if self.cursor_x < line_len {
            self.cursor_x += 1;
            self.desired_cursor_x = self.cursor_x;
        }
    }

    pub fn insert_char(&mut self, c: char) {
        self.document.insert(self.cursor_x as usize, self.cursor_y as usize, c);
        self.cursor_x += 1;
        self.desired_cursor_x = self.cursor_x;
        self.status_message = "".to_string();
    }

    pub fn delete_char(&mut self) { // Backspace
        if self.cursor_x > 0 {
            self.cursor_x -= 1;
            self.document.delete(self.cursor_x as usize, self.cursor_y as usize);
            self.desired_cursor_x = self.cursor_x;
        } else if self.cursor_y > 0 {
            let prev_line_len = self.document.lines[self.cursor_y as usize - 1].len();
            let current_line = self.document.lines.remove(self.cursor_y as usize);
            self.document.lines[self.cursor_y as usize - 1].push_str(&current_line);
            self.cursor_y -= 1;
            self.cursor_x = prev_line_len as i32;
            self.desired_cursor_x = self.cursor_x;
        }
    }

    pub fn delete_forward_char(&mut self) { // Ctrl-D
        let y = self.cursor_y as usize;
        let x = self.cursor_x as usize;
        let line_len = self.document.lines.get(y).map_or(0, |l| l.len());
        if x < line_len {
            self.document.delete(x, y);
        } else if y < self.document.lines.len() - 1 {
            let next_line = self.document.lines.remove(y + 1);
            self.document.lines[y].push_str(&next_line);
        }
    }

    pub fn insert_newline(&mut self) {
        self.document.insert_newline(self.cursor_x as usize, self.cursor_y as usize);
        self.cursor_y += 1;
        self.cursor_x = 0;
        self.desired_cursor_x = 0;
    }

    pub fn go_to_start_of_line(&mut self) {
        self.cursor_x = 0;
        self.desired_cursor_x = 0;
    }

    pub fn go_to_end_of_line(&mut self) {
        let y = self.cursor_y as usize;
        self.cursor_x = self.document.lines[y].len() as i32;
        self.desired_cursor_x = self.cursor_x;
    }

    pub fn save_document(&mut self) {
        if self.document.save().is_ok() {
            self.status_message = "File saved successfully.".to_string();
        } else {
            self.status_message = "Error saving file!".to_string();
        }
    }

    pub fn quit(&mut self) {
        self.document.save().ok();
        self.should_quit = true;
    }

    fn clamp_cursor_x(&mut self) {
        let y = self.cursor_y as usize;
        if y < self.document.lines.len() {
            let line_len = self.document.lines[y].len() as i32;
            self.cursor_x = self.desired_cursor_x.min(line_len);
        }
    }

    pub fn cursor_pos(&self) -> (i32, i32) {
        (self.cursor_x, self.cursor_y)
    }
}
