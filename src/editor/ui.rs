use pancurses::{A_BOLD, A_DIM, A_REVERSE, Window};
use unicode_width::UnicodeWidthChar;

use crate::editor::Editor;

const TAB_STOP: usize = 4;

impl Editor {
    pub fn is_separator_line(line: &str) -> bool {
        line == "---"
    }

    pub fn draw(&mut self, window: &Window) {
        let screen_rows = window.get_max_y() as usize;
        let screen_cols = window.get_max_x() as usize;

        self.scroll();

        window.erase();

        let selection_range = self.selection.get_selection_range(self.cursor_pos());

        // Draw text
        for (index, line) in self.document.lines.iter().enumerate() {
            if index < self.row_offset {
                continue;
            }
            let row = index - self.row_offset;
            if row >= screen_rows.saturating_sub(2) {
                // Account for status bar and horizontal line
                break;
            }

            let is_comment = line.trim_start().starts_with('#');
            if is_comment {
                window.attron(A_DIM);
            }

            if Self::is_separator_line(line) {
                // Ensure A_DIM is off for this special line, in case it was turned on by is_comment
                if is_comment {
                    window.attroff(A_DIM);
                }

                let replacement_char_chtype = pancurses::ACS_HLINE();
                for i in 0..screen_cols {
                    if i < 3 {
                        // First three characters, no dim
                        window.mvaddch(row as i32, i as i32, replacement_char_chtype);
                    } else {
                        // Remaining characters, with dim
                        window.attron(A_DIM);
                        window.mvaddch(row as i32, i as i32, replacement_char_chtype);
                        window.attroff(A_DIM); // Turn off immediately after drawing
                    }
                }
                continue;
            }

            let mut display_x = 0;
            let mut byte_idx = 0;
            let line_len = line.len();
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

                let is_selected =
                    if let Some(((sel_start_x, sel_start_y), (sel_end_x, sel_end_y))) =
                        self.selection.get_selection_range(self.cursor_pos())
                    {
                        // Check if the current line is within the selection range
                        if index >= sel_start_y && index <= sel_end_y {
                            // If it's the start line, check from sel_start_x
                            if index == sel_start_y && index == sel_end_y {
                                // Single line selection
                                byte_idx >= sel_start_x && byte_idx < sel_end_x
                            } else if index == sel_start_y {
                                // Start of multi-line selection
                                byte_idx >= sel_start_x
                            } else if index == sel_end_y {
                                // End of multi-line selection
                                byte_idx < sel_end_x
                            } else {
                                // Full line in between multi-line selection
                                true
                            }
                        } else {
                            false
                        }
                    } else {
                        false
                    };

                if is_highlighted || is_selected {
                    window.attron(A_REVERSE);
                }

                // Draw character
                if display_x > self.col_offset {
                    let screen_x = char_start_display_x.saturating_sub(self.col_offset);
                    if screen_x < screen_cols {
                        if ch == '\x09' {
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

                if is_highlighted || is_selected {
                    window.attroff(A_REVERSE);
                }
                byte_idx += ch.len_utf8();
            }

            // Handle the virtual end-of-line character for selection highlighting
            if let Some(((_sel_start_x, sel_start_y), (sel_end_x, sel_end_y))) = selection_range {
                let highlight_eol_char = if index == sel_start_y && index == sel_end_y {
                    // Single line selection: highlight only if selection ends at the end of the line
                    sel_end_x == line_len
                } else if index == sel_start_y && index < sel_end_y {
                    // Start of multi-line selection: always highlight the newline
                    true
                } else if index > sel_start_y && index < sel_end_y {
                    // Full line in between multi-line selection: always highlight the newline
                    true
                } else if index == sel_end_y && index > sel_start_y {
                    // End of multi-line selection: highlight only if selection ends at the end of the line
                    sel_end_x == line_len
                } else {
                    false
                };

                if highlight_eol_char {
                    let eol_screen_x = display_x.saturating_sub(self.col_offset);
                    if eol_screen_x < screen_cols {
                        window.attron(A_REVERSE);
                        window.mvaddch(row as i32, eol_screen_x as i32, ' '); // Draw a reversed space
                        window.attroff(A_REVERSE);
                    }
                }
            }

            if is_comment {
                window.attroff(A_DIM);
            }
        }

        // Draw horizontal line above status bar
        window.attron(A_DIM);
        for i in 0..screen_cols {
            window.mvaddch(window.get_max_y() - 2, i as i32, pancurses::ACS_HLINE());
        }
        window.attroff(A_DIM);

        // Draw filename (bold) and modified indicator
        let filename_display = self.document.filename.as_deref().unwrap_or("[No Name]");
        let modified_indicator = if self.document.is_dirty() { "*" } else { "" };
        let filename_and_modified = format!("{filename_display}{modified_indicator}");
        window.attron(A_BOLD);
        window.mvaddstr(window.get_max_y() - 1, 0, &filename_and_modified);
        window.attroff(A_BOLD);

        // Calculate the display width of the filename and modified indicator
        let mut current_col = 0;
        for ch in filename_and_modified.chars() {
            current_col += ch.width().unwrap_or(0);
        }

        // Draw line count
        let line_count_str = format!(" - {} lines", self.document.lines.len());
        window.mvaddstr(window.get_max_y() - 1, current_col as i32, &line_count_str);
        for ch in line_count_str.chars() {
            current_col += ch.width().unwrap_or(0);
        }

        // Draw the status message on the right, if present
        if !self.status_message.is_empty() {
            let mut message_display_width = 0;
            for ch in self.status_message.chars() {
                message_display_width += ch.width().unwrap_or(0);
            }
            let message_start_col = screen_cols.saturating_sub(message_display_width);
            window.mvaddstr(
                window.get_max_y() - 1,
                message_start_col as i32,
                &self.status_message,
            );
        }

        // Move cursor
        let display_cursor_x =
            self.get_display_width(&self.document.lines[self.cursor_y], self.cursor_x);
        window.mv(
            (self.cursor_y - self.row_offset) as i32,
            (display_cursor_x - self.col_offset) as i32,
        );
        window.refresh();
    }

    pub fn scroll(&mut self) {
        // Vertical scroll
        if self.cursor_y < self.row_offset {
            self.row_offset = self.cursor_y;
        }
        if self.cursor_y >= self.row_offset + self.screen_rows {
            self.row_offset = self.cursor_y - self.screen_rows + 1;
        }

        // Horizontal scroll
        let display_cursor_x =
            self.get_display_width(&self.document.lines[self.cursor_y], self.cursor_x);
        if display_cursor_x < self.col_offset {
            self.col_offset = display_cursor_x;
        }
        if display_cursor_x >= self.col_offset + self.screen_cols {
            self.col_offset = display_cursor_x.saturating_sub(self.screen_cols) + 1;
        }
    }
}
