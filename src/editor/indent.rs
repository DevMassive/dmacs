use crate::document::ActionDiff;
use crate::editor::{Editor, LastActionType};
use crate::error::Result;

impl Editor {
    fn handle_selection_indent_outdent<F>(&mut self, operation: F) -> Result<()>
    where
        F: Fn(&str) -> String,
    {
        self.last_action_was_kill = false;

        // Store original selection points
        let original_cursor_pos = self.cursor_pos();
        let original_marker_pos = self.selection.marker_pos.unwrap();

        let (start, end) = self.selection.get_selection_range(original_cursor_pos).unwrap();
        let start_y = start.1;
        let mut end_y = end.1;

        if end.0 == 0 && end_y > start_y {
            end_y -= 1;
        }

        if end_y < start_y {
            return Ok(());
        }

        let mut original_lines = Vec::new();
        let mut new_lines = Vec::new();
        let mut line_deltas = std::collections::HashMap::new();

        for y in start_y..=end_y {
            if y < self.document.lines.len() {
                let line = &self.document.lines[y];
                original_lines.push(line.clone());
                if line.is_empty() {
                    new_lines.push(line.clone());
                    line_deltas.insert(y, 0);
                } else {
                    let new_line = operation(line);
                    line_deltas.insert(y, new_line.len() as isize - line.len() as isize);
                    new_lines.push(new_line);
                }
            }
        }

        if original_lines.is_empty() {
            return Ok(());
        }

        let original_end_x = self.document.lines.get(end_y).map_or(0, |l| l.len());

        self.save_state_for_undo(LastActionType::Other);

        // 1. Delete the original lines
        let delete_diff = ActionDiff {
            cursor_start_x: self.cursor_x,
            cursor_start_y: self.cursor_y,
            cursor_end_x: 0,
            cursor_end_y: start_y,
            start_x: 0,
            start_y,
            end_x: original_end_x,
            end_y,
            new: vec![],
            old: original_lines,
        };
        if let Some(last_transaction) = self.undo_stack.last_mut() {
            last_transaction.push(delete_diff.clone());
        }
        let (new_x, new_y) = self.document.apply_action_diff(&delete_diff, false).unwrap();
        self.cursor_x = new_x;
        self.cursor_y = new_y;

        // 2. Insert the new lines
        let new_last_line_len = new_lines.last().map_or(0, |l| l.len());
        let insert_diff = ActionDiff {
            cursor_start_x: self.cursor_x,
            cursor_start_y: self.cursor_y,
            cursor_end_x: self.cursor_x, // Keep cursor at start of modified region
            cursor_end_y: self.cursor_y,
            start_x: self.cursor_x,
            start_y: self.cursor_y,
            end_x: new_last_line_len,
            end_y: self.cursor_y + new_lines.len() - 1,
            new: new_lines,
            old: vec![],
        };
        if let Some(last_transaction) = self.undo_stack.last_mut() {
            last_transaction.push(insert_diff.clone());
        }
        self.document.apply_action_diff(&insert_diff, false).unwrap();

        // 3. Calculate and set new selection points
        let mut new_cursor_pos = original_cursor_pos;
        if let Some(delta) = line_deltas.get(&new_cursor_pos.1) {
            new_cursor_pos.0 = (new_cursor_pos.0 as isize + delta).max(0) as usize;
        }

        let mut new_marker_pos = original_marker_pos;
        if let Some(delta) = line_deltas.get(&new_marker_pos.1) {
            new_marker_pos.0 = (new_marker_pos.0 as isize + delta).max(0) as usize;
        }

        // 4. Update state
        self.cursor_x = new_cursor_pos.0;
        self.cursor_y = new_cursor_pos.1;
        self.selection.marker_pos = Some(new_marker_pos);

        self.desired_cursor_x = self
            .scroll
            .get_display_width(&self.document.lines[self.cursor_y], self.cursor_x);

        Ok(())
    }

    pub fn indent_line(&mut self) -> Result<()> {
        if self.selection.is_selection_active() {
            self.handle_selection_indent_outdent(|line| format!("  {}", line))
        } else {
            let y = self.cursor_y;
            if y >= self.document.lines.len() {
                return Ok(());
            }
            self.commit(
                LastActionType::Other,
                &ActionDiff {
                    cursor_start_x: self.cursor_x,
                    cursor_start_y: self.cursor_y,
                    cursor_end_x: self.cursor_x + 2,
                    cursor_end_y: self.cursor_y,
                    start_x: 0,
                    start_y: y,
                    end_x: 0,
                    end_y: y,
                    new: vec!["  ".to_string()],
                    old: vec![],
                },
            );
            self.last_action_was_kill = false;
            Ok(())
        }
    }

    pub fn outdent_line(&mut self) -> Result<()> {
        if self.selection.is_selection_active() {
            self.handle_selection_indent_outdent(|line| {
                if line.starts_with("  ") {
                    line[2..].to_string()
                } else if line.starts_with(' ') {
                    line[1..].to_string()
                } else {
                    line.to_string()
                }
            })
        } else {
            let y = self.cursor_y;
            if y >= self.document.lines.len() {
                return Ok(());
            }
            let line = &self.document.lines[y];
            if line.starts_with("  ") {
                self.commit(
                    LastActionType::Other,
                    &ActionDiff {
                        cursor_start_x: self.cursor_x,
                        cursor_start_y: self.cursor_y,
                        cursor_end_x: self.cursor_x.saturating_sub(2),
                        cursor_end_y: self.cursor_y,
                        start_x: 0,
                        start_y: y,
                        end_x: 2,
                        end_y: y,
                        new: vec![],
                        old: vec!["  ".to_string()],
                    },
                );
            } else if line.starts_with(' ') {
                self.commit(
                    LastActionType::Other,
                    &ActionDiff {
                        cursor_start_x: self.cursor_x,
                        cursor_start_y: self.cursor_y,
                        cursor_end_x: self.cursor_x.saturating_sub(1),
                        cursor_end_y: self.cursor_y,
                        start_x: 0,
                        start_y: y,
                        end_x: 1,
                        end_y: y,
                        new: vec![],
                        old: vec![" ".to_string()],
                    },
                );
            }
            self.last_action_was_kill = false;
            Ok(())
        }
    }
}
