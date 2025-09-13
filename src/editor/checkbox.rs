use crate::document::ActionDiff;
use crate::editor::{Editor, LastActionType};
use crate::error::Result;

impl Editor {
    pub fn toggle_checkbox(&mut self) -> Result<()> {
        if self.selection.is_selection_active() {
            if let Some(((_start_x, start_y), (_end_x, end_y))) =
                self.selection.get_selection_range(self.cursor_pos())
            {
                let (original_cursor_x, original_cursor_y) = self.cursor_pos();

                let mut states_to_process = Vec::new();
                for y in start_y..=end_y {
                    if y >= self.document.lines.len() {
                        continue;
                    }

                    let line = &self.document.lines[y];
                    let is_last_line_and_excluded =
                        y == end_y && original_cursor_y == end_y && original_cursor_x == 0;

                    if line.is_empty() || is_last_line_and_excluded {
                        continue; // Skip empty lines and the last line if cursor is at x=0
                    }
                    states_to_process.push(get_line_state(line));
                }

                if states_to_process.is_empty() {
                    return Ok(()); // No lines to process
                }

                let all_same_state = states_to_process.windows(2).all(|w| w[0] == w[1]);
                let target_state = if all_same_state {
                    states_to_process
                        .first()
                        .map_or(LineState::ListItem, |s| s.next())
                } else {
                    LineState::ListItem
                };

                let mut new_lines = Vec::new();
                let mut old_lines = Vec::new();
                for y in start_y..=end_y {
                    if y >= self.document.lines.len() {
                        continue;
                    }
                    let original_line = &self.document.lines[y];
                    old_lines.push(original_line.clone());

                    let is_last_line_and_excluded =
                        y == end_y && original_cursor_y == end_y && original_cursor_x == 0;
                    if original_line.is_empty() || is_last_line_and_excluded {
                        new_lines.push(original_line.clone());
                    } else {
                        new_lines.push(transform_line(original_line, target_state));
                    }
                }

                let original_end_line_len = self.document.lines.get(end_y).map_or(0, |l| l.len());

                // Use two-step commit (delete then insert) for undo safety
                self.commit(
                    LastActionType::ToggleCheckbox,
                    &ActionDiff {
                        cursor_start_x: original_cursor_x,
                        cursor_start_y: original_cursor_y,
                        cursor_end_x: original_cursor_x, // Not important for this part
                        cursor_end_y: start_y,           // Cursor moves to start of selection
                        start_x: 0,
                        start_y,
                        end_x: original_end_line_len,
                        end_y,
                        new: vec![],
                        old: old_lines,
                    },
                );

                self.commit(
                    LastActionType::Ammend,
                    &ActionDiff {
                        cursor_start_x: self.cursor_x,   // Current x after delete
                        cursor_start_y: self.cursor_y,   // Current y after delete (is start_y)
                        cursor_end_x: original_cursor_x, // Restore original cursor x
                        cursor_end_y: original_cursor_y, // Restore original cursor y
                        start_x: 0,
                        start_y,
                        end_x: new_lines.last().map_or(0, |l| l.len()),
                        end_y: start_y + new_lines.len() - 1,
                        new: new_lines,
                        old: vec![],
                    },
                );

                self.status_message = format!("Toggled selection to {target_state:?}.");
            }
        } else {
            // Original single-line logic
            let y = self.cursor_y;
            if y >= self.document.lines.len() {
                return Ok(());
            }

            let original_line = self.document.lines[y].clone();
            let state = get_line_state(&original_line);
            let next_state = state.next();
            let new_line = transform_line(&original_line, next_state);

            let cursor_x_change: isize = match (state, next_state) {
                (LineState::Plain, LineState::ListItem) => 2, // "- "
                (LineState::ListItem, LineState::Unchecked) => 4, // "[ ] "
                (LineState::Unchecked, LineState::Checked) => 0,
                (LineState::Checked, LineState::Plain) => -6, // "- [x] "
                _ => 0,
            };

            let leading_whitespace_len = original_line.len() - original_line.trim_start().len();
            let mut new_cursor_x = self.cursor_x;
            if cursor_x_change > 0 {
                if self.cursor_x >= leading_whitespace_len {
                    new_cursor_x += cursor_x_change as usize;
                } else {
                    new_cursor_x = leading_whitespace_len + cursor_x_change as usize;
                }
            } else if cursor_x_change < 0 {
                new_cursor_x = new_cursor_x.saturating_sub(cursor_x_change.unsigned_abs());
            }

            if new_cursor_x > new_line.len() {
                new_cursor_x = new_line.len();
            }

            // Revert to two-commit approach for undo safety
            self.commit(
                LastActionType::ToggleCheckbox,
                &ActionDiff {
                    cursor_start_x: self.cursor_x,
                    cursor_start_y: self.cursor_y,
                    cursor_end_x: 0, // Cursor position after deletion is irrelevant
                    cursor_end_y: self.cursor_y,
                    start_x: 0,
                    start_y: self.cursor_y,
                    end_x: original_line.len(),
                    end_y: self.cursor_y,
                    new: vec![],
                    old: vec![original_line],
                },
            );
            self.commit(
                LastActionType::Ammend,
                &ActionDiff {
                    cursor_start_x: 0, // Cursor position before insertion is irrelevant
                    cursor_start_y: self.cursor_y,
                    cursor_end_x: new_cursor_x,
                    cursor_end_y: self.cursor_y,
                    start_x: 0,
                    start_y: self.cursor_y,
                    end_x: new_line.len(),
                    end_y: self.cursor_y,
                    new: vec![new_line],
                    old: vec![],
                },
            );

            self.status_message = format!("Toggled to {next_state:?}.");
        }

        Ok(())
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum LineState {
    Checked,
    Unchecked,
    ListItem,
    Plain,
}

impl LineState {
    fn next(&self) -> Self {
        match self {
            LineState::Plain => LineState::ListItem,
            LineState::ListItem => LineState::Unchecked,
            LineState::Unchecked => LineState::Checked,
            LineState::Checked => LineState::Plain,
        }
    }
}

fn get_line_state(line: &str) -> LineState {
    let trimmed = line.trim_start();
    if trimmed.starts_with("- [x] ") {
        LineState::Checked
    } else if trimmed.starts_with("- [ ] ") {
        LineState::Unchecked
    } else if trimmed.starts_with("- ") {
        LineState::ListItem
    } else {
        LineState::Plain
    }
}

fn transform_line(original_line: &str, target_state: LineState) -> String {
    let leading_whitespace_len = original_line.len() - original_line.trim_start().len();
    let leading_whitespace = &original_line[..leading_whitespace_len];
    let trimmed_line = original_line.trim_start();

    let content = match get_line_state(original_line) {
        LineState::Checked => trimmed_line.strip_prefix("- [x] ").unwrap_or(trimmed_line),
        LineState::Unchecked => trimmed_line.strip_prefix("- [ ] ").unwrap_or(trimmed_line),
        LineState::ListItem => trimmed_line.strip_prefix("- ").unwrap_or(trimmed_line),
        LineState::Plain => trimmed_line,
    };

    match target_state {
        LineState::Checked => format!("{leading_whitespace}- [x] {content}"),
        LineState::Unchecked => format!("{leading_whitespace}- [ ] {content}"),
        LineState::ListItem => format!("{leading_whitespace}- {content}"),
        LineState::Plain => format!("{leading_whitespace}{content}"),
    }
}
