use unicode_width::UnicodeWidthChar;

use crate::document::Document;
use crate::editor::search::Search;
use crate::error::{DmacsError, Result};

pub mod input;
pub mod search;
pub mod ui;

const TAB_STOP: usize = 4;

pub struct Editor {
    pub should_quit: bool,
    pub document: Document,
    pub cursor_x: usize, // byte index
    pub cursor_y: usize,
    pub desired_cursor_x: usize, // column index
    pub status_message: String,
    pub row_offset: usize, // public for tests
    pub col_offset: usize, // public for tests
    pub screen_rows: usize,
    pub screen_cols: usize,
    undo_stack: Vec<(Document, usize, usize)>, // Stores (Document, cursor_x, cursor_y)
    pub kill_buffer: String,
    pub last_action_was_kill: bool,
    pub is_alt_pressed: bool,
    pub search: Search,
}

impl Editor {
    pub fn new(filename: Option<String>) -> Self {
        let document = match filename {
            Some(fname) => {
                if let Ok(doc) = Document::open(&fname) {
                    doc
                } else {
                    let mut doc = Document::new_empty();
                    doc.filename = Some(fname);
                    doc
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
            screen_rows: 0,
            screen_cols: 0,
            undo_stack: Vec::new(),
            kill_buffer: String::new(),
            last_action_was_kill: false,
            is_alt_pressed: false,
            search: Search::new(),
        }
    }

    pub fn update_screen_size(&mut self, screen_rows: usize, screen_cols: usize) {
        self.screen_rows = screen_rows;
        self.screen_cols = screen_cols;
    }

    pub fn save_state_for_undo(&mut self) {
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

    pub fn insert_char(&mut self, c: char) -> Result<()> {
        self.last_action_was_kill = false;
        self.save_state_for_undo();
        self.document.insert(self.cursor_x, self.cursor_y, c)?;
        self.cursor_x += c.len_utf8();
        self.desired_cursor_x =
            self.get_display_width(&self.document.lines[self.cursor_y], self.cursor_x);
        self.status_message = "".to_string();
        Ok(())
    }

    pub fn delete_char(&mut self) -> Result<()> {
        self.last_action_was_kill = false;
        // Backspace
        self.save_state_for_undo();
        if self.cursor_x > 0 {
            self.move_cursor_left();
            self.document.delete(self.cursor_x, self.cursor_y)?;
        } else if self.cursor_y > 0 {
            let prev_line_len = self.document.lines[self.cursor_y - 1].len();
            let current_line = self.document.lines.remove(self.cursor_y);
            self.document.lines[self.cursor_y - 1].push_str(&current_line);
            self.cursor_y -= 1;
            self.cursor_x = prev_line_len;
            self.desired_cursor_x =
                self.get_display_width(&self.document.lines[self.cursor_y], self.cursor_x);
        }
        Ok(())
    }

    pub fn delete_forward_char(&mut self) -> Result<()> {
        self.last_action_was_kill = false;
        // Ctrl-D
        self.save_state_for_undo();
        let y = self.cursor_y;
        let x = self.cursor_x;
        let line_len = self.document.lines.get(y).map_or(0, |l| l.len());
        if x < line_len {
            self.document.delete(x, y)?;
        } else if y < self.document.lines.len() - 1 {
            let next_line = self.document.lines.remove(y + 1);
            self.document.lines[y].push_str(&next_line);
        }
        Ok(())
    }

    pub fn insert_newline(&mut self) -> Result<()> {
        self.last_action_was_kill = false;
        self.save_state_for_undo();
        self.document.insert_newline(self.cursor_x, self.cursor_y)?;
        self.cursor_y += 1;
        self.cursor_x = 0;
        self.desired_cursor_x = 0;
        Ok(())
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
            self.kill_buffer.push('\x0a');
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
            self.kill_buffer.push('\x0a');
            self.kill_buffer.push_str(&next_line_content);
        }
        self.last_action_was_kill = true;
    }

    pub fn yank(&mut self) -> Result<()> {
        self.last_action_was_kill = false;
        self.save_state_for_undo();
        let text_to_yank = self.kill_buffer.clone();
        let mut current_x = self.cursor_x;
        let mut current_y = self.cursor_y;

        let lines_to_yank: Vec<&str> = text_to_yank.split('\x0a').collect();

        if lines_to_yank.is_empty() {
            return Ok(());
        }

        // Insert the first part of the yanked text into the current line
        self.document
            .insert_string(current_x, current_y, lines_to_yank[0])?;
        current_x += lines_to_yank[0].len();

        // Insert subsequent lines
        for line_to_yank in lines_to_yank.iter().skip(1) {
            self.document.insert_newline(current_x, current_y)?;
            current_y += 1;
            current_x = 0;
            self.document
                .insert_string(current_x, current_y, line_to_yank)?;
            current_x += line_to_yank.len();
        }

        self.cursor_x = current_x;
        self.cursor_y = current_y;
        self.desired_cursor_x =
            self.get_display_width(&self.document.lines[self.cursor_y], self.cursor_x);
        Ok(())
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

    pub fn move_cursor_word_left(&mut self) -> Result<()> {
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
            return Ok(());
        }

        let mut chars_iter = current_line[..new_cursor_x].char_indices().rev();

        // Step 1: Skip any non-word characters (including whitespace) to the left
        // until we hit a word character or the beginning of the line.
        let mut found_word_char = false;
        for (idx, ch) in chars_iter.by_ref() {
            if is_word_char(ch) {
                new_cursor_x = idx; // This is the start of a word
                found_word_char = true;
                break;
            }
            new_cursor_x = idx; // Keep moving left
        }

        // Step 2: If we found a word character, now skip all word characters
        // to find the actual beginning of the word.
        if found_word_char {
            for (idx, ch) in chars_iter {
                if !is_word_char(ch) {
                    new_cursor_x = idx + ch.len_utf8();
                    break;
                }
                new_cursor_x = idx; // Keep moving left
            }
        }

        self.cursor_x = new_cursor_x;
        self.desired_cursor_x = self.get_display_width(current_line, self.cursor_x);
        Ok(())
    }

    pub fn move_cursor_word_right(&mut self) -> Result<()> {
        self.last_action_was_kill = false;
        let current_line = &self.document.lines[self.cursor_y];
        let line_len = current_line.len();

        if self.cursor_x == line_len {
            if self.cursor_y < self.document.lines.len() - 1 {
                self.cursor_y += 1;
                self.cursor_x = 0;
                self.desired_cursor_x = 0;
            }
            return Ok(());
        }

        let mut current_byte_idx = self.cursor_x;

        // If we are currently on a non-word character, skip until we hit a word character.
        if current_byte_idx < line_len
            && !is_word_char(
                current_line[current_byte_idx..]
                    .chars()
                    .next()
                    .ok_or(DmacsError::Editor("Invalid character".to_string()))?,
            )
        {
            for ch in current_line[current_byte_idx..].chars() {
                if is_word_char(ch) {
                    break;
                }
                current_byte_idx += ch.len_utf8();
            }
        }

        // Now, current_byte_idx is either at the start of a word, or at the end of the line.
        // If it's at the start of a word, skip until we hit a non-word character or end of line.
        if current_byte_idx < line_len
            && is_word_char(
                current_line[current_byte_idx..]
                    .chars()
                    .next()
                    .ok_or(DmacsError::Editor("Invalid character".to_string()))?,
            )
        {
            for ch in current_line[current_byte_idx..].chars() {
                if !is_word_char(ch) {
                    break;
                }
                current_byte_idx += ch.len_utf8();
            }
        }

        self.cursor_x = current_byte_idx;
        self.desired_cursor_x = self.get_display_width(current_line, self.cursor_x);
        Ok(())
    }

    pub fn save_document(&mut self) -> Result<()> {
        self.last_action_was_kill = false;
        self.document.save()?;
        self.status_message = "File saved successfully.".to_string();
        Ok(())
    }

    pub fn quit(&mut self) -> Result<()> {
        self.last_action_was_kill = false;
        self.document.save()?;
        self.should_quit = true;
        Ok(())
    }

    pub fn clamp_cursor_x(&mut self) {
        if self.cursor_y >= self.document.lines.len() {
            self.cursor_x = 0;
            return;
        }
        let line_len = self.document.lines[self.cursor_y].len();
        if self.cursor_x > line_len {
            self.cursor_x = line_len;
        }
    }

    pub fn get_display_width(&self, line: &str, until_byte: usize) -> usize {
        let mut width = 0;
        let mut bytes = 0;
        for ch in line.chars() {
            if bytes >= until_byte {
                break;
            }
            if ch == '\x09' {
                width += TAB_STOP - (width % TAB_STOP);
            } else {
                width += ch.width().unwrap_or(0);
            }
            bytes += ch.len_utf8();
        }
        width
    }

    pub fn get_byte_pos_from_display_width(&self, display_x: usize) -> usize {
        let line = &self.document.lines[self.cursor_y];
        let mut current_display_x = 0;
        let mut byte_pos = 0;
        for ch in line.chars() {
            if current_display_x >= display_x {
                break;
            }
            if ch == '\x09' {
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
            self.cursor_y -= 1;
            self.document.swap_lines(self.cursor_y, self.cursor_y + 1);
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
        } else {
            self.status_message = "Cannot move line down further.".to_string();
        }
    }

    pub fn scroll_page_down(&mut self) {
        self.last_action_was_kill = false;
        let page_height = self.screen_rows.saturating_sub(2).max(1);
        self.row_offset = self.row_offset.saturating_add(page_height);
        self.row_offset = self
            .row_offset
            .min(self.document.lines.len().saturating_sub(1));
        self.cursor_y = self.row_offset;
        self.clamp_cursor_x();
    }

    pub fn scroll_page_up(&mut self) {
        self.last_action_was_kill = false;
        let page_height = self.screen_rows.saturating_sub(2).max(1);
        self.row_offset = self.row_offset.saturating_sub(page_height);
        self.cursor_y = self.row_offset;
        self.clamp_cursor_x();
    }

    pub fn go_to_start_of_file(&mut self) {
        self.last_action_was_kill = false;
        self.cursor_y = 0;
        self.cursor_x = 0;
        self.desired_cursor_x = 0;
        self.row_offset = 0;
        self.col_offset = 0;
    }

    pub fn go_to_end_of_file(&mut self) {
        self.last_action_was_kill = false;
        self.cursor_y = self.document.lines.len().saturating_sub(1);
        self.cursor_x = self.document.lines[self.cursor_y].len();
        self.desired_cursor_x =
            self.get_display_width(&self.document.lines[self.cursor_y], self.cursor_x);
        let screen_height = self.screen_rows.saturating_sub(1);
        if self.cursor_y >= self.row_offset + screen_height {
            self.row_offset = self.cursor_y.saturating_sub(screen_height) + 1;
        }
        self.clamp_cursor_x();
    }

    pub fn move_cursor_up(&mut self) {
        self.last_action_was_kill = false;
        if self.cursor_y > 0 {
            self.cursor_y -= 1;
            self.cursor_x = self.get_byte_pos_from_display_width(self.desired_cursor_x);
        } else {
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
            self.go_to_end_of_line();
        }
    }

    pub fn move_cursor_left(&mut self) {
        self.last_action_was_kill = false;
        let line = &self.document.lines[self.cursor_y];
        if self.cursor_x > 0 {
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

    pub fn set_alt_pressed(&mut self, is_alt_pressed: bool) {
        self.is_alt_pressed = is_alt_pressed;
    }

    pub fn move_to_next_delimiter(&mut self) {
        self.last_action_was_kill = false;
        let current_line_idx = self.cursor_y;
        let num_lines = self.document.lines.len();

        if num_lines == 0 {
            return; // Nothing to do in an empty document
        }

        let mut target_line_y: Option<usize> = None;

        // Scenario 1: Current line is a delimiter. Move to the line immediately after it.
        if current_line_idx < num_lines && self.document.lines[current_line_idx] == "---" {
            target_line_y = Some(current_line_idx + 1);
        } else {
            // Scenario 2: Current line is not a delimiter. Search for the next delimiter *after* the current position.
            for i in (current_line_idx + 1)..num_lines {
                if self.document.lines[i] == "---" {
                    target_line_y = Some(i + 1);
                    break;
                }
            }
            // If target_line_y is still None here, it means there are no delimiters
            // after the current position. According to the user's request, we should do nothing
            // in this case (no wrapping around to previous delimiters).
        }

        if let Some(new_cursor_y) = target_line_y {
            // If moving past the last line, and it was the last delimiter, do nothing.
            // This handles the case where the last delimiter is at the very end of the file.
            if new_cursor_y >= num_lines {
                return; // Do nothing if moving past the last delimiter and no more exist.
            }

            self.cursor_y = new_cursor_y;
            self.cursor_x = 0;
            self.desired_cursor_x = 0;
            self.row_offset = self.cursor_y; // Scroll to make cursor at top
        }
        // If target_line_y is None, do nothing, which is the desired behavior.
    }

    pub fn move_to_previous_delimiter(&mut self) {
        self.last_action_was_kill = false;
        let current_line_idx = self.cursor_y;
        let num_lines = self.document.lines.len();

        if num_lines == 0 {
            return; // Nothing to do in an empty document
        }

        let mut target_line_y: Option<usize> = None;

        // Iterate backwards from the line *before* the current cursor position
        // to find the closest "page position" above it.
        for i in (0..current_line_idx).rev() {
            // Check if 'i' itself is a page position (i.e., line 0 or line after a delimiter)
            if i == 0 {
                target_line_y = Some(0);
                break;
            }
            if self.document.lines[i - 1] == "---" {
                target_line_y = Some(i); // 'i' is the line after the delimiter at 'i-1'
                break;
            }
        }

        // If no page position found above (meaning we reached the beginning of the file
        // without finding a delimiter), move to page 0.
        if target_line_y.is_none() {
            target_line_y = Some(0);
        }

        if let Some(new_cursor_y) = target_line_y {
            self.cursor_y = new_cursor_y;
            self.cursor_x = 0;
            self.desired_cursor_x = 0;
            self.row_offset = self.cursor_y; // Scroll to make cursor at top
        }
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
            delete_start = idx;
            found_word_char = true;
            break;
        }
        delete_start = idx;
    }

    // Step 2: If we found a word character, now skip all word characters
    // to find the actual beginning of the word.
    if found_word_char {
        for (idx, ch) in chars_to_left {
            if !is_word_char(ch) {
                delete_start = idx + ch.len_utf8();
                break;
            }
            delete_start = idx;
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
            break;
        }
    }

    final_delete_start
}

fn is_word_char(ch: char) -> bool {
    ch.is_alphanumeric() || ch == '_' || ch.is_alphabetic()
}
