use pancurses::{A_BOLD, A_DIM, A_REVERSE, Window};
use unicode_width::UnicodeWidthChar;

use crate::editor::state::Editor;

const TAB_STOP: usize = 4;

impl Editor {
    pub fn draw(&mut self, window: &Window) {
        self.scroll(self.screen_cols, self.screen_rows - 2);

        window.erase();

        // Draw text
        for (index, line) in self.document.lines.iter().enumerate() {
            if index < self.row_offset {
                continue;
            }
            let row = index - self.row_offset;
            if row >= self.screen_rows.saturating_sub(2) {
                // Account for status bar and horizontal line
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
                let char_width = if ch == '\x09' {
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
                        if ch == '\x09' {
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
        let modified_indicator = if self.document.is_dirty() { "*" } else { "" };
        let status_message_str = if self.status_message.is_empty() {
            "".to_string()
        } else {
            format!(" | {}", self.status_message)
        };
        let status_bar = format!(
            "{} - {} lines{}{}",
            self.document.filename.as_deref().unwrap_or("[No Name]"),
            self.document.lines.len(),
            modified_indicator,
            status_message_str
        );
        // Draw horizontal line above status bar
        for i in 0..self.screen_cols {
            window.mvaddch(window.get_max_y() - 2, i as i32, pancurses::ACS_HLINE());
        }

        window.mvaddstr(window.get_max_y() - 1, 0, &status_bar);
        window.attron(A_BOLD);
        window.mvaddstr(
            window.get_max_y() - 1,
            0,
            self.document.filename.as_deref().unwrap_or("[No Name]"),
        );
        window.attroff(A_BOLD);

        // Move cursor
        let display_cursor_x =
            self.get_display_width(&self.document.lines[self.cursor_y], self.cursor_x);
        window.mv(
            (self.cursor_y - self.row_offset) as i32,
            (display_cursor_x - self.col_offset) as i32,
        );
        window.refresh();
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
}
