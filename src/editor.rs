use pancurses::{A_DIM, Input, Window};

use unicode_width::UnicodeWidthChar;

use crate::document::Document;

const TAB_STOP: usize = 4;

// Editor state
pub struct Editor {
    pub should_quit: bool,
    pub document: Document,
    cursor_x: usize, // byte index
    cursor_y: usize,
    desired_cursor_x: usize, // column index
    status_message: String,
    pub row_offset: usize,                     // public for tests
    pub col_offset: usize,                     // public for tests
    undo_stack: Vec<(Document, usize, usize)>, // Stores (Document, cursor_x, cursor_y)
    pub kill_buffer: String,
    last_action_was_kill: bool,
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
            row_offset: 0,
            col_offset: 0,
            undo_stack: Vec::new(),
            kill_buffer: String::new(),
            last_action_was_kill: false,
        }
    }

    fn save_state_for_undo(&mut self) {
        self.undo_stack
            .push((self.document.clone(), self.cursor_x, self.cursor_y));
    }

    pub fn undo(&mut self) {
        self.last_action_was_kill = false;
        if let Some((prev_document, prev_cursor_x, prev_cursor_y)) = self.undo_stack.pop() {
            self.document = prev_document;
            self.cursor_x = prev_cursor_x;
            self.cursor_y = prev_cursor_y;
            self.desired_cursor_x =
                self.get_display_width(&self.document.lines[self.cursor_y], self.cursor_x);
            self.status_message = "Undo successful.".to_string();
        } else {
            self.status_message = "Nothing to undo.".to_string();
        }
    }

    pub fn handle_keypress(&mut self, key: Input) {
        match key {
            Input::Character(c) => match c {
                '\x18' => self.quit(),
                '\x13' => self.save_document(),
                '\x01' => self.go_to_start_of_line(),
                '\x05' => self.go_to_end_of_line(),
                '\x04' => self.delete_forward_char(),
                '\x0b' => {
                    self.kill_line();
                    self.last_action_was_kill = true;
                }
                '\x19' => self.yank(),                 // Ctrl + Y
                '\x7f' | '\x08' => self.delete_char(), // Backspace
                '\n' | '\r' => self.insert_newline(),
                '\x00' => {}
                '\x02' => self.move_cursor_word_left(), // Ctrl + B
                '\x06' => self.move_cursor_word_right(), // Ctrl + F
                '\x1f' => self.undo(),                  // Ctrl + _ for undo
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

    fn get_display_width(&self, line: &str, until_byte: usize) -> usize {
        let mut width = 0;
        let mut bytes = 0;
        for ch in line.chars() {
            if bytes >= until_byte {
                break;
            }
            if ch == '\t' {
                width += TAB_STOP - (width % TAB_STOP);
            } else {
                width += ch.width().unwrap_or(0);
            }
            bytes += ch.len_utf8();
        }
        width
    }

    pub fn scroll(&mut self, screen_cols: usize, screen_rows: usize) {
        // Vertical scroll
        if self.cursor_y < self.row_offset {
            self.row_offset = self.cursor_y;
        }
        if self.cursor_y >= self.row_offset + screen_rows {
            self.row_offset = self.cursor_y - screen_rows + 1;
        }

        // Horizontal scroll
        let display_cursor_x =
            self.get_display_width(&self.document.lines[self.cursor_y], self.cursor_x);
        if display_cursor_x < self.col_offset {
            self.col_offset = display_cursor_x;
        }
        if display_cursor_x >= self.col_offset + screen_cols {
            self.col_offset = display_cursor_x - screen_cols + 1;
        }
    }

    pub fn draw(&mut self, window: &Window) {
        let (screen_rows, screen_cols) =
            (window.get_max_y() as usize - 1, window.get_max_x() as usize);
        self.scroll(screen_cols, screen_rows);

        window.erase();

        // Draw text
        for (index, line) in self.document.lines.iter().enumerate() {
            if index < self.row_offset {
                continue;
            }
            let row = index - self.row_offset;
            if row >= screen_rows {
                break;
            }

            let is_comment = line.trim_start().starts_with('#');
            if is_comment {
                window.attron(A_DIM);
            }

            let mut display_x = 0;
            for ch in line.chars() {
                let char_start_display_x = display_x;

                // Calculate character width
                let char_width = if ch == '\t' {
                    TAB_STOP - (display_x % TAB_STOP)
                } else {
                    ch.width().unwrap_or(0)
                };
                display_x += char_width;

                // Draw character
                if display_x > self.col_offset {
                    let screen_x = char_start_display_x.saturating_sub(self.col_offset);
                    if screen_x < screen_cols {
                        if ch == '\t' {
                            // Draw a tab as spaces
                            for i in 0..char_width {
                                if screen_x + i < screen_cols {
                                    window.mvaddch(row as i32, (screen_x + i) as i32, ' ');
                                }
                            }
                        } else {
                            // Draw character
                            window.mvaddstr(row as i32, screen_x as i32, ch.to_string());
                        }
                    }
                }
                // Stop drawing if we reach the end of the screen
                if char_start_display_x.saturating_sub(self.col_offset) >= screen_cols {
                    break;
                }
            }
            if is_comment {
                window.attroff(A_DIM);
            }
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
        let display_cursor_x =
            self.get_display_width(&self.document.lines[self.cursor_y], self.cursor_x);
        window.mv(
            (self.cursor_y - self.row_offset) as i32,
            (display_cursor_x - self.col_offset) as i32,
        );
        window.refresh();
    }

    pub fn move_cursor_up(&mut self) {
        self.last_action_was_kill = false;
        if self.cursor_y > 0 {
            self.cursor_y -= 1;
            self.cursor_x = self.get_byte_pos_from_display_width(self.desired_cursor_x);
        } else {
            // If at the top line, move to the beginning of the line
            self.cursor_x = 0;
            self.desired_cursor_x = 0;
        }
    }

    pub fn move_cursor_down(&mut self) {
        self.last_action_was_kill = false;
        if self.cursor_y < self.document.lines.len() - 1 {
            self.cursor_y += 1;
            self.cursor_x = self.get_byte_pos_from_display_width(self.desired_cursor_x);
        } else {
            // If at the bottom line, move to the end of the line
            self.go_to_end_of_line();
        }
    }

    pub fn move_cursor_left(&mut self) {
        self.last_action_was_kill = false;
        if self.cursor_x > 0 {
            let line = &self.document.lines[self.cursor_y];
            let mut new_pos = self.cursor_x - 1;
            while !line.is_char_boundary(new_pos) {
                new_pos -= 1;
            }
            self.cursor_x = new_pos;
            self.desired_cursor_x = self.get_display_width(line, self.cursor_x);
        } else if self.cursor_y > 0 {
            self.cursor_y -= 1;
            self.cursor_x = self.document.lines[self.cursor_y].len();
            self.desired_cursor_x =
                self.get_display_width(&self.document.lines[self.cursor_y], self.cursor_x);
        }
    }

    pub fn move_cursor_right(&mut self) {
        self.last_action_was_kill = false;
        let line = &self.document.lines[self.cursor_y];
        if self.cursor_x < line.len() {
            let mut new_pos = self.cursor_x + 1;
            while !line.is_char_boundary(new_pos) {
                new_pos += 1;
            }
            self.cursor_x = new_pos;
            self.desired_cursor_x = self.get_display_width(line, self.cursor_x);
        } else if self.cursor_y < self.document.lines.len() - 1 {
            self.cursor_y += 1;
            self.cursor_x = 0;
            self.desired_cursor_x = 0;
        }
    }

    pub fn insert_char(&mut self, c: char) {
        self.last_action_was_kill = false;
        self.save_state_for_undo();
        self.document.insert(self.cursor_x, self.cursor_y, c);
        self.cursor_x += c.len_utf8();
        self.desired_cursor_x =
            self.get_display_width(&self.document.lines[self.cursor_y], self.cursor_x);
        self.status_message = "".to_string();
    }

    pub fn delete_char(&mut self) {
        self.last_action_was_kill = false;
        // Backspace
        self.save_state_for_undo();
        if self.cursor_x > 0 {
            self.move_cursor_left();
            self.document.delete(self.cursor_x, self.cursor_y);
        } else if self.cursor_y > 0 {
            let prev_line_len = self.document.lines[self.cursor_y - 1].len();
            let current_line = self.document.lines.remove(self.cursor_y);
            self.document.lines[self.cursor_y - 1].push_str(&current_line);
            self.cursor_y -= 1;
            self.cursor_x = prev_line_len;
            self.desired_cursor_x =
                self.get_display_width(&self.document.lines[self.cursor_y], self.cursor_x);
        }
    }

    pub fn delete_forward_char(&mut self) {
        self.last_action_was_kill = false;
        // Ctrl-D
        self.save_state_for_undo();
        let y = self.cursor_y;
        let x = self.cursor_x;
        let line_len = self.document.lines.get(y).map_or(0, |l| l.len());
        if x < line_len {
            self.document.delete(x, y);
        } else if y < self.document.lines.len() - 1 {
            let next_line = self.document.lines.remove(y + 1);
            self.document.lines[y].push_str(&next_line);
        }
    }

    pub fn insert_newline(&mut self) {
        self.last_action_was_kill = false;
        self.save_state_for_undo();
        self.document.insert_newline(self.cursor_x, self.cursor_y);
        self.cursor_y += 1;
        self.cursor_x = 0;
        self.desired_cursor_x = 0;
    }

    pub fn kill_line(&mut self) {
        self.save_state_for_undo();
        let y = self.cursor_y;
        let x = self.cursor_x;
        if y >= self.document.lines.len() {
            return;
        }

        let current_line_len = self.document.lines[y].len();
        if !self.last_action_was_kill {
            self.kill_buffer.clear();
        }

        if x == 0 && current_line_len == 0 && y < self.document.lines.len() - 1 {
            // Case 3: Cursor is at the beginning of an empty line, and it's not the last line
            // Kill the newline and remove the empty line
            self.document.lines.remove(y); // Remove the empty line
            self.kill_buffer.push('\n');
        } else if x < current_line_len {
            // Case 1: Cursor is within the line (not at the very end)
            // Kill from cursor to end of line
            let killed_text = self.document.lines[y].split_off(x);
            self.kill_buffer.push_str(&killed_text);
        } else if x == current_line_len && y < self.document.lines.len() - 1 {
            // Case 2: Cursor is at the end of the line, and it's not the last line
            // Kill the newline and join with the next line
            let next_line_content = self.document.lines.remove(y + 1);
            self.document.lines[y].push_str(&next_line_content);
            self.kill_buffer.push('\n');
            self.kill_buffer.push_str(&next_line_content);
        }
        self.last_action_was_kill = true;
    }

    pub fn yank(&mut self) {
        self.last_action_was_kill = false;
        self.save_state_for_undo();
        let text_to_yank = self.kill_buffer.clone();
        let mut current_x = self.cursor_x;
        let mut current_y = self.cursor_y;

        let lines_to_yank: Vec<&str> = text_to_yank.split('\n').collect();

        if lines_to_yank.is_empty() {
            return;
        }

        // Insert the first part of the yanked text into the current line
        self.document
            .insert_string(current_x, current_y, lines_to_yank[0]);
        current_x += lines_to_yank[0].len();

        // Insert subsequent lines
        for line_to_yank in lines_to_yank.iter().skip(1) {
            self.document.insert_newline(current_x, current_y);
            current_y += 1;
            current_x = 0;
            self.document
                .insert_string(current_x, current_y, line_to_yank);
            current_x += line_to_yank.len();
        }

        self.cursor_x = current_x;
        self.cursor_y = current_y;
        self.desired_cursor_x =
            self.get_display_width(&self.document.lines[self.cursor_y], self.cursor_x);
    }

    pub fn hungry_delete(&mut self) {
        self.save_state_for_undo();
        let (x, y) = (self.cursor_x, self.cursor_y);
        if y >= self.document.lines.len() {
            return;
        }

        let current_line = &mut self.document.lines[y];
        if x == 0 && y == 0 {
            return;
        } // Cannot delete before start of document

        if x == 0 {
            // At the beginning of a line, join with previous line
            if y > 0 {
                let prev_line_len = self.document.lines[y - 1].len();
                let current_line_content = self.document.lines.remove(y);
                self.document.lines[y - 1].push_str(&current_line_content);
                self.cursor_y -= 1;
                self.cursor_x = prev_line_len;
                self.desired_cursor_x =
                    self.get_display_width(&self.document.lines[self.cursor_y], self.cursor_x);
            }
            return;
        }

        let mut chars_to_left = current_line[..x].char_indices().rev();
        let mut start_delete_byte = x;

        if let Some((idx, first_char_to_left)) = chars_to_left.next() {
            if first_char_to_left.is_whitespace() {
                // Delete all preceding whitespace
                start_delete_byte = idx; // Start of the first whitespace char
                for (idx, ch) in chars_to_left {
                    if ch.is_whitespace() {
                        start_delete_byte = idx;
                    } else {
                        break;
                    }
                }
            } else {
                // Delete the word to the left and any preceding whitespace
                start_delete_byte = idx; // Start of the first non-whitespace char
                // Find the beginning of the word
                for (idx, ch) in chars_to_left.by_ref() {
                    // Use by_ref to continue iteration
                    if !ch.is_whitespace() {
                        start_delete_byte = idx;
                    } else {
                        break;
                    }
                }
                // Now find the beginning of any preceding whitespace
                for (idx, ch) in chars_to_left {
                    // Continue from where the previous loop left off
                    if ch.is_whitespace() {
                        start_delete_byte = idx;
                    } else {
                        break;
                    }
                }
            }
        }

        current_line.replace_range(start_delete_byte..x, "");
        self.cursor_x = start_delete_byte;
        self.desired_cursor_x =
            self.get_display_width(&self.document.lines[self.cursor_y], self.cursor_x);
    }

    pub fn go_to_start_of_line(&mut self) {
        self.last_action_was_kill = false;
        self.cursor_x = 0;
        self.desired_cursor_x = 0;
    }

    pub fn go_to_end_of_line(&mut self) {
        self.last_action_was_kill = false;
        let y = self.cursor_y;
        self.cursor_x = self.document.lines[y].len();
        self.desired_cursor_x = self.get_display_width(&self.document.lines[y], self.cursor_x);
    }

    pub fn move_cursor_word_left(&mut self) {
        self.last_action_was_kill = false;
        let current_line = &self.document.lines[self.cursor_y];
        let mut new_cursor_x = self.cursor_x;

        if new_cursor_x == 0 {
            if self.cursor_y > 0 {
                self.cursor_y -= 1;
                self.cursor_x = self.document.lines[self.cursor_y].len();
                self.desired_cursor_x =
                    self.get_display_width(&self.document.lines[self.cursor_y], self.cursor_x);
            }
            return;
        }

        let mut char_indices = current_line[..new_cursor_x].char_indices().rev();

        // Skip any trailing non-word characters (whitespace, punctuation)
        for (idx, ch) in char_indices.by_ref() {
            if is_word_char(ch) {
                new_cursor_x = idx + ch.len_utf8(); // Move past the non-word char
                break;
            }
            new_cursor_x = idx;
        }

        // Skip word characters
        for (idx, ch) in char_indices {
            if !is_word_char(ch) {
                new_cursor_x = idx + ch.len_utf8(); // Move past the word char
                break;
            }
            new_cursor_x = idx;
        }

        self.cursor_x = new_cursor_x;
        self.desired_cursor_x = self.get_display_width(current_line, self.cursor_x);
    }

    pub fn move_cursor_word_right(&mut self) {
        self.last_action_was_kill = false;
        let current_line = &self.document.lines[self.cursor_y];
        let mut new_cursor_x = self.cursor_x;
        let line_len = current_line.len();

        if new_cursor_x == line_len {
            if self.cursor_y < self.document.lines.len() - 1 {
                self.cursor_y += 1;
                self.cursor_x = 0;
                self.desired_cursor_x = 0;
            }
            return;
        }

        // Find the start of the next word
        let mut found_word_start = false;
        for (idx, ch) in current_line[new_cursor_x..].char_indices() {
            if is_word_char(ch) {
                new_cursor_x += idx; // Move to the start of the word
                found_word_start = true;
                break;
            }
        }

        // If no word found after current position, move to end of line
        if !found_word_start {
            self.cursor_x = line_len;
            self.desired_cursor_x = self.get_display_width(current_line, self.cursor_x);
            return;
        }

        // Find the end of the current word
        let chars_from_word_start = current_line[new_cursor_x..].chars();
        for ch in chars_from_word_start {
            if !is_word_char(ch) {
                break; // Found non-word character, so word ends before this
            }
            new_cursor_x += ch.len_utf8();
        }

        self.cursor_x = new_cursor_x;
        self.desired_cursor_x = self.get_display_width(current_line, self.cursor_x);
    }

    pub fn save_document(&mut self) {
        self.last_action_was_kill = false;
        if self.document.save().is_ok() {
            self.status_message = "File saved successfully.".to_string();
        } else {
            self.status_message = "Error saving file!".to_string();
        }
    }

    pub fn quit(&mut self) {
        self.last_action_was_kill = false;
        self.document.save().ok();
        self.should_quit = true;
    }

    fn clamp_cursor_x(&mut self) {
        if self.cursor_y >= self.document.lines.len() {
            self.cursor_x = 0;
            return;
        }
        let line_len = self.document.lines[self.cursor_y].len();
        if self.cursor_x > line_len {
            self.cursor_x = line_len;
        }
    }

    fn get_byte_pos_from_display_width(&self, display_x: usize) -> usize {
        let line = &self.document.lines[self.cursor_y];
        let mut current_display_x = 0;
        let mut byte_pos = 0;
        for ch in line.chars() {
            if current_display_x >= display_x {
                break;
            }
            if ch == '\t' {
                current_display_x += TAB_STOP - (current_display_x % TAB_STOP);
            } else {
                current_display_x += ch.width().unwrap_or(0);
            }
            if current_display_x > display_x {
                break;
            }
            byte_pos += ch.len_utf8();
        }
        byte_pos
    }

    pub fn cursor_pos(&self) -> (usize, usize) {
        (self.cursor_x, self.cursor_y)
    }

    pub fn set_cursor_pos(&mut self, x: usize, y: usize) {
        self.cursor_x = x;
        self.cursor_y = y;
        self.clamp_cursor_x();
    }

    pub fn set_message(&mut self, message: &str) {
        self.status_message = message.to_string();
    }

    pub fn move_line_up(&mut self) {
        self.last_action_was_kill = false;
        if self.cursor_y > 0 {
            self.save_state_for_undo();
            self.document.swap_lines(self.cursor_y, self.cursor_y - 1);
            self.cursor_y -= 1;
            self.status_message = "Line moved up.".to_string();
        } else {
            self.status_message = "Cannot move line up further.".to_string();
        }
    }

    pub fn move_line_down(&mut self) {
        self.last_action_was_kill = false;
        if self.cursor_y < self.document.lines.len() - 1 {
            self.save_state_for_undo();
            self.document.swap_lines(self.cursor_y, self.cursor_y + 1);
            self.cursor_y += 1;
            self.status_message = "Line moved down.".to_string();
        } else {
            self.status_message = "Cannot move line down further.".to_string();
        }
    }
}

fn is_word_char(ch: char) -> bool {
    ch.is_alphanumeric() || ch == '_'
}
