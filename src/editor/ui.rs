use crate::editor::Editor;
use pancurses::{A_BOLD, A_DIM, A_REVERSE, Window};
use std::cmp::min;
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

const TAB_STOP: usize = 4;
pub const STATUS_BAR_HEIGHT: usize = 2;

impl Editor {
    fn draw_fuzzy_search(&mut self, window: &Window) {
        let screen_rows = window.get_max_y() as usize;

        window.erase();

        // Draw the matches
        let matches = &self.fuzzy_search.matches;
        let selected_index = self.fuzzy_search.selected_index;

        let list_height = screen_rows.saturating_sub(1);

        if selected_index < self.fuzzy_search.scroll_offset {
            self.fuzzy_search.scroll_offset = selected_index;
        }
        if selected_index >= self.fuzzy_search.scroll_offset + list_height {
            self.fuzzy_search.scroll_offset = selected_index - list_height + 1;
        }

        let scroll_offset = self.fuzzy_search.scroll_offset;

        for (idx, (line, line_number)) in matches
            .iter()
            .skip(scroll_offset)
            .take(min(list_height, matches.len() - scroll_offset))
            .enumerate()
        {
            let i = scroll_offset + idx;
            let display_text = format!("{}: {}", line_number + 1, line);
            if i == selected_index {
                window.attron(A_REVERSE);
            }
            window.mvaddstr(idx as i32, 0, &display_text);
            if i == selected_index {
                window.attroff(A_REVERSE);
            }
        }

        // Draw the search prompt
        let prompt = format!("FUZZY SEARCH: {}", self.fuzzy_search.query);
        window.mvaddstr(screen_rows as i32 - 1, 0, &prompt);

        // Move cursor to the end of the prompt
        window.mv(screen_rows as i32 - 1, prompt.width() as i32);
        window.refresh();
    }

    pub fn is_separator_line(line: &str) -> bool {
        line == "---"
    }

    pub fn is_unchecked_checkbox(line: &str) -> bool {
        line.trim_start().starts_with("- [ ]")
    }

    pub fn is_checked_checkbox(line: &str) -> bool {
        line.trim_start().starts_with("- [x]")
    }

    pub fn draw(&mut self, window: &Window) {
        let screen_rows = window.get_max_y() as usize;
        let screen_cols = window.get_max_x() as usize;

        if self.mode == crate::editor::EditorMode::FuzzySearch {
            self.draw_fuzzy_search(window);
            return;
        }

        self.scroll();

        window.erase();

        let selection_range = self.selection.get_selection_range(self.cursor_pos());

        let document_start_row = STATUS_BAR_HEIGHT; // Default for normal mode
        let mut document_end_row = screen_rows;

        if self.mode == crate::editor::EditorMode::TaskSelection {
            // Calculate task UI height and position
            let task_ui_height = self.task_ui_height();
            let start_task_row = screen_rows.saturating_sub(task_ui_height);

            // Draw tasks
            for (i, (_original_idx, task_content)) in self.task.tasks.iter().enumerate() {
                let display_row = start_task_row + i - self.task.task_display_offset;
                if display_row >= start_task_row + task_ui_height {
                    // Ensure we don't draw beyond the task UI area
                    break;
                }
                if display_row < start_task_row {
                    // Ensure we don't draw above the task UI area
                    continue;
                }

                if Some(i) == self.task.selected_task_index {
                    window.attron(A_REVERSE);
                }
                window.mvaddstr(display_row as i32, 0, task_content);
                if Some(i) == self.task.selected_task_index {
                    window.attroff(A_REVERSE);
                }
            }

            // Draw a separator line above the task UI
            window.attron(A_DIM);
            for i in 0..screen_cols {
                window.mvaddch(start_task_row as i32 - 1, i as i32, pancurses::ACS_HLINE());
            }
            window.attroff(A_DIM);

            // Document starts from the top, below status bar
            // The document drawing area ends at the start of the task UI, excluding the separator line
            document_end_row = start_task_row.saturating_sub(1);
        }

        // Draw text
        for (index, line) in self.document.lines.iter().enumerate() {
            if index < self.scroll.row_offset {
                continue;
            }
            let row = index - self.scroll.row_offset;
            if row >= document_end_row.saturating_sub(document_start_row) {
                // Use document_start_row here
                break;
            }
            let row = row + document_start_row; // Adjust row based on document_start_row

            let is_comment = line.trim_start().starts_with('#');
            let is_unchecked = Self::is_unchecked_checkbox(line);
            let is_checked = Self::is_checked_checkbox(line);

            if is_comment || is_checked {
                window.attron(A_DIM);
            }
            if is_unchecked {
                window.attron(A_BOLD);
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

            let (mut start_byte, mut display_x_at_start) = if self.scroll.col_offset > 0 {
                self.scroll
                    .get_byte_pos_from_display_width(line, self.scroll.col_offset)
            } else {
                (0, 0)
            };

            if display_x_at_start < self.scroll.col_offset {
                if let Some(ch) = line[start_byte..].chars().next() {
                    let first_char_width = if ch == '\t' {
                        TAB_STOP - (display_x_at_start % TAB_STOP)
                    } else {
                        ch.width().unwrap_or(0)
                    };
                    display_x_at_start += first_char_width;
                    start_byte += ch.len_utf8();
                }
            }

            let mut byte_idx = start_byte;
            let line_len = line.len();
            let mut current_display_x = display_x_at_start;
            let mut screen_x = display_x_at_start.saturating_sub(self.scroll.col_offset);

            for ch in line[start_byte..].chars() {
                let char_width = if ch == '\t' {
                    TAB_STOP - (current_display_x % TAB_STOP)
                } else {
                    UnicodeWidthChar::width(ch).unwrap_or(0)
                };

                if screen_x + char_width > screen_cols {
                    break;
                }

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
                let display_string = if ch == '\t' {
                    " ".repeat(char_width)
                } else {
                    ch.to_string()
                };
                window.mvaddstr(row as i32, screen_x as i32, &display_string);

                if is_highlighted || is_selected {
                    window.attroff(A_REVERSE);
                }

                screen_x += char_width;
                current_display_x += char_width;
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
                    let eol_screen_x = current_display_x.saturating_sub(self.scroll.col_offset);
                    if eol_screen_x < screen_cols {
                        window.attron(A_REVERSE);
                        window.mvaddch(row as i32, eol_screen_x as i32, ' '); // Draw a reversed space
                        window.attroff(A_REVERSE);
                    }
                }
            }

            if is_comment || is_checked {
                window.attroff(A_DIM);
            }
            if is_unchecked {
                window.attroff(A_BOLD);
            }
        }

        // Draw filename (bold) and modified indicator
        let filename_display = self.document.filename.as_deref().unwrap_or("[No Name]");
        let modified_indicator = if self.document.is_dirty() { "*" } else { "" };
        let filename_and_modified = format!("{filename_display}{modified_indicator}");
        window.attron(A_BOLD);
        window.mvaddstr(0, 0, &filename_and_modified);
        window.attroff(A_BOLD);

        // Draw horizontal line below status bar content
        window.attron(A_DIM);
        for i in 0..screen_cols {
            window.mvaddch(
                STATUS_BAR_HEIGHT as i32 - 1,
                i as i32,
                pancurses::ACS_HLINE(),
            );
        }
        window.attroff(A_DIM);

        // Calculate the display width of the filename and modified indicator
        let mut current_col = 0;
        for ch in filename_and_modified.chars() {
            current_col += ch.width().unwrap_or(0);
        }

        // Draw line count
        let line_count_str = format!(" - {} lines", self.document.lines.len());
        window.mvaddstr(0, current_col as i32, &line_count_str);
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
            window.mvaddstr(0, message_start_col as i32, &self.status_message);
        }

        // Move cursor
        let display_cursor_x = self
            .scroll
            .get_display_width(&self.document.lines[self.cursor_y], self.cursor_x);
        window.mv(
            (self.cursor_y - self.scroll.row_offset + document_start_row) as i32,
            (display_cursor_x.saturating_sub(self.scroll.col_offset)) as i32,
        );
        window.refresh();
    }

    pub fn scroll(&mut self) {
        let mut visible_content_height = self.scroll.screen_rows.saturating_sub(STATUS_BAR_HEIGHT);

        if self.mode == crate::editor::EditorMode::TaskSelection {
            let task_ui_height = self.task_ui_height();
            visible_content_height = self
                .scroll
                .screen_rows
                .saturating_sub(STATUS_BAR_HEIGHT)
                .saturating_sub(task_ui_height);
        }

        // Vertical scroll
        if self.cursor_y < self.scroll.row_offset {
            self.scroll.row_offset = self.cursor_y;
        }
        if self.cursor_y >= self.scroll.row_offset + visible_content_height {
            self.scroll.row_offset = self.cursor_y - visible_content_height + 1;
        }

        // Horizontal scroll
        let scroll_margin = 10;
        let display_cursor_x = self
            .scroll
            .get_display_width(&self.document.lines[self.cursor_y], self.cursor_x);

        // Scroll right
        if display_cursor_x >= self.scroll.col_offset + self.scroll.screen_cols - scroll_margin {
            self.scroll.col_offset =
                display_cursor_x.saturating_sub(self.scroll.screen_cols - scroll_margin);
        }
        // Scroll left
        else if display_cursor_x < self.scroll.col_offset + scroll_margin {
            self.scroll.col_offset = display_cursor_x.saturating_sub(scroll_margin);
        }
    }
}
