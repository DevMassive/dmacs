use crate::document::ActionDiff;
use crate::editor::{Editor, LastActionType};
use crate::error::Result;

impl Editor {
    fn handle_selection_indent_outdent<F>(&mut self, operation: F) -> Result<()>
    where
        F: Fn(&str) -> String,
    {
        let original_cursor_pos = self.cursor_pos();
        let original_marker_pos = self.selection.marker_pos.unwrap_or(original_cursor_pos);

        let (start, end) = self
            .selection
            .get_selection_range(original_cursor_pos)
            .unwrap();
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

        let mut new_cursor_pos = original_cursor_pos;
        if let Some(delta) = line_deltas.get(&new_cursor_pos.1) {
            new_cursor_pos.0 = (new_cursor_pos.0 as isize + delta).max(0) as usize;
        }

        let mut new_marker_pos = original_marker_pos;
        if let Some(delta) = line_deltas.get(&new_marker_pos.1) {
            new_marker_pos.0 = (new_marker_pos.0 as isize + delta).max(0) as usize;
        }

        // Use two commits to ensure undo works as a single transaction
        // 1. Delete the original lines
        self.commit(
            LastActionType::Other,
            &ActionDiff {
                cursor_start_x: self.cursor_x,
                cursor_start_y: self.cursor_y,
                cursor_end_x: 0,
                cursor_end_y: start_y,
                start_x: 0,
                start_y,
                end_x: original_lines.last().map_or(0, |l| l.len()),
                end_y,
                new: vec![],
                old: original_lines,
            },
        );

        // 2. Insert the new lines, ammend to the previous action
        self.commit(
            LastActionType::Ammend,
            &ActionDiff {
                cursor_start_x: self.cursor_x,
                cursor_start_y: self.cursor_y,
                cursor_end_x: new_cursor_pos.0,
                cursor_end_y: new_cursor_pos.1,
                start_x: 0,
                start_y,
                end_x: new_lines.last().map_or(0, |l| l.len()),
                end_y: start_y + new_lines.len() - 1,
                new: new_lines,
                old: vec![],
            },
        );

        self.selection.marker_pos = Some(new_marker_pos);

        self.desired_cursor_x = self
            .scroll
            .get_display_width_from_bytes(&self.document.lines[self.cursor_y], self.cursor_x);

        Ok(())
    }

    pub fn indent_line(&mut self) -> Result<()> {
        if self.selection.is_selection_active() {
            self.handle_selection_indent_outdent(|line| format!("  {line}"))
        } else {
            let y = self.cursor_y;
            if y >= self.document.lines.len() {
                return Ok(());
            }
            let original_line = self.document.lines[y].clone();
            let new_line = format!("  {original_line}");
            let new_cursor_x = self.cursor_x + 2;

            self.commit(
                LastActionType::Indent,
                &ActionDiff {
                    cursor_start_x: self.cursor_x,
                    cursor_start_y: self.cursor_y,
                    cursor_end_x: 0,
                    cursor_end_y: y,
                    start_x: 0,
                    start_y: y,
                    end_x: original_line.len(),
                    end_y: y,
                    new: vec![],
                    old: vec![original_line],
                },
            );
            self.commit(
                LastActionType::Ammend,
                &ActionDiff {
                    cursor_start_x: 0,
                    cursor_start_y: y,
                    cursor_end_x: new_cursor_x,
                    cursor_end_y: y,
                    start_x: 0,
                    start_y: y,
                    end_x: new_line.len(),
                    end_y: y,
                    new: vec![new_line],
                    old: vec![],
                },
            );
            Ok(())
        }
    }

    pub fn outdent_line(&mut self) -> Result<()> {
        if self.selection.is_selection_active() {
            self.handle_selection_indent_outdent(|line| {
                if let Some(stripped) = line.strip_prefix("  ") {
                    stripped.to_string()
                } else if let Some(stripped) = line.strip_prefix(' ') {
                    stripped.to_string()
                } else {
                    line.to_string()
                }
            })
        } else {
            let y = self.cursor_y;
            if y >= self.document.lines.len() {
                return Ok(());
            }
            let original_line = self.document.lines[y].clone();
            let (new_line, new_cursor_x) = if let Some(stripped) = original_line.strip_prefix("  ")
            {
                (stripped.to_string(), self.cursor_x.saturating_sub(2))
            } else if let Some(stripped) = original_line.strip_prefix(' ') {
                (stripped.to_string(), self.cursor_x.saturating_sub(1))
            } else {
                (original_line.clone(), self.cursor_x)
            };

            if original_line == new_line {
                return Ok(()); // Nothing to outdent
            }

            self.commit(
                LastActionType::Outdent,
                &ActionDiff {
                    cursor_start_x: self.cursor_x,
                    cursor_start_y: self.cursor_y,
                    cursor_end_x: 0,
                    cursor_end_y: y,
                    start_x: 0,
                    start_y: y,
                    end_x: original_line.len(),
                    end_y: y,
                    new: vec![],
                    old: vec![original_line],
                },
            );
            self.commit(
                LastActionType::Ammend,
                &ActionDiff {
                    cursor_start_x: 0,
                    cursor_start_y: y,
                    cursor_end_x: new_cursor_x,
                    cursor_end_y: y,
                    start_x: 0,
                    start_y: y,
                    end_x: new_line.len(),
                    end_y: y,
                    new: vec![new_line],
                    old: vec![],
                },
            );
            Ok(())
        }
    }
}
