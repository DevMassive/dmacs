use crate::editor::Editor;

pub struct Search {
    pub mode: bool,
    pub query: String,
    pub results: Vec<(usize, usize)>,
    pub current_match_index: Option<usize>,
}

impl Default for Search {
    fn default() -> Self {
        Self::new()
    }
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

impl Editor {
    pub fn enter_search_mode(&mut self) {
        self.search.mode = true;
        self.search.query.clear();
        self.search.results.clear();
        self.search.current_match_index = None;

        self.status_message = "Search: ".to_string();
    }

    pub fn handle_search_input(&mut self, key: pancurses::Input) {
        if let pancurses::Input::Character(c) = key {
            match c {
                '\x1b' | '\x0a' | '\x0d' | '\x07' => {
                    // Escape or Enter or Ctrl+G to exit search mode
                    self.search.mode = false;
                    self.search.query.clear();
                    self.search.results.clear();
                    self.search.current_match_index = None;
                    self.status_message.clear();
                }
                '\x13' => {
                    // Ctrl + S for next match
                    self.move_to_next_match();
                }
                '\x0e' => {
                    // Ctrl + N for next match (already handled, but keeping for consistency)
                    self.move_to_next_match();
                }
                '\x12' => {
                    // Ctrl + R for previous match
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

    pub fn search(&mut self) {
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

    pub fn move_to_match(&mut self) {
        if let Some(index) = self.search.current_match_index {
            if let Some(&(row, col)) = self.search.results.get(index) {
                self.cursor_y = row;
                self.cursor_x = col;
                self.desired_cursor_x = self.scroll.get_display_width_from_bytes(
                    &self.document.lines[self.cursor_y],
                    self.cursor_x,
                );
            }
        }
    }

    pub fn move_to_next_match(&mut self) {
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

    pub fn move_to_prev_match(&mut self) {
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
}
