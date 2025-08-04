use pancurses::{A_DIM, A_REVERSE, Input, Window};

use unicode_width::UnicodeWidthChar;

use crate::document::Document;

const TAB_STOP: usize = 4;

// Editor state
pub struct Search {
    pub mode: bool,
    pub query: String,
    pub results: Vec<(usize, usize)>,
    pub current_match_index: Option<usize>,
}

impl Search {
    pub fn new() -> Self {
        Self {
            mode: false,
            query: String::new(),
            results: Vec::new(),
            current_match_index: None,
        }
    }
}

pub struct Editor {
    pub should_quit: bool,
    pub document: Document,
    cursor_x: usize, // byte index
    cursor_y: usize,
    desired_cursor_x: usize, // column index
    pub status_message: String,
    pub row_offset: usize,                     // public for tests
    pub col_offset: usize,                     // public for tests
    undo_stack: Vec<(Document, usize, usize)>, // Stores (Document, cursor_x, cursor_y)
    pub kill_buffer: String,
    last_action_was_kill: bool,
    pub screen_rows: usize,
    pub screen_cols: usize,
    pub search: Search,
    pub previous_status_message: String,
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
            screen_rows: 0,
            screen_cols: 0,
            search: Search::new(),
            previous_status_message: String::new(),
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

    pub fn process_input(&mut self, key: Input, next_key: Option<Input>, third_key: Option<Input>) {
        if self.search.mode {
            self.handle_search_input(key);
            return;
        }

        match key {
            pancurses::Input::Character('\x1b') => {
                // Escape key, potential start of Alt/Option sequence
                if let Some(next_key_val) = next_key {
                    match next_key_val {
                        pancurses::Input::Character('v') => self.scroll_page_up(), // Alt/Option + V for page up (often sends ESC v)
                        pancurses::Input::Character('b') => self.move_cursor_word_left(), // Alt/Option + Left Arrow (often sends ESC b)
                        pancurses::Input::Character('f') => self.move_cursor_word_right(), // Alt/Option + Right Arrow (often sends ESC f)
                        pancurses::Input::Character('[') => {
                            if let Some(third_key_val) = third_key {
                                match third_key_val {
                                    pancurses::Input::Character('A') => self.move_line_up(), // Alt/Option + Up Arrow (often sends ESC [A)
                                    pancurses::Input::Character('B') => self.move_line_down(), // Alt/Option + Down Arrow (often sends ESC [B)
                                    _ => self.handle_keypress(pancurses::Input::Character('\x1b')), // Pass Escape if not a recognized sequence
                                }
                            } else {
                                self.handle_keypress(pancurses::Input::Character('\x1b')); // Pass Escape if no third key
                            }
                        }
                        pancurses::Input::Character('\x7f') | pancurses::Input::KeyBackspace => {
                            self.hungry_delete()
                        } // Alt/Option + Backspace
                        _ => self.handle_keypress(pancurses::Input::Character('\x1b')), // Pass Escape if not followed by Backspace
                    }
                } else {
                    self.handle_keypress(pancurses::Input::Character('\x1b')); // If no next_key, treat as plain Escape
                }
            }
            _ => self.handle_keypress(key),
        }
    }

    pub fn handle_keypress(&mut self, key: Input) {
        match key {
            Input::Character(c) => match c {
                '\x18' => self.quit(),
                '\x13' => self.enter_search_mode(), // Ctrl + S
                '\x01' => self.go_to_start_of_line(),
                '\x05' => self.go_to_end_of_line(),
                '\x04' => self.delete_forward_char(),
                '\x0b' => {
                    self.kill_line();
                    self.last_action_was_kill = true;
                }
                '\x19' => self.yank(),                 // Ctrl + Y
                '\x16' => self.scroll_page_down(),     // Ctrl + V
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

    fn enter_search_mode(&mut self) {
        self.search.mode = true;
        self.search.query.clear();
        self.search.results.clear();
        self.search.current_match_index = None;
        self.previous_status_message = self.status_message.clone(); // Save current status message
        self.status_message = "Search: ".to_string();
    }

    fn handle_search_input(&mut self, key: Input) {
        if let Input::Character(c) = key {
            match c {
                '\x1b' => {
                    // Escape key to exit search mode
                    self.search.mode = false;
                    self.search.query.clear();
                    self.search.results.clear();
                    self.search.current_match_index = None;
                    self.status_message = self.previous_status_message.clone(); // Restore previous status message
                }
                '\n' | '\r' => {
                    // Enter to confirm search and exit search mode
                    self.search.mode = false;
                    self.search.query.clear();
                    self.search.results.clear();
                    self.search.current_match_index = None;
                    self.status_message = self.previous_status_message.clone(); // Restore previous status message
                }
                '\x13' => {
                    // Ctrl + S for next match
                    self.move_to_next_match();
                }
                '\x0e' => {
                    // Ctrl + N for next match (already handled, but keeping for consistency)
                    self.move_to_next_match();
                }
                '\x10' => {
                    // Ctrl + P for previous match
                    self.move_to_prev_match();
                }
                '\x7f' | '\x08' => {
                    // Backspace
                    self.search.query.pop();
                    self.search();
                }
                _ => {
                    self.search.query.push(c);
                    self.search();
                }
            }
        }
        if self.search.mode {
            self.status_message = format!(
                "Search: {}{}",
                self.search.query,
                if self.search.query.is_empty() {
                    ""
                } else if self.search.results.is_empty() {
                    " (No match)"
                } else {
                    ""
                }
            );
        }
    }

    fn search(&mut self) {
        self.search.results.clear();
        self.search.current_match_index = None;

        if self.search.query.is_empty() {
            return;
        }

        for (row_idx, line) in self.document.lines.iter().enumerate() {
            for (col_idx, _) in line.match_indices(&self.search.query) {
                self.search.results.push((row_idx, col_idx));
            }
        }

        if !self.search.results.is_empty() {
            // Try to find a match from the current cursor position onwards
            let current_pos = (self.cursor_y, self.cursor_x);
            let mut found_current_or_next = false;
            for (i, &(row, col)) in self.search.results.iter().enumerate() {
                if row > current_pos.0 || (row == current_pos.0 && col >= current_pos.1) {
                    self.search.current_match_index = Some(i);
                    self.move_to_match();
                    found_current_or_next = true;
                    break;
                }
            }
            if !found_current_or_next {
                // If no match found after current position, wrap around to the first match
                self.search.current_match_index = Some(0);
                self.move_to_match();
            }
        }
    }

    fn move_to_match(&mut self) {
        if let Some(index) = self.search.current_match_index {
            if let Some(&(row, col)) = self.search.results.get(index) {
                self.cursor_y = row;
                self.cursor_x = col;
                self.desired_cursor_x =
                    self.get_display_width(&self.document.lines[self.cursor_y], self.cursor_x);
            }
        }
    }

    fn move_to_next_match(&mut self) {
        if self.search.results.is_empty() {
            return;
        }
        let next_index = match self.search.current_match_index {
            Some(idx) => (idx + 1) % self.search.results.len(),
            None => 0,
        };
        self.search.current_match_index = Some(next_index);
        self.move_to_match();
    }

    fn move_to_prev_match(&mut self) {
        if self.search.results.is_empty() {
            return;
        }
        let prev_index = match self.search.current_match_index {
            Some(idx) => {
                if idx == 0 {
                    self.search.results.len() - 1
                } else {
                    idx - 1
                }
            }
            None => self.search.results.len() - 1,
        };
        self.search.current_match_index = Some(prev_index);
        self.move_to_match();
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
        self.scroll(self.screen_cols, self.screen_rows - 1);

        window.erase();

        // Draw text
        for (index, line) in self.document.lines.iter().enumerate() {
            if index < self.row_offset {
                continue;
            }
            let row = index - self.row_offset;
            if row >= self.screen_rows.saturating_sub(1) {
                // Account for status bar
                break;
            }

            let is_comment = line.trim_start().starts_with('#');
            if is_comment {
                window.attron(A_DIM);
            }

            let mut display_x = 0;
            let mut byte_idx = 0;
            for ch in line.chars() {
                let char_start_display_x = display_x;

                // Calculate character width
                let char_width = if ch == '\t' {
                    TAB_STOP - (display_x % TAB_STOP)
                } else {
                    ch.width().unwrap_or(0)
                };
                display_x += char_width;

                // Check if this character is part of a search result
                let is_highlighted = self.search.mode
                    && self.search.results.iter().any(|&(r, c)| {
                        r == index && byte_idx >= c && byte_idx < c + self.search.query.len()
                    });

                if is_highlighted {
                    window.attron(A_REVERSE);
                }

                // Draw character
                if display_x > self.col_offset {
                    let screen_x = char_start_display_x.saturating_sub(self.col_offset);
                    if screen_x < self.screen_cols {
                        if ch == '\t' {
                            // Draw a tab as spaces
                            for i in 0..char_width {
                                if screen_x + i < self.screen_cols {
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
                if char_start_display_x.saturating_sub(self.col_offset) >= self.screen_cols {
                    break;
                }

                if is_highlighted {
                    window.attroff(A_REVERSE);
                }
                byte_idx += ch.len_utf8();
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

        if x == 0 {
            // If at the beginning of a line, join with previous line if available
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

        let start_delete_byte = find_word_boundary_left(current_line, x);

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

    pub fn update_screen_size(&mut self, rows: usize, cols: usize) {
        self.screen_rows = rows;
        self.screen_cols = cols;
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

    pub fn scroll_page_down(&mut self) {
        self.last_action_was_kill = false;
        let page_height = self.screen_rows.saturating_sub(1).max(1); // Usable screen height, ensure at least 1
        self.row_offset = self.row_offset.saturating_add(page_height);
        self.row_offset = self
            .row_offset
            .min(self.document.lines.len().saturating_sub(1));
        self.cursor_y = self.row_offset; // Set cursor to the top of the new view
        self.clamp_cursor_x();
    }

    pub fn scroll_page_up(&mut self) {
        self.last_action_was_kill = false;
        let page_height = self.screen_rows.saturating_sub(1).max(1); // Usable screen height, ensure at least 1
        self.row_offset = self.row_offset.saturating_sub(page_height);
        self.cursor_y = self.row_offset; // Set cursor to the top of the new view
        self.clamp_cursor_x();
    }
}

fn find_word_boundary_left(line: &str, current_x: usize) -> usize {
    let mut delete_start = current_x;

    if delete_start == 0 {
        return 0;
    }

    let mut chars_to_left = line[..delete_start].char_indices().rev();

    // Step 1: Skip any trailing non-word characters (e.g., punctuation, spaces after a word)
    // until we hit a word character.
    let mut found_word_char = false;
    for (idx, ch) in chars_to_left.by_ref() {
        if is_word_char(ch) {
            delete_start = idx; // This is the start of the first word character encountered
            found_word_char = true;
            break;
        }
        delete_start = idx; // Keep moving left through non-word chars
    }

    // Step 2: If we found a word character, now skip all word characters
    // to find the actual beginning of the word.
    if found_word_char {
        for (idx, ch) in chars_to_left.by_ref() {
            if !is_word_char(ch) {
                delete_start = idx + ch.len_utf8(); // This is the end of the non-word block (start of whitespace/punctuation)
                break;
            }
            delete_start = idx; // Keep moving left through word chars
        }
    }

    // Step 3: Now, `delete_start` is at the beginning of the word (or the beginning of the line
    // if no word was found). We need to also delete any preceding whitespace.
    // Iterate left from `delete_start` to find the first non-whitespace character.
    // We need a new iterator for this, starting from `delete_start`.
    let mut final_delete_start = delete_start;
    let whitespace_chars_to_left = line[..delete_start].char_indices().rev();
    for (idx, ch) in whitespace_chars_to_left {
        if ch.is_whitespace() {
            final_delete_start = idx;
        } else {
            break; // Found a non-whitespace character, stop.
        }
    }

    final_delete_start
}

fn is_word_char(ch: char) -> bool {
    ch.is_alphanumeric() || ch == '_'
}
