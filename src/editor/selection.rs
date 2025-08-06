use crate::document::Document;
use crate::error::Result;

pub struct Selection {
    pub marker_pos: Option<(usize, usize)>,
}

impl Default for Selection {
    fn default() -> Self {
        Self::new()
    }
}

impl Selection {
    pub fn new() -> Self {
        Self { marker_pos: None }
    }

    pub fn set_marker(&mut self, cursor_pos: (usize, usize)) {
        self.marker_pos = Some(cursor_pos);
    }

    pub fn clear_marker(&mut self) {
        self.marker_pos = None;
    }

    pub fn is_selection_active(&self) -> bool {
        self.marker_pos.is_some()
    }

    pub fn get_selection_range(
        &self,
        cursor_pos: (usize, usize),
    ) -> Option<((usize, usize), (usize, usize))> {
        if let Some(marker) = self.marker_pos {
            if marker.1 < cursor_pos.1 || (marker.1 == cursor_pos.1 && marker.0 < cursor_pos.0) {
                Some((marker, cursor_pos))
            } else {
                Some((cursor_pos, marker))
            }
        } else {
            None
        }
    }

    pub fn cut_selection(
        &mut self,
        document: &mut Document,
        cursor_pos: (usize, usize),
    ) -> Result<String> {
        if let Some(((start_x, start_y), (end_x, end_y))) = self.get_selection_range(cursor_pos) {
            let mut killed_text = String::new();

            if start_y == end_y {
                // Single line selection
                let line = &mut document.lines[start_y];
                let removed = line.drain(start_x..end_x).collect::<String>();
                killed_text.push_str(&removed);
            } else {
                // Multi-line selection
                // Part of the start line
                let start_line = &mut document.lines[start_y];
                let removed_start = start_line.drain(start_x..).collect::<String>();
                killed_text.push_str(&removed_start);
                killed_text.push('\n');

                // Full lines in between
                for _ in (start_y + 1)..end_y {
                    killed_text.push_str(&document.lines.remove(start_y + 1));
                    killed_text.push('\n');
                }

                // Part of the end line
                let end_line = document.lines.remove(start_y + 1);
                killed_text.push_str(&end_line[..end_x]);
                document.lines[start_y].push_str(&end_line[end_x..]);
            }

            self.clear_marker();
            Ok(killed_text)
        } else {
            Ok(String::new())
        }
    }

    pub fn copy_selection(
        &mut self,
        document: &Document,
        cursor_pos: (usize, usize),
    ) -> Result<String> {
        if let Some(((start_x, start_y), (end_x, end_y))) = self.get_selection_range(cursor_pos) {
            let mut copied_text = String::new();

            if start_y == end_y {
                // Single line selection
                copied_text.push_str(&document.lines[start_y][start_x..end_x]);
            } else {
                // Multi-line selection
                // Part of the start line
                copied_text.push_str(&document.lines[start_y][start_x..]);
                copied_text.push('\n');

                // Full lines in between
                for i in (start_y + 1)..end_y {
                    copied_text.push_str(&document.lines[i]);
                    copied_text.push('\n');
                }

                // Part of the end line
                copied_text.push_str(&document.lines[end_y][..end_x]);
            }
            self.clear_marker();
            Ok(copied_text)
        } else {
            Ok(String::new())
        }
    }
}
