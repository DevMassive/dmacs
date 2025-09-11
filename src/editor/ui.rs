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

    pub fn get_prefix_info(&self, line: &str) -> (usize, usize) {
        // (byte_len, display_width)
        let mut byte_pos = 0;
        for ch in line.chars() {
            if !ch.is_whitespace() {
                break;
            }
            byte_pos += ch.len_utf8();
        }

        let after_indent = &line[byte_pos..];
        let mut comment_prefix_len = 0;
        let mut content_after_comment = after_indent;

        if after_indent.starts_with("# ") {
            comment_prefix_len = 2; // "# "
            content_after_comment = &after_indent[comment_prefix_len..];
        }

        let marker_bytes = if content_after_comment.starts_with("- [ ] ")
            || content_after_comment.starts_with("- [x] ")
        {
            6
        } else if content_after_comment.starts_with("- ") {
            2
        } else if content_after_comment.starts_with('/') {
            if let Some(end_pos) = content_after_comment.find(' ') {
                end_pos + 1
            } else {
                0
            }
        } else {
            0
        };

        let prefix_byte_len = byte_pos + comment_prefix_len + marker_bytes;
        let prefix_display_width = self
            .scroll
            .get_display_width_from_bytes(line, prefix_byte_len);
        (prefix_byte_len, prefix_display_width)
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
            let task_ui_height = self.task_ui_height();
            let start_task_row = screen_rows.saturating_sub(task_ui_height);

            for (i, (_original_idx, task_content)) in self.task.tasks.iter().enumerate() {
                let display_row = start_task_row + i - self.task.task_display_offset;
                if display_row >= start_task_row + task_ui_height {
                    break;
                }
                if display_row < start_task_row {
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

            window.attron(A_DIM);
            for i in 0..screen_cols {
                window.mvaddch(start_task_row as i32 - 1, i as i32, pancurses::ACS_HLINE());
            }
            window.attroff(A_DIM);

            document_end_row = start_task_row.saturating_sub(1);
        }

        // Draw text
        for (index, line) in self.document.lines.iter().enumerate() {
            if index < self.scroll.row_offset {
                continue;
            }
            let row = index - self.scroll.row_offset;
            if row >= document_end_row.saturating_sub(document_start_row) {
                break;
            }
            let row = row + document_start_row;

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
                if is_comment {
                    window.attroff(A_DIM);
                }

                let replacement_char_chtype = pancurses::ACS_HLINE();
                for i in 0..screen_cols {
                    if i < 3 {
                        window.mvaddch(row as i32, i as i32, replacement_char_chtype);
                    } else {
                        window.attron(A_DIM);
                        window.mvaddch(row as i32, i as i32, replacement_char_chtype);
                        window.attroff(A_DIM);
                    }
                }
                continue;
            }

            let (prefix_byte_len, _) = self.get_prefix_info(line);
            let content_col_offset = if index == self.cursor_y { self.scroll.col_offset } else { 0 };

            let mut current_display_x = 0;
            let mut screen_x = 0;

            let (mut content_start_byte_in_content, display_pos) = if content_col_offset > 0 {
                self.scroll
                    .get_byte_pos_from_display_width(&line[prefix_byte_len..], content_col_offset)
            } else {
                (0, 0)
            };

            let wide_char_scroll_adjust =
                content_col_offset > 0 && display_pos < content_col_offset;
            if wide_char_scroll_adjust {
                if let Some(ch) = &line[prefix_byte_len + content_start_byte_in_content..]
                    .chars()
                    .next()
                {
                    content_start_byte_in_content += ch.len_utf8();
                }
            }
            let content_start_byte = prefix_byte_len + content_start_byte_in_content;

            let mut ellipsis_drawn = false;

            for (byte_idx, ch) in line.char_indices() {
                if screen_x >= screen_cols {
                    break;
                }

                let is_in_prefix = byte_idx < prefix_byte_len;
                let mut should_draw = false;

                if is_in_prefix {
                    should_draw = true;
                } else {
                    if !ellipsis_drawn && content_col_offset > 0 {
                        let ellipsis = if wide_char_scroll_adjust { "… " } else { "…" };
                        let ellipsis_width = UnicodeWidthStr::width(ellipsis);
                        if screen_x + ellipsis_width <= screen_cols {
                            window.mvaddstr(row as i32, screen_x as i32, ellipsis);
                            screen_x += ellipsis_width;
                        }
                        ellipsis_drawn = true;
                    }

                    if byte_idx >= content_start_byte {
                        should_draw = true;
                    }
                }

                if should_draw {
                    let char_width = if ch == '\t' {
                        TAB_STOP - (current_display_x % TAB_STOP)
                    } else {
                        UnicodeWidthChar::width(ch).unwrap_or(0)
                    };
                    if screen_x + char_width > screen_cols {
                        break;
                    }

                    let is_highlighted = self.search.mode
                        && self.search.results.iter().any(|&(r, c)| {
                            r == index && byte_idx >= c && byte_idx < c + self.search.query.len()
                        });
                    let is_selected =
                        if let Some(((sel_start_x, sel_start_y), (sel_end_x, sel_end_y))) =
                            selection_range
                        {
                            if index >= sel_start_y && index <= sel_end_y {
                                if index == sel_start_y && index == sel_end_y {
                                    byte_idx >= sel_start_x && byte_idx < sel_end_x
                                } else if index == sel_start_y {
                                    byte_idx >= sel_start_x
                                } else if index == sel_end_y {
                                    byte_idx < sel_end_x
                                } else {
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
                }

                let char_width_for_display = if ch == '\t' {
                    TAB_STOP - (current_display_x % TAB_STOP)
                } else {
                    UnicodeWidthChar::width(ch).unwrap_or(0)
                };
                current_display_x += char_width_for_display;
            }

            if is_comment || is_checked {
                window.attroff(A_DIM);
            }
            if is_unchecked {
                window.attroff(A_BOLD);
            }
        }

        let filename_display = self.document.filename.as_deref().unwrap_or("[No Name]");
        let modified_indicator = if self.document.is_dirty() { "*" } else { "" };
        let filename_and_modified = format!("{}{}", filename_display, modified_indicator);
        window.attron(A_BOLD);
        window.mvaddstr(0, 0, &filename_and_modified);
        window.attroff(A_BOLD);

        window.attron(A_DIM);
        for i in 0..screen_cols {
            window.mvaddch(
                STATUS_BAR_HEIGHT as i32 - 1,
                i as i32,
                pancurses::ACS_HLINE(),
            );
        }
        window.attroff(A_DIM);

        let mut current_col = 0;
        for ch in filename_and_modified.chars() {
            current_col += ch.width().unwrap_or(0);
        }

        let line_count_str = format!(" - {} lines", self.document.lines.len());
        window.mvaddstr(0, current_col as i32, &line_count_str);
        for ch in line_count_str.chars() {
            current_col += ch.width().unwrap_or(0);
        }

        if !self.status_message.is_empty() {
            let mut message_display_width = 0;
            for ch in self.status_message.chars() {
                message_display_width += ch.width().unwrap_or(0);
            }
            let message_start_col = screen_cols.saturating_sub(message_display_width);
            window.mvaddstr(0, message_start_col as i32, &self.status_message);
        }

        let (prefix_byte_len, prefix_display_width) =
            self.get_prefix_info(&self.document.lines[self.cursor_y]);
        let display_cursor_x = self
            .scroll
            .get_display_width_from_bytes(&self.document.lines[self.cursor_y], self.cursor_x);

        let final_cursor_x = if self.cursor_x < prefix_byte_len {
            display_cursor_x
        } else {
            let content_display_cursor_x = display_cursor_x.saturating_sub(prefix_display_width);
            let content_line = &self.document.lines[self.cursor_y][prefix_byte_len..];
            let (_, display_pos) =
                self.scroll
                    .get_byte_pos_from_display_width(content_line, self.scroll.col_offset);
            let wide_char_scroll_adjust =
                self.scroll.col_offset > 0 && display_pos < self.scroll.col_offset;

            let effective_col_offset = if wide_char_scroll_adjust {
                self.scroll.col_offset + 1
            } else {
                self.scroll.col_offset
            };
            let cursor_pos_in_scrolled_content =
                content_display_cursor_x.saturating_sub(effective_col_offset);

            let ellipsis_width = if self.scroll.col_offset > 0 {
                if wide_char_scroll_adjust { 2 } else { 1 }
            } else {
                0
            };

            prefix_display_width + ellipsis_width + cursor_pos_in_scrolled_content
        };

        window.mv(
            (self.cursor_y - self.scroll.row_offset + document_start_row) as i32,
            final_cursor_x as i32,
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
        let screen_width = self.scroll.screen_cols;
        let current_line = &self.document.lines[self.cursor_y];

        let (prefix_byte_len, prefix_display_width) = self.get_prefix_info(current_line);
        let display_cursor_x = self
            .scroll
            .get_display_width_from_bytes(current_line, self.cursor_x);

        if self.cursor_x < prefix_byte_len {
            self.scroll.col_offset = 0;
        } else {
            let content_display_cursor_x = display_cursor_x.saturating_sub(prefix_display_width);
            let available_width = screen_width.saturating_sub(prefix_display_width);

            let will_be_scrolled = self.scroll.col_offset > 0
                || content_display_cursor_x
                    >= available_width.saturating_sub(scroll_margin.min(available_width));
            let ellipsis_width = if will_be_scrolled {
                UnicodeWidthStr::width("…")
            } else {
                0
            };

            let available_content_width = available_width.saturating_sub(ellipsis_width);

            let desired_cursor_screen_x = if available_content_width > scroll_margin {
                available_content_width.saturating_sub(scroll_margin)
            } else {
                available_content_width
            };

            if will_be_scrolled {
                self.scroll.col_offset =
                    content_display_cursor_x.saturating_sub(desired_cursor_screen_x);
            } else {
                self.scroll.col_offset = 0;
            }
        }
    }
}