use crate::document::ActionDiff;
use crate::editor::{Editor, LastActionType};
use crate::error::Result;

const COMMENT_PREFIX: &str = "# ";

impl Editor {
    pub fn toggle_comment(&mut self) -> Result<()> {
        self.last_action_was_kill = false;

        if self.selection.is_selection_active() {
            if let Some(((_start_x, start_y), (_end_x, end_y))) =
                self.selection.get_selection_range(self.cursor_pos())
            {
                let (original_cursor_x, original_cursor_y) = self.cursor_pos();

                let mut lines_to_process = Vec::new();
                for y in start_y..=end_y {
                    if y >= self.document.lines.len() {
                        continue;
                    }
                    let line = &self.document.lines[y];
                    let is_last_line_and_excluded =
                        y == end_y && original_cursor_y == end_y && original_cursor_x == 0;

                    if line.is_empty() || is_last_line_and_excluded {
                        continue;
                    }
                    lines_to_process.push(line);
                }

                if lines_to_process.is_empty() {
                    return Ok(());
                }

                let all_commented = lines_to_process
                    .iter()
                    .all(|line| line.trim_start().starts_with(COMMENT_PREFIX));

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
                    } else if all_commented {
                        new_lines.push(uncomment_line(original_line));
                    } else {
                        new_lines.push(comment_line(original_line));
                    }
                }

                let original_end_line_len = self.document.lines.get(end_y).map_or(0, |l| l.len());

                self.commit(
                    LastActionType::ToggleComment,
                    &ActionDiff {
                        cursor_start_x: original_cursor_x,
                        cursor_start_y: original_cursor_y,
                        cursor_end_x: original_cursor_x,
                        cursor_end_y: start_y,
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
                        cursor_start_x: self.cursor_x,
                        cursor_start_y: self.cursor_y,
                        cursor_end_x: original_cursor_x,
                        cursor_end_y: original_cursor_y,
                        start_x: 0,
                        start_y,
                        end_x: new_lines.last().map_or(0, |l| l.len()),
                        end_y: start_y + new_lines.len() - 1,
                        new: new_lines,
                        old: vec![],
                    },
                );

                self.status_message = "Toggled comment on selection.".to_string();
            }
        } else {
            // Single line
            let y = self.cursor_y;
            if y >= self.document.lines.len() {
                return Ok(());
            }

            let original_line = self.document.lines[y].clone();
            if original_line.is_empty() {
                return Ok(());
            }

            let is_commented = original_line.trim_start().starts_with(COMMENT_PREFIX);
            let new_line = if is_commented {
                uncomment_line(&original_line)
            } else {
                comment_line(&original_line)
            };

            let cursor_x_change = new_line.len() as isize - original_line.len() as isize;
            let mut new_cursor_x = self.cursor_x as isize + cursor_x_change;
            if self.cursor_x < original_line.len() - original_line.trim_start().len() {
                new_cursor_x = self.cursor_x as isize;
            }
            new_cursor_x = new_cursor_x.max(0);

            self.commit(
                LastActionType::ToggleComment,
                &ActionDiff {
                    cursor_start_x: self.cursor_x,
                    cursor_start_y: self.cursor_y,
                    cursor_end_x: 0,
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
                    cursor_start_x: 0,
                    cursor_start_y: self.cursor_y,
                    cursor_end_x: new_cursor_x as usize,
                    cursor_end_y: self.cursor_y,
                    start_x: 0,
                    start_y: self.cursor_y,
                    end_x: new_line.len(),
                    end_y: self.cursor_y,
                    new: vec![new_line.clone()],
                    old: vec![],
                },
            );

            self.status_message = if is_commented {
                "Uncommented line.".to_string()
            } else {
                "Commented line.".to_string()
            };
        }

        Ok(())
    }
}

fn comment_line(line: &str) -> String {
    let leading_whitespace_len = line.len() - line.trim_start().len();
    let leading_whitespace = &line[..leading_whitespace_len];
    format!(
        "{}{}{}",
        leading_whitespace,
        COMMENT_PREFIX,
        &line[leading_whitespace_len..]
    )
}

fn uncomment_line(line: &str) -> String {
    let leading_whitespace_len = line.len() - line.trim_start().len();
    let leading_whitespace = &line[..leading_whitespace_len];
    let trimmed_line = line.trim_start();
    if let Some(stripped) = trimmed_line.strip_prefix(COMMENT_PREFIX) {
        format!("{leading_whitespace}{stripped}")
    } else {
        line.to_string()
    }
}
