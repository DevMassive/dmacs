use crate::document::Document;
use crate::editor::ui::STATUS_BAR_HEIGHT;
use unicode_width::UnicodeWidthChar;

const TAB_STOP: usize = 4;

pub struct Scroll {
    pub row_offset: usize,
    pub col_offset: usize,
    pub screen_rows: usize,
    pub screen_cols: usize,
}

impl Default for Scroll {
    fn default() -> Self {
        Self::new()
    }
}

impl Scroll {
    pub fn new() -> Self {
        Self {
            row_offset: 0,
            col_offset: 0,
            screen_rows: 0,
            screen_cols: 0,
        }
    }

    pub fn new_with_offset(row_offset: usize, col_offset: usize) -> Self {
        Self {
            row_offset,
            col_offset,
            screen_rows: 0, // These will be updated later by update_screen_size
            screen_cols: 0, // These will be updated later by update_screen_size
        }
    }

    pub fn update_screen_size(&mut self, screen_rows: usize, screen_cols: usize) {
        self.screen_rows = screen_rows;
        self.screen_cols = screen_cols;
    }

    // Helper functions that were in Editor, now in Scroll
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

    pub fn get_byte_pos_from_display_width(&self, line: &str, display_x: usize) -> (usize, usize) {
        let mut current_display_x = 0;
        let mut byte_pos = 0;
        for ch in line.chars() {
            if current_display_x >= display_x {
                return (byte_pos, current_display_x);
            }
            let next_display_x = if ch == '\t' {
                current_display_x + (TAB_STOP - (current_display_x % TAB_STOP))
            } else {
                current_display_x + ch.width().unwrap_or(0)
            };

            if next_display_x > display_x {
                return (byte_pos, current_display_x);
            }
            current_display_x = next_display_x;
            byte_pos += ch.len_utf8();
        }
        (byte_pos, current_display_x)
    }

    // Helper for clamping cursor_x, now part of Scroll
    pub fn clamp_cursor_x(&self, cursor_x: &mut usize, cursor_y: &usize, document: &Document) {
        if *cursor_y >= document.lines.len() {
            *cursor_x = 0;
            return;
        }
        let line_len = document.lines[*cursor_y].len();
        if *cursor_x > line_len {
            *cursor_x = line_len;
        }
    }

    // Methods that modify Editor's cursor and document
    pub fn scroll_page_down(
        &mut self,
        cursor_y: &mut usize,
        cursor_x: &mut usize,
        document: &Document,
        last_action_was_kill: &mut bool,
    ) {
        *last_action_was_kill = false;
        let page_height = self.screen_rows.saturating_sub(STATUS_BAR_HEIGHT).max(1);
        self.row_offset = self.row_offset.saturating_add(page_height);
        self.row_offset = self.row_offset.min(document.lines.len().saturating_sub(1));
        *cursor_y = self.row_offset;
        self.clamp_cursor_x(cursor_x, cursor_y, document);
    }

    pub fn scroll_page_up(
        &mut self,
        cursor_y: &mut usize,
        cursor_x: &mut usize,
        document: &Document,
        last_action_was_kill: &mut bool,
    ) {
        *last_action_was_kill = false;
        let page_height = self.screen_rows.saturating_sub(STATUS_BAR_HEIGHT).max(1);
        self.row_offset = self.row_offset.saturating_sub(page_height);
        *cursor_y = self.row_offset;
        self.clamp_cursor_x(cursor_x, cursor_y, document);
    }

    pub fn go_to_start_of_file(
        &mut self,
        cursor_y: &mut usize,
        cursor_x: &mut usize,
        desired_cursor_x: &mut usize,
        last_action_was_kill: &mut bool,
    ) {
        *last_action_was_kill = false;
        *cursor_y = 0;
        *cursor_x = 0;
        *desired_cursor_x = 0;
        self.row_offset = 0;
        self.col_offset = 0;
    }

    pub fn go_to_end_of_file(
        &mut self,
        cursor_y: &mut usize,
        cursor_x: &mut usize,
        desired_cursor_x: &mut usize,
        document: &Document,
        last_action_was_kill: &mut bool,
    ) {
        *last_action_was_kill = false;
        *cursor_y = document.lines.len().saturating_sub(1);
        *cursor_x =
            self.get_display_width(&document.lines[*cursor_y], document.lines[*cursor_y].len());
        *desired_cursor_x = *cursor_x;
        let screen_height = self.screen_rows.saturating_sub(1);
        if *cursor_y >= self.row_offset + screen_height {
            self.row_offset = cursor_y.saturating_sub(screen_height) + 1;
        }
        self.clamp_cursor_x(cursor_x, cursor_y, document);
    }

    pub fn move_cursor_up(
        &mut self,
        cursor_y: &mut usize,
        cursor_x: &mut usize,
        desired_cursor_x: &mut usize,
        document: &Document,
        last_action_was_kill: &mut bool,
    ) {
        *last_action_was_kill = false;
        if *cursor_y > 0 {
            *cursor_y -= 1;
            *cursor_x = self
                .get_byte_pos_from_display_width(&document.lines[*cursor_y], *desired_cursor_x)
                .0;
        } else {
            *cursor_x = 0;
            *desired_cursor_x = 0;
        }
    }

    pub fn move_cursor_down(
        &mut self,
        cursor_y: &mut usize,
        cursor_x: &mut usize,
        desired_cursor_x: &mut usize,
        document: &Document,
        last_action_was_kill: &mut bool,
    ) {
        *last_action_was_kill = false;
        if *cursor_y < document.lines.len().saturating_sub(1) {
            *cursor_y += 1;
            *cursor_x = self
                .get_byte_pos_from_display_width(&document.lines[*cursor_y], *desired_cursor_x)
                .0;
        } else {
            *cursor_x = document.lines[*cursor_y].len();
            *desired_cursor_x = self.get_display_width(&document.lines[*cursor_y], *cursor_x);
        }
    }

    pub fn move_cursor_left(
        &mut self,
        cursor_y: &mut usize,
        cursor_x: &mut usize,
        desired_cursor_x: &mut usize,
        document: &Document,
        last_action_was_kill: &mut bool,
    ) {
        *last_action_was_kill = false;
        let line = &document.lines[*cursor_y];
        if *cursor_x > 0 {
            let mut new_pos = *cursor_x - 1;
            while !line.is_char_boundary(new_pos) {
                new_pos -= 1;
            }
            *cursor_x = new_pos;
            *desired_cursor_x = self.get_display_width(line, *cursor_x);
        } else if *cursor_y > 0 {
            *cursor_y -= 1;
            *cursor_x = document.lines[*cursor_y].len();
            *desired_cursor_x = self.get_display_width(&document.lines[*cursor_y], *cursor_x);
        }
    }

    pub fn move_cursor_right(
        &mut self,
        cursor_y: &mut usize,
        cursor_x: &mut usize,
        desired_cursor_x: &mut usize,
        document: &Document,
        last_action_was_kill: &mut bool,
    ) {
        *last_action_was_kill = false;
        let line = &document.lines[*cursor_y];
        if *cursor_x < line.len() {
            let mut new_pos = *cursor_x + 1;
            while !line.is_char_boundary(new_pos) {
                new_pos += 1;
            }
            *cursor_x = new_pos;
            *desired_cursor_x = self.get_display_width(line, *cursor_x);
        } else if *cursor_y < document.lines.len().saturating_sub(1) {
            *cursor_y += 1;
            *cursor_x = 0;
            *desired_cursor_x = 0;
        }
    }
}
