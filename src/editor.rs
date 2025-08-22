use log::debug;
use std::time::{Duration, Instant};
use unicode_width::UnicodeWidthChar;

use crate::document::{ActionDiff, Diff, Document};
use crate::editor::search::Search;
use crate::error::Result;

pub mod command;
pub mod input;
pub mod search;
pub mod selection;
pub mod ui;
use crate::editor::ui::STATUS_BAR_HEIGHT;

const TAB_STOP: usize = 4;

#[derive(PartialEq, Debug)]
pub enum LastActionType {
    None,
    Insertion,
    Deletion,
    Newline,
    LineMovement,
    ToggleCheckbox, // For checkbox toggling
    Other,          // For actions like kill_line, yank, etc.
}

pub struct Editor {
    pub should_quit: bool,
    pub document: Document,
    pub cursor_x: usize, // byte index
    pub cursor_y: usize,
    pub desired_cursor_x: usize, // column index
    pub status_message: String,
    pub row_offset: usize, // public for tests
    pub col_offset: usize, // public for tests
    pub screen_rows: usize,
    pub screen_cols: usize,
    pub undo_stack: Vec<Vec<ActionDiff>>,
    pub redo_stack: Vec<Vec<ActionDiff>>,
    pub kill_buffer: String,
    pub last_action_was_kill: bool,
    pub is_alt_pressed: bool,
    pub search: Search,
    pub selection: selection::Selection,
    // New fields for debouncing
    last_action_time: Option<Instant>,
    last_action_type: LastActionType,
    undo_debounce_threshold: Duration,
}

impl Editor {
    pub fn new(filename: Option<String>) -> Self {
        let document = match filename {
            Some(fname) => {
                if let Ok(doc) = Document::open(&fname) {
                    doc
                } else {
                    let mut doc = Document::new_empty();
                    doc.filename = Some(fname);
                    doc
                }
            }
            None => Document::default(),
        };

        // Save the initial state for undo after construction
        Self {
            should_quit: false,
            document,
            cursor_x: 0,
            cursor_y: 0,
            desired_cursor_x: 0,
            status_message: "".to_string(),
            row_offset: 0,
            col_offset: 0,
            screen_rows: 0,
            screen_cols: 0,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            kill_buffer: String::new(),
            last_action_was_kill: false,
            is_alt_pressed: false,
            search: Search::new(),
            selection: selection::Selection::new(),
            // Initialize new fields
            last_action_time: None,
            last_action_type: LastActionType::None,
            undo_debounce_threshold: Duration::from_millis(500),
        }
    }

    pub fn update_screen_size(&mut self, screen_rows: usize, screen_cols: usize) {
        self.screen_rows = screen_rows;
        self.screen_cols = screen_cols;
    }

    pub fn save_state_for_undo(&mut self, current_action_type: LastActionType) {
        let now = Instant::now();
        debug!(
            "save_state_for_undo: current_action_type={:?}, last_action_type={:?}, undo_debounce_threshold={:?}",
            current_action_type, self.last_action_type, self.undo_debounce_threshold
        );

        let should_start_new_group = if self.last_action_time.is_none() {
            debug!("save_state_for_undo: First action ever");
            true // Always start new group for the very first action
        } else if current_action_type == LastActionType::ToggleCheckbox {
            debug!("save_state_for_undo: ToggleCheckbox always starts a new group");
            true
        } else {
            let time_since_last_action = now.duration_since(self.last_action_time.unwrap());
            debug!("save_state_for_undo: time_since_last_action={time_since_last_action:?}");
            self.last_action_type != current_action_type // Action type changed
            || time_since_last_action >= self.undo_debounce_threshold // Debounce time exceeded
        };

        if should_start_new_group {
            debug!("save_state_for_undo: Pushing new undo group");
            self.undo_stack.push(Vec::new()); // Push a new empty vector for the new transaction
            self.redo_stack.clear(); // Clear redo stack on new action
        }
        self.last_action_time = Some(now);
        self.last_action_type = current_action_type;
    }

    pub fn undo(&mut self) {
        self.last_action_was_kill = false;
        debug!(
            "Undo called. Current undo_stack length: {}. Current document: {:?}",
            self.undo_stack.len(),
            self.document.lines
        );
        if let Some(mut actions_to_undo) = self.undo_stack.pop() {
            let mut actions_for_redo = Vec::new();
            let mut current_cursor_x = self.cursor_x;
            let mut current_cursor_y = self.cursor_y;

            // Apply actions in reverse order for undo
            actions_to_undo.reverse();
            for action_diff in actions_to_undo.iter() {
                match self.document.apply_action_diff(action_diff, true) {
                    Ok((new_x, new_y)) => {
                        current_cursor_x = new_x;
                        current_cursor_y = new_y;
                        actions_for_redo.push(action_diff.clone()); // Store for redo
                    }
                    Err(e) => {
                        self.status_message = format!("Undo failed: {e:?}");
                        debug!("Undo failed: {e:?}");
                        // Re-push the failed transaction back to undo_stack if partial undo is not desired
                        self.undo_stack.push(actions_to_undo);
                        return;
                    }
                }
            }
            actions_for_redo.reverse();
            self.redo_stack.push(actions_for_redo);

            self.cursor_x = current_cursor_x;
            self.cursor_y = current_cursor_y;
            self.desired_cursor_x =
                self.get_display_width(&self.document.lines[self.cursor_y], self.cursor_x);
            self.status_message = "Undo successful.".to_string();
            debug!("Document after undo: {:?}", self.document.lines);
        } else {
            self.status_message = "Nothing to undo.".to_string();
            debug!("Undo stack is empty. Nothing to undo.");
        }
    }

    pub fn redo(&mut self) {
        self.last_action_was_kill = false;
        debug!(
            "Redo called. Current redo_stack length: {}. Current document: {:?}",
            self.redo_stack.len(),
            self.document.lines
        );
        if let Some(actions_to_redo) = self.redo_stack.pop() {
            let mut actions_for_undo = Vec::new();
            let mut current_cursor_x = self.cursor_x;
            let mut current_cursor_y = self.cursor_y;

            // Apply actions in original order for redo
            for action_diff in actions_to_redo.iter() {
                match self.document.apply_action_diff(action_diff, false) {
                    Ok((new_x, new_y)) => {
                        current_cursor_x = new_x;
                        current_cursor_y = new_y;
                        actions_for_undo.push(action_diff.clone()); // Store for undo
                    }
                    Err(e) => {
                        self.status_message = format!("Redo failed: {e:?}");
                        debug!("Redo failed: {e:?}");
                        // Re-push the failed transaction back to redo_stack if partial redo is not desired
                        self.redo_stack.push(actions_to_redo);
                        return;
                    }
                }
            }
            self.undo_stack.push(actions_for_undo);

            self.cursor_x = current_cursor_x;
            self.cursor_y = current_cursor_y;
            self.desired_cursor_x =
                self.get_display_width(&self.document.lines[self.cursor_y], self.cursor_x);
            self.status_message = "Redo successful.".to_string();
            debug!("Document after redo: {:?}", self.document.lines);
        } else {
            self.status_message = "Nothing to redo.".to_string();
            debug!("Redo stack is empty. Nothing to redo.");
        }
    }

    pub fn insert_text(&mut self, text: &str) -> Result<()> {
        self.last_action_was_kill = false;
        self.save_state_for_undo(LastActionType::Insertion);
        let diff = Diff {
            x: self.cursor_x,
            y: self.cursor_y,
            added_text: text.to_string(),
            deleted_text: "".to_string(),
        };
        let action_diff = ActionDiff::CharChange(diff);
        let (new_x, new_y) = self.document.apply_action_diff(&action_diff, false)?;
        if let Some(last_transaction) = self.undo_stack.last_mut() {
            last_transaction.push(action_diff);
        }
        self.cursor_x = new_x;
        self.cursor_y = new_y;
        self.desired_cursor_x =
            self.get_display_width(&self.document.lines[self.cursor_y], self.cursor_x);
        self.status_message = "".to_string();
        Ok(())
    }

    pub fn delete_char(&mut self) -> Result<()> {
        self.last_action_was_kill = false;
        // Backspace
        self.save_state_for_undo(LastActionType::Deletion);
        if self.cursor_x > 0 {
            let line = &self.document.lines[self.cursor_y];
            let mut char_to_delete = String::new();
            let mut char_start_byte = 0;

            if let Some((idx, ch)) = line[..self.cursor_x].char_indices().next_back() {
                char_to_delete = ch.to_string();
                char_start_byte = idx;
            }
            let diff = Diff {
                x: char_start_byte,
                y: self.cursor_y,
                added_text: "".to_string(),
                deleted_text: char_to_delete,
            };
            let action_diff = ActionDiff::CharChange(diff);
            let (new_x, new_y) = self.document.apply_action_diff(&action_diff, false)?;
            if let Some(last_transaction) = self.undo_stack.last_mut() {
                last_transaction.push(action_diff);
            }
            self.cursor_x = new_x;
            self.cursor_y = new_y;
        } else if self.cursor_y > 0 {
            let original_x = 0;
            let original_y = self.cursor_y;
            let (undo_x, undo_y) = self.document.delete_newline(original_x, original_y)?;

            let action_diff = ActionDiff::NewlineDeletion {
                original_x,
                original_y,
                undo_x,
                undo_y,
            };
            if let Some(last_transaction) = self.undo_stack.last_mut() {
                last_transaction.push(action_diff);
            }
            self.cursor_x = undo_x;
            self.cursor_y = undo_y;
            self.desired_cursor_x =
                self.get_display_width(&self.document.lines[self.cursor_y], self.cursor_x);
        }
        Ok(())
    }

    pub fn delete_forward_char(&mut self) -> Result<()> {
        self.last_action_was_kill = false;
        // Ctrl-D
        self.save_state_for_undo(LastActionType::Deletion);
        let y = self.cursor_y;
        let x = self.cursor_x;
        let line_len = self.document.lines.get(y).map_or(0, |l| l.len());
        if x < line_len {
            let line = &self.document.lines[y];
            let mut char_to_delete = String::new();
            let mut char_start_byte = 0;

            if let Some((idx, ch)) = line[x..].char_indices().next() {
                char_to_delete = ch.to_string();
                char_start_byte = x + idx;
            }
            let diff = Diff {
                x: char_start_byte,
                y,
                added_text: "".to_string(),
                deleted_text: char_to_delete,
            };
            let action_diff = ActionDiff::CharChange(diff);
            let (_new_x, new_y) = self.document.apply_action_diff(&action_diff, false)?;
            if let Some(last_transaction) = self.undo_stack.last_mut() {
                last_transaction.push(action_diff);
            }
            // Cursor position does not change for delete_forward_char, but update y in case modify changes it
            self.cursor_y = new_y;
        } else if y < self.document.lines.len() - 1 {
            let original_x = self.document.lines[y].len();
            let original_y = y;
            let (undo_x, undo_y) = self.document.delete_newline(original_x, original_y)?;

            let action_diff = ActionDiff::NewlineDeletion {
                original_x,
                original_y,
                undo_x,
                undo_y,
            };
            if let Some(last_transaction) = self.undo_stack.last_mut() {
                last_transaction.push(action_diff);
            }
            self.cursor_x = undo_x;
            self.cursor_y = undo_y;
        }
        Ok(())
    }

    pub fn insert_newline(&mut self) -> Result<()> {
        self.last_action_was_kill = false;

        let y = self.cursor_y;
        let x = self.cursor_x;
        let current_line = self.document.lines[y].clone();

        // Check for command execution
        if x == current_line.len() {
            if let Some(command_output) = command::execute_command(&current_line) {
                self.save_state_for_undo(LastActionType::Other);
                let delete_diff = Diff {
                    x: 0,
                    y,
                    added_text: "".to_string(),
                    deleted_text: current_line.clone(),
                };
                let delete_action = ActionDiff::CharChange(delete_diff);
                let (new_x, new_y) = self.document.apply_action_diff(&delete_action, false)?;
                if let Some(last_transaction) = self.undo_stack.last_mut() {
                    last_transaction.push(delete_action);
                }
                self.cursor_x = new_x;
                self.cursor_y = new_y;

                let insert_diff = Diff {
                    x: self.cursor_x,
                    y: self.cursor_y,
                    added_text: command_output.clone(),
                    deleted_text: "".to_string(),
                };
                let insert_action = ActionDiff::CharChange(insert_diff);
                let (new_x, new_y) = self.document.apply_action_diff(&insert_action, false)?;
                if let Some(last_transaction) = self.undo_stack.last_mut() {
                    last_transaction.push(insert_action);
                }
                self.cursor_x = new_x;
                self.cursor_y = new_y;
                self.desired_cursor_x =
                    self.get_display_width(&self.document.lines[self.cursor_y], self.cursor_x);
                self.status_message = current_line.to_string();

                // Insert a newline after the command output
                self.insert_newline()?;

                return Ok(());
            }
        }

        self.save_state_for_undo(LastActionType::Newline);
        let action_diff = ActionDiff::NewlineInsertion {
            x: self.cursor_x,
            y: self.cursor_y,
        };
        let (new_x, new_y) = self.document.apply_action_diff(&action_diff, false)?;
        if let Some(last_transaction) = self.undo_stack.last_mut() {
            last_transaction.push(action_diff);
        }
        self.cursor_y = new_y;
        self.cursor_x = new_x;
        self.desired_cursor_x = 0;
        Ok(())
    }

    pub fn kill_line(&mut self) -> Result<()> {
        let should_clear_kill_buffer = !self.last_action_was_kill;
        self.save_state_for_undo(LastActionType::Deletion); // Start a new transaction for kill_line

        let y = self.cursor_y;
        let x = self.cursor_x;
        if y >= self.document.lines.len() {
            return Ok(());
        }

        if should_clear_kill_buffer {
            self.kill_buffer.clear();
        }

        let current_line_len = self.document.lines[y].len();

        if x == 0 && current_line_len == 0 && y < self.document.lines.len() - 1 {
            // Case 3: Cursor is at the beginning of an empty line, and it's not the last line
            // Kill the empty line and move cursor to the beginning of the next line
            if let Some(last_transaction) = self.undo_stack.last_mut() {
                // To undo this, we need to re-insert the empty line at 'y'
                let action_diff = ActionDiff::NewlineInsertion { x: 0, y }; // This will be used for undo
                self.document.lines.remove(y); // Directly remove the line
                last_transaction.push(action_diff);
                self.kill_buffer.push('\x0a'); // A newline was killed
                self.cursor_x = 0; // Cursor moves to beginning of the line
                // self.cursor_y remains 'y' as the line at 'y' is now the one that was at 'y+1'
            }
        } else if x < current_line_len {
            // Case 1: Cursor is within the line (not at the very end)
            // Kill from cursor to end of line
            let current_line = self.document.lines[y].clone();
            let killed_text = current_line[x..].to_string();
            let diff = Diff {
                x,
                y,
                added_text: "".to_string(),
                deleted_text: killed_text.clone(),
            };
            let action_diff = ActionDiff::CharChange(diff);
            if let Some(last_transaction) = self.undo_stack.last_mut() {
                let (new_x, new_y) = self.document.apply_action_diff(&action_diff, false)?;
                last_transaction.push(action_diff);
                self.kill_buffer.push_str(&killed_text);
                self.cursor_x = new_x;
                self.cursor_y = new_y;
            }
        } else if x == current_line_len && y < self.document.lines.len() - 1 {
            // Case 2: Cursor is at the end of the line, and it's not the last line
            // Kill the newline and join with the next line
            let next_line_content = self.document.lines[y + 1].clone();
            let original_x = x;
            let original_y = y;
            let (undo_x, undo_y) = self.document.delete_newline(original_x, original_y)?;

            let action_diff = ActionDiff::NewlineDeletion {
                original_x,
                original_y,
                undo_x,
                undo_y,
            };
            if let Some(last_transaction) = self.undo_stack.last_mut() {
                last_transaction.push(action_diff);
                self.kill_buffer.push('\x0a');
                self.kill_buffer.push_str(&next_line_content);
                self.cursor_x = undo_x;
                self.cursor_y = undo_y;
            }
        }
        self.last_action_was_kill = true;
        Ok(())
    }

    pub fn yank(&mut self) -> Result<()> {
        self.last_action_was_kill = false;
        self.save_state_for_undo(LastActionType::Insertion); // Start a new transaction for yank

        let text_to_yank = self.kill_buffer.clone();

        if text_to_yank.is_empty() {
            self.status_message = "Kill buffer is empty.".to_string();
            return Ok(());
        }

        let lines_to_yank: Vec<&str> = text_to_yank.split('\x0a').collect();

        if let Some(last_transaction) = self.undo_stack.last_mut() {
            // Insert the first part of the yanked text into the current line
            if let Some(first_line) = lines_to_yank.first() {
                if !first_line.is_empty() {
                    let diff = Diff {
                        x: self.cursor_x,
                        y: self.cursor_y,
                        added_text: first_line.to_string(),
                        deleted_text: "".to_string(),
                    };
                    let action_diff = ActionDiff::CharChange(diff);
                    let (new_x, new_y) = self.document.apply_action_diff(&action_diff, false)?;
                    last_transaction.push(action_diff);
                    self.cursor_x = new_x;
                    self.cursor_y = new_y;
                }
            }

            // Insert subsequent lines
            for line_to_yank in lines_to_yank.iter().skip(1) {
                let action_diff_newline = ActionDiff::NewlineInsertion {
                    x: self.cursor_x,
                    y: self.cursor_y,
                };
                let (new_x_after_newline, new_y_after_newline) = self
                    .document
                    .apply_action_diff(&action_diff_newline, false)?;
                last_transaction.push(action_diff_newline);
                self.cursor_y = new_y_after_newline;
                self.cursor_x = new_x_after_newline;

                if !line_to_yank.is_empty() {
                    let diff = Diff {
                        x: self.cursor_x,
                        y: self.cursor_y,
                        added_text: line_to_yank.to_string(),
                        deleted_text: "".to_string(),
                    };
                    let action_diff_char = ActionDiff::CharChange(diff);
                    let (new_x, new_y) =
                        self.document.apply_action_diff(&action_diff_char, false)?;
                    last_transaction.push(action_diff_char);
                    self.cursor_x = new_x;
                    self.cursor_y = new_y;
                }
            }
        }

        self.desired_cursor_x =
            self.get_display_width(&self.document.lines[self.cursor_y], self.cursor_x);
        Ok(())
    }

    pub fn hungry_delete(&mut self) -> Result<()> {
        self.save_state_for_undo(LastActionType::Deletion);
        let (x, y) = (self.cursor_x, self.cursor_y);
        if y >= self.document.lines.len() {
            return Ok(());
        }

        let current_line = &mut self.document.lines[y];

        if let Some(last_transaction) = self.undo_stack.last_mut() {
            if x == 0 {
                // If at the beginning of a line, join with previous line if available
                if y > 0 {
                    let original_x = 0;
                    let original_y = y;
                    let (undo_x, undo_y) = self.document.delete_newline(original_x, original_y)?;

                    let action_diff = ActionDiff::NewlineDeletion {
                        original_x,
                        original_y,
                        undo_x,
                        undo_y,
                    };
                    if let Some(last_transaction) = self.undo_stack.last_mut() {
                        last_transaction.push(action_diff);
                        self.cursor_x = undo_x;
                        self.cursor_y = undo_y;
                        self.desired_cursor_x = self
                            .get_display_width(&self.document.lines[self.cursor_y], self.cursor_x);
                    }
                }
                return Ok(());
            }

            let start_delete_byte = find_word_boundary_left(current_line, x);

            // Need to clone the line content before modification for the Diff
            let line_content_before_delete = current_line.clone();
            let deleted_text = line_content_before_delete[start_delete_byte..x].to_string();
            let diff = Diff {
                x: start_delete_byte,
                y,
                added_text: "".to_string(),
                deleted_text,
            };
            let action_diff = ActionDiff::CharChange(diff);
            let (new_x, new_y) = self.document.apply_action_diff(&action_diff, false)?;
            last_transaction.push(action_diff);
            self.cursor_x = new_x;
            self.cursor_y = new_y;
            self.desired_cursor_x =
                self.get_display_width(&self.document.lines[self.cursor_y], self.cursor_x);
        }
        Ok(())
    }

    pub fn go_to_start_of_line(&mut self) {
        self.last_action_was_kill = false;
        self.cursor_x = 0;
        self.desired_cursor_x = 0;
    }

    pub fn go_to_end_of_line(&mut self) {
        self.last_action_was_kill = false;
        let y = self.cursor_y;
        self.cursor_x = self.document.lines[y].len();
        self.desired_cursor_x = self.get_display_width(&self.document.lines[y], self.cursor_x);
    }

    pub fn move_cursor_word_left(&mut self) -> Result<()> {
        self.last_action_was_kill = false;
        let current_line = &self.document.lines[self.cursor_y];
        let mut new_cursor_x = self.cursor_x;

        if new_cursor_x == 0 {
            if self.cursor_y > 0 {
                self.cursor_y -= 1;
                self.cursor_x = self.document.lines[self.cursor_y].len();
                self.desired_cursor_x =
                    self.get_display_width(&self.document.lines[self.cursor_y], self.cursor_x);
            }
            return Ok(());
        }

        let mut chars_iter = current_line[..new_cursor_x].char_indices().rev();

        // Step 1: Skip any non-word characters (including whitespace) to the left
        // until we hit a word character or the beginning of the line.
        let mut found_word_char = false;
        for (idx, ch) in chars_iter.by_ref() {
            if is_word_char(ch) {
                new_cursor_x = idx; // This is the start of a word
                found_word_char = true;
                break;
            }
            new_cursor_x = idx; // Keep moving left
        }

        // Step 2: If we found a word character, now skip all word characters
        // to find the actual beginning of the word.
        if found_word_char {
            for (idx, ch) in chars_iter {
                if !is_word_char(ch) {
                    new_cursor_x = idx + ch.len_utf8();
                    break;
                }
                new_cursor_x = idx; // Keep moving left
            }
        }

        self.cursor_x = new_cursor_x;
        self.desired_cursor_x = self.get_display_width(current_line, self.cursor_x);
        Ok(())
    }

    pub fn move_cursor_word_right(&mut self) -> Result<()> {
        self.last_action_was_kill = false;
        let current_line = &self.document.lines[self.cursor_y];
        let line_len = current_line.len();

        if self.cursor_x == line_len {
            if self.cursor_y < self.document.lines.len() - 1 {
                self.cursor_y += 1;
                self.cursor_x = 0;
                self.desired_cursor_x = 0;
            }
            return Ok(());
        }

        let mut new_cursor_x = self.cursor_x;
        let mut chars_iter = current_line[new_cursor_x..].chars().peekable();

        // Step 1: Skip any non-word characters (including whitespace)
        // until we hit a word character or the end of the line.
        while let Some(&ch) = chars_iter.peek() {
            if is_word_char(ch) {
                break; // Found start of a word
            }
            new_cursor_x += ch.len_utf8();
            chars_iter.next(); // Consume the character
        }

        // Step 2: Skip all word characters
        while let Some(&ch) = chars_iter.peek() {
            if !is_word_char(ch) {
                break; // Found end of a word
            }
            new_cursor_x += ch.len_utf8();
            chars_iter.next(); // Consume the character
        }

        self.cursor_x = new_cursor_x;
        self.desired_cursor_x = self.get_display_width(current_line, self.cursor_x);
        Ok(())
    }

    pub fn save_document(&mut self) -> Result<()> {
        self.last_action_was_kill = false;
        self.document.save(None)?;
        self.status_message = "File saved successfully.".to_string();
        debug!("Document saved.");
        Ok(())
    }

    pub fn quit(&mut self) -> Result<()> {
        self.last_action_was_kill = false;
        self.document.save(None)?;
        self.should_quit = true;
        debug!("Editor quitting.");
        Ok(())
    }

    pub fn clamp_cursor_x(&mut self) {
        if self.cursor_y >= self.document.lines.len() {
            self.cursor_x = 0;
            return;
        }
        let line_len = self.document.lines[self.cursor_y].len();
        if self.cursor_x > line_len {
            self.cursor_x = line_len;
        }
    }

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

    pub fn get_byte_pos_from_display_width(&self, display_x: usize) -> usize {
        let line = &self.document.lines[self.cursor_y];
        let mut current_display_x = 0;
        let mut byte_pos = 0;
        for ch in line.chars() {
            if current_display_x >= display_x {
                break;
            }
            if ch == '\x09' {
                current_display_x += TAB_STOP - (current_display_x % TAB_STOP);
            } else {
                current_display_x += ch.width().unwrap_or(0);
            }
            if current_display_x > display_x {
                break;
            }
            byte_pos += ch.len_utf8();
        }
        byte_pos
    }

    pub fn cursor_pos(&self) -> (usize, usize) {
        (self.cursor_x, self.cursor_y)
    }

    pub fn set_cursor_pos(&mut self, x: usize, y: usize) {
        self.cursor_x = x;
        self.cursor_y = y;
        self.clamp_cursor_x();
    }

    pub fn set_message(&mut self, message: &str) {
        self.status_message = message.to_string();
    }

    pub fn move_line_up(&mut self) {
        self.last_action_was_kill = false;
        if self.cursor_y > 0 {
            self.save_state_for_undo(LastActionType::LineMovement);
            let action_diff = ActionDiff::LineSwap {
                y1: self.cursor_y - 1,
                y2: self.cursor_y,
                original_cursor_x: self.cursor_x,
                original_cursor_y: self.cursor_y,
                new_cursor_x: self.cursor_x,
                new_cursor_y: self.cursor_y - 1,
            };
            if let Some(last_transaction) = self.undo_stack.last_mut() {
                if let Ok((_new_x, _new_y)) = self.document.apply_action_diff(&action_diff, false) {
                    last_transaction.push(action_diff);
                    self.cursor_y -= 1; // Explicitly move cursor up
                }
            }
        } else {
            self.status_message = "Cannot move line up further.".to_string();
        }
    }

    pub fn move_line_down(&mut self) {
        self.last_action_was_kill = false;
        if self.cursor_y < self.document.lines.len() - 1 {
            self.save_state_for_undo(LastActionType::LineMovement);
            let action_diff = ActionDiff::LineSwap {
                y1: self.cursor_y,
                y2: self.cursor_y + 1,
                original_cursor_x: self.cursor_x,
                original_cursor_y: self.cursor_y,
                new_cursor_x: self.cursor_x,
                new_cursor_y: self.cursor_y + 1,
            };
            if let Some(last_transaction) = self.undo_stack.last_mut() {
                if let Ok((_new_x, _new_y)) = self.document.apply_action_diff(&action_diff, false) {
                    last_transaction.push(action_diff);
                    self.cursor_y += 1; // Explicitly move cursor down
                }
            }
        } else {
            self.status_message = "Cannot move line down further.".to_string();
        }
    }

    pub fn scroll_page_down(&mut self) {
        self.last_action_was_kill = false;
        let page_height = self.screen_rows.saturating_sub(STATUS_BAR_HEIGHT).max(1);
        self.row_offset = self.row_offset.saturating_add(page_height);
        self.row_offset = self
            .row_offset
            .min(self.document.lines.len().saturating_sub(1));
        self.cursor_y = self.row_offset;
        self.clamp_cursor_x();
    }

    pub fn scroll_page_up(&mut self) {
        self.last_action_was_kill = false;
        let page_height = self.screen_rows.saturating_sub(STATUS_BAR_HEIGHT).max(1);
        self.row_offset = self.row_offset.saturating_sub(page_height);
        self.cursor_y = self.row_offset;
        self.clamp_cursor_x();
    }

    pub fn go_to_start_of_file(&mut self) {
        self.last_action_was_kill = false;
        self.cursor_y = 0;
        self.cursor_x = 0;
        self.desired_cursor_x = 0;
        self.row_offset = 0;
        self.col_offset = 0;
    }

    pub fn go_to_end_of_file(&mut self) {
        self.last_action_was_kill = false;
        self.cursor_y = self.document.lines.len().saturating_sub(1);
        self.cursor_x = self.document.lines[self.cursor_y].len();
        self.desired_cursor_x =
            self.get_display_width(&self.document.lines[self.cursor_y], self.cursor_x);
        let screen_height = self.screen_rows.saturating_sub(1);
        if self.cursor_y >= self.row_offset + screen_height {
            self.row_offset = self.cursor_y.saturating_sub(screen_height) + 1;
        }
        self.clamp_cursor_x();
    }

    pub fn move_cursor_up(&mut self) {
        self.last_action_was_kill = false;
        if self.cursor_y > 0 {
            self.cursor_y -= 1;
            self.cursor_x = self.get_byte_pos_from_display_width(self.desired_cursor_x);
        } else {
            self.cursor_x = 0;
            self.desired_cursor_x = 0;
        }
    }

    pub fn move_cursor_down(&mut self) {
        self.last_action_was_kill = false;
        if self.cursor_y < self.document.lines.len() - 1 {
            self.cursor_y += 1;
            self.cursor_x = self.get_byte_pos_from_display_width(self.desired_cursor_x);
        } else {
            self.go_to_end_of_line();
        }
    }

    pub fn move_cursor_left(&mut self) {
        self.last_action_was_kill = false;
        let line = &self.document.lines[self.cursor_y];
        if self.cursor_x > 0 {
            let mut new_pos = self.cursor_x - 1;
            while !line.is_char_boundary(new_pos) {
                new_pos -= 1;
            }
            self.cursor_x = new_pos;
            self.desired_cursor_x = self.get_display_width(line, self.cursor_x);
        } else if self.cursor_y > 0 {
            self.cursor_y -= 1;
            self.cursor_x = self.document.lines[self.cursor_y].len();
            self.desired_cursor_x =
                self.get_display_width(&self.document.lines[self.cursor_y], self.cursor_x);
        }
    }

    pub fn move_cursor_right(&mut self) {
        self.last_action_was_kill = false;
        let line = &self.document.lines[self.cursor_y];
        if self.cursor_x < line.len() {
            let mut new_pos = self.cursor_x + 1;
            while !line.is_char_boundary(new_pos) {
                new_pos += 1;
            }
            self.cursor_x = new_pos;
            self.desired_cursor_x = self.get_display_width(line, self.cursor_x);
        } else if self.cursor_y < self.document.lines.len() - 1 {
            self.cursor_y += 1;
            self.cursor_x = 0;
            self.desired_cursor_x = 0;
        }
    }

    pub fn set_alt_pressed(&mut self, is_alt_pressed: bool) {
        self.is_alt_pressed = is_alt_pressed;
    }

    pub fn set_marker_action(&mut self) {
        self.selection.set_marker(self.cursor_pos());
        self.status_message = "Marker set.".to_string();
    }

    pub fn clear_marker_action(&mut self) {
        self.selection.clear_marker();
        self.status_message = "Marker cleared.".to_string();
    }

    pub fn cut_selection_action(&mut self) -> Result<()> {
        self.save_state_for_undo(LastActionType::Deletion);
        let cursor_pos = self.cursor_pos();
        let (killed_text, action_diff_option) =
            self.selection.cut_selection(&self.document, cursor_pos)?;

        if let Some(action_diff) = action_diff_option {
            let (new_x, new_y) = self.document.apply_action_diff(&action_diff, false)?;
            if let Some(last_transaction) = self.undo_stack.last_mut() {
                last_transaction.push(action_diff);
            }
            self.cursor_x = new_x;
            self.cursor_y = new_y;
            self.desired_cursor_x =
                self.get_display_width(&self.document.lines[self.cursor_y], self.cursor_x);
        }

        self.kill_buffer = killed_text;
        self.status_message = "Selection cut.".to_string();
        debug!("Selection cut. Kill buffer: '{}'", self.kill_buffer);

        Ok(())
    }

    pub fn copy_selection_action(&mut self) -> Result<()> {
        let cursor_pos = self.cursor_pos();
        self.kill_buffer = self.selection.copy_selection(&self.document, cursor_pos)?;
        self.status_message = "Selection copied.".to_string();
        debug!("Selection copied. Kill buffer: '{}'", self.kill_buffer);
        Ok(())
    }

    pub fn move_to_next_delimiter(&mut self) {
        self.last_action_was_kill = false;
        let current_line_idx = self.cursor_y;
        let num_lines = self.document.lines.len();

        if num_lines == 0 {
            return; // Nothing to do in an empty document
        }

        let mut target_line_y: Option<usize> = None;

        // Scenario 1: Current line is a delimiter. Move to the line immediately after it.
        if current_line_idx < num_lines && self.document.lines[current_line_idx] == "---" {
            target_line_y = Some(current_line_idx + 1);
        } else {
            // Scenario 2: Current line is not a delimiter. Search for the next delimiter *after* the current position.
            for i in (current_line_idx + 1)..num_lines {
                if self.document.lines[i] == "---" {
                    target_line_y = Some(i + 1);
                    break;
                }
            }
            // If target_line_y is still None here, it means there are no delimiters
            // after the current position. According to the user's request, we should do nothing
            // in this case (no wrapping around to previous delimiters).
        }

        if let Some(new_cursor_y) = target_line_y {
            // If moving past the last line, and it was the last delimiter, do nothing.
            // This handles the case where the last delimiter is at the very end of the file.
            if new_cursor_y >= num_lines {
                return; // Do nothing if moving past the last delimiter and no more exist.
            }

            self.cursor_y = new_cursor_y;
            self.cursor_x = 0;
            self.desired_cursor_x = 0;
            self.row_offset = self.cursor_y; // Scroll to make cursor at top
        }
        // If target_line_y is None, do nothing, which is the desired behavior.
    }

    pub fn move_to_previous_delimiter(&mut self) {
        self.last_action_was_kill = false;
        let current_line_idx = self.cursor_y;
        let num_lines = self.document.lines.len();

        if num_lines == 0 {
            return; // Nothing to do in an empty document
        }

        let mut target_line_y: Option<usize> = None;

        // Iterate backwards from the line *before* the current cursor position
        // to find the closest "page position" above it.
        for i in (0..current_line_idx).rev() {
            // Check if 'i' itself is a page position (i.e., line 0 or line after a delimiter)
            if i == 0 {
                target_line_y = Some(0);
                break;
            }
            if self.document.lines[i - 1] == "---" {
                target_line_y = Some(i); // 'i' is the line after the delimiter at 'i-1'
                break;
            }
        }

        // If no page position found above (meaning we reached the beginning of the file
        // without finding a delimiter), move to page 0.
        if target_line_y.is_none() {
            target_line_y = Some(0);
        }

        if let Some(new_cursor_y) = target_line_y {
            self.cursor_y = new_cursor_y;
            self.cursor_x = 0;
            self.desired_cursor_x = 0;
            self.row_offset = self.cursor_y; // Scroll to make cursor at top
        }
    }

    pub fn toggle_checkbox(&mut self) -> Result<()> {
        self.last_action_was_kill = false;
        self.save_state_for_undo(LastActionType::ToggleCheckbox);

        let y = self.cursor_y;
        if y >= self.document.lines.len() {
            return Ok(());
        }

        let original_line = self.document.lines[y].clone();
        let (new_line, cursor_x_change, message) =
            if let Some(stripped) = original_line.strip_prefix("- [x] ") {
                (stripped.to_string(), -6isize, "Checkbox removed.")
            } else if let Some(stripped) = original_line.strip_prefix("- [ ] ") {
                (format!("- [x] {stripped}"), 0, "Checkbox checked.")
            } else {
                (format!("- [ ] {original_line}"), 6, "Checkbox added.")
            };

        let diff = Diff {
            x: 0,
            y,
            added_text: new_line,
            deleted_text: original_line,
        };
        let action_diff = ActionDiff::CharChange(diff);

        if let Some(last_transaction) = self.undo_stack.last_mut() {
            let (_new_x, new_y) = self.document.apply_action_diff(&action_diff, false)?;
            last_transaction.push(action_diff);

            self.cursor_y = new_y;
            // Adjust cursor_x based on the change in line length at the beginning
            if cursor_x_change > 0 {
                self.cursor_x += cursor_x_change as usize;
            } else {
                self.cursor_x = self.cursor_x.saturating_sub(cursor_x_change.unsigned_abs());
            }
            // Ensure cursor is not beyond the new line length
            self.clamp_cursor_x();
            self.desired_cursor_x =
                self.get_display_width(&self.document.lines[self.cursor_y], self.cursor_x);
            self.status_message = message.to_string();
        }

        Ok(())
    }

    pub fn set_undo_debounce_threshold(&mut self, threshold_ms: u64) {
        self.undo_debounce_threshold = Duration::from_millis(threshold_ms);
    }
}

fn find_word_boundary_left(line: &str, current_x: usize) -> usize {
    let mut delete_start = current_x;

    if delete_start == 0 {
        return 0;
    }

    let mut chars_to_left = line[..delete_start].char_indices().rev();

    // Step 1: Skip any trailing non-word characters (e.g., punctuation, spaces after a word)
    // until we hit a word character.
    let mut found_word_char = false;
    for (idx, ch) in chars_to_left.by_ref() {
        if is_word_char(ch) {
            delete_start = idx;
            found_word_char = true;
            break;
        }
        delete_start = idx;
    }

    // Step 2: If we found a word character, now skip all word characters
    // to find the actual beginning of the word.
    if found_word_char {
        for (idx, ch) in chars_to_left {
            if !is_word_char(ch) {
                delete_start = idx + ch.len_utf8();
                break;
            }
            delete_start = idx;
        }
    }

    // Step 3: Now, `delete_start` is at the beginning of the word (or the beginning of the line
    // if no word was found). We need to also delete any preceding whitespace.
    // Iterate left from `delete_start` to find the first non-whitespace character.
    // We need a new iterator for this, starting from `delete_start`.
    let mut final_delete_start = delete_start;
    let whitespace_chars_to_left = line[..delete_start].char_indices().rev();
    for (idx, ch) in whitespace_chars_to_left {
        if ch.is_whitespace() {
            final_delete_start = idx;
        } else {
            break;
        }
    }

    final_delete_start
}

fn is_word_char(ch: char) -> bool {
    ch.is_alphanumeric() || ch == '_' || ch.is_alphabetic()
}
