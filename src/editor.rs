use crate::document::{ActionDiff, Document};
use crate::editor::search::Search;
use crate::error::Result;
use crate::persistence::{self, CursorPosition};
use arboard::Clipboard;
use log::debug;

use std::time::{Duration, Instant};

pub mod checkbox;
pub mod command;
pub mod comment;
pub mod indent;
pub mod input;
pub mod scroll;
pub mod search;
pub mod selection;
pub mod task;
pub mod ui;
use crate::editor::scroll::Scroll;
pub mod fuzzy_search;
use crate::editor::task::Task;

#[derive(PartialEq, Debug)]
pub enum LastActionType {
    None,
    Insertion,
    Deletion,
    Newline,
    LineMovement,
    Ammend,
    ToggleCheckbox, // For checkbox toggling
    ToggleComment,  // For comment toggling
    Other,          // For actions like kill_line, yank, etc.
}

#[derive(PartialEq, Debug)]
pub enum EditorMode {
    Normal,
    TaskSelection,
    Search,
    FuzzySearch,
}

pub struct Editor {
    pub should_quit: bool,
    pub document: Document,
    pub cursor_x: usize, // byte index
    pub cursor_y: usize,
    pub desired_cursor_x: usize, // column index
    pub status_message: String,
    pub scroll: Scroll,
    pub undo_stack: Vec<Vec<ActionDiff>>,
    pub redo_stack: Vec<Vec<ActionDiff>>,
    pub kill_buffer: String,
    pub last_action_was_kill: bool,
    pub is_alt_pressed: bool,
    pub search: Search,
    pub selection: selection::Selection,
    pub no_exit_on_save: bool,
    // New fields for debouncing
    last_action_time: Option<Instant>,
    last_action_type: LastActionType,
    undo_debounce_threshold: Duration,
    // New fields for task command
    pub mode: EditorMode,
    pub task: Task,
    pub fuzzy_search: fuzzy_search::FuzzySearch,
    clipboard_enabled: bool,
}

impl Editor {
    pub fn new(filename: Option<String>) -> Self {
        let document = match filename {
            Some(fname) => {
                if let Ok(doc) = Document::open(&fname) {
                    let last_modified = doc.last_modified().ok();
                    if let Some(lm) = last_modified {
                        debug!(
                            "Attempting to restore cursor for file: {fname}, last_modified: {lm:?}"
                        );
                        if let Some((x, y, scroll_row, scroll_col)) =
                            persistence::get_cursor_position(&fname, lm)
                        {
                            debug!(
                                "Restoring cursor position for {fname}: ({x}, {y}), scroll: ({scroll_row}, {scroll_col})"
                            );
                            return Self {
                                should_quit: false,
                                document: doc,
                                cursor_x: x,
                                cursor_y: y,
                                desired_cursor_x: x,
                                status_message: "".to_string(),
                                scroll: Scroll::new_with_offset(scroll_row, scroll_col),
                                undo_stack: Vec::new(),
                                redo_stack: Vec::new(),
                                kill_buffer: String::new(),
                                last_action_was_kill: false,
                                is_alt_pressed: false,
                                search: Search::new(),
                                selection: selection::Selection::new(),
                                no_exit_on_save: false,
                                last_action_time: None,
                                last_action_type: LastActionType::None,
                                undo_debounce_threshold: Duration::from_millis(500),
                                // New fields for task command
                                mode: EditorMode::Normal,
                                task: Task::new(),
                                fuzzy_search: fuzzy_search::FuzzySearch::new(),
                                clipboard_enabled: true,
                            };
                        } else {
                            debug!(
                                "No matching cursor position found for {fname}. Starting at (0,0)."
                            );
                        }
                    } else {
                        debug!("Could not get last modified date for {fname}. Starting at (0,0).");
                    }
                    doc
                } else {
                    debug!("Could not open file {fname}. Creating new empty document.");
                    let mut doc = Document::new_empty();
                    doc.filename = Some(fname);
                    doc
                }
            }
            None => {
                debug!("No filename provided. Creating new empty document.");
                Document::default()
            }
        };

        // Save the initial state for undo after construction
        Self {
            should_quit: false,
            document,
            cursor_x: 0,
            cursor_y: 0,
            desired_cursor_x: 0,
            status_message: "".to_string(),
            scroll: Scroll::new(),
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            kill_buffer: String::new(),
            last_action_was_kill: false,
            is_alt_pressed: false,
            search: Search::new(),
            selection: selection::Selection::new(),
            no_exit_on_save: false,
            // Initialize new fields
            last_action_time: None,
            last_action_type: LastActionType::None,
            undo_debounce_threshold: Duration::from_millis(500),
            // New fields for task command
            mode: EditorMode::Normal,
            task: Task::new(),
            fuzzy_search: fuzzy_search::FuzzySearch::new(),
            clipboard_enabled: true,
        }
    }

    pub fn update_screen_size(&mut self, screen_rows: usize, screen_cols: usize) {
        self.scroll.update_screen_size(screen_rows, screen_cols);
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
        } else if current_action_type == LastActionType::Ammend {
            debug!("save_state_for_undo: Ammend");
            false
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
        if current_action_type != LastActionType::Ammend {
            self.last_action_type = current_action_type;
        }
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
            self.desired_cursor_x = self
                .scroll
                .get_display_width(&self.document.lines[self.cursor_y], self.cursor_x);
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
            self.desired_cursor_x = self
                .scroll
                .get_display_width(&self.document.lines[self.cursor_y], self.cursor_x);
            self.status_message = "Redo successful.".to_string();
            debug!("Document after redo: {:?}", self.document.lines);
        } else {
            self.status_message = "Nothing to redo.".to_string();
            debug!("Redo stack is empty. Nothing to redo.");
        }
    }

    pub(super) fn commit(&mut self, action_type: LastActionType, action_diff: &ActionDiff) {
        self.save_state_for_undo(action_type);
        if let Some(last_transaction) = self.undo_stack.last_mut() {
            last_transaction.push(action_diff.clone());
        }
        let (new_x, new_y) = self.document.apply_action_diff(action_diff, false).unwrap();
        self.cursor_x = new_x;
        self.cursor_y = new_y;
        self.desired_cursor_x = self
            .scroll
            .get_display_width(&self.document.lines[self.cursor_y], self.cursor_x);
    }

    pub fn insert_text(&mut self, text: &str) -> Result<()> {
        self.commit(
            LastActionType::Insertion,
            &ActionDiff {
                cursor_start_x: self.cursor_x,
                cursor_start_y: self.cursor_y,
                cursor_end_x: self.cursor_x + text.len(),
                cursor_end_y: self.cursor_y,
                start_x: self.cursor_x,
                start_y: self.cursor_y,
                end_x: self.cursor_x + text.len(),
                end_y: self.cursor_y,
                new: vec![text.to_string()],
                old: vec![],
            },
        );
        self.status_message = "".to_string();
        Ok(())
    }

    pub fn delete_char(&mut self) -> Result<()> {
        self.last_action_was_kill = false;
        // Backspace
        if self.cursor_x > 0 {
            let line = &self.document.lines[self.cursor_y];
            let prefix = &line[..self.cursor_x];
            if prefix.chars().all(|c| c.is_whitespace()) && prefix.ends_with("  ") {
                // Delete 2 spaces
                let char_start_byte = self.cursor_x - 2;
                self.commit(
                    LastActionType::Deletion,
                    &ActionDiff {
                        cursor_start_x: self.cursor_x,
                        cursor_start_y: self.cursor_y,
                        cursor_end_x: char_start_byte,
                        cursor_end_y: self.cursor_y,
                        start_x: char_start_byte,
                        start_y: self.cursor_y,
                        end_x: self.cursor_x,
                        end_y: self.cursor_y,
                        new: vec![],
                        old: vec!["  ".to_string()],
                    },
                );
                return Ok(());
            }

            let mut char_to_delete = String::new();
            let mut char_start_byte = 0;

            if let Some((idx, ch)) = line[..self.cursor_x].char_indices().next_back() {
                char_to_delete = ch.to_string();
                char_start_byte = idx;
            }

            self.commit(
                LastActionType::Deletion,
                &ActionDiff {
                    cursor_start_x: self.cursor_x,
                    cursor_start_y: self.cursor_y,
                    cursor_end_x: char_start_byte,
                    cursor_end_y: self.cursor_y,
                    start_x: char_start_byte,
                    start_y: self.cursor_y,
                    end_x: self.cursor_x,
                    end_y: self.cursor_y,
                    new: vec![],
                    old: vec![char_to_delete],
                },
            );
        } else if self.cursor_y > 0 {
            self.commit(
                LastActionType::Deletion,
                &ActionDiff {
                    cursor_start_x: self.cursor_x,
                    cursor_start_y: self.cursor_y,
                    cursor_end_x: self.document.lines[self.cursor_y - 1].len(),
                    cursor_end_y: self.cursor_y - 1,
                    start_x: self.document.lines[self.cursor_y - 1].len(),
                    start_y: self.cursor_y - 1,
                    end_x: self.cursor_x,
                    end_y: self.cursor_y,
                    new: vec![],
                    old: vec!["".to_string(), "".to_string()],
                },
            );
        }
        Ok(())
    }

    pub fn delete_forward_char(&mut self) -> Result<()> {
        self.last_action_was_kill = false;
        // Ctrl-D
        let y = self.cursor_y;
        let x = self.cursor_x;
        let line_len = self.document.lines.get(y).map_or(0, |l| l.len());
        if x < line_len {
            let line = &self.document.lines[y];
            let mut char_to_delete = String::new();

            if let Some((_, ch)) = line[x..].char_indices().next() {
                char_to_delete = ch.to_string();
            }
            self.commit(
                LastActionType::Deletion,
                &ActionDiff {
                    cursor_start_x: self.cursor_x,
                    cursor_start_y: self.cursor_y,
                    cursor_end_x: self.cursor_x,
                    cursor_end_y: self.cursor_y,
                    start_x: self.cursor_x,
                    start_y: self.cursor_y,
                    end_x: self.cursor_x + char_to_delete.len(),
                    end_y: self.cursor_y,
                    new: vec![],
                    old: vec![char_to_delete],
                },
            );
        } else if y < self.document.lines.len() - 1 {
            self.commit(
                LastActionType::Deletion,
                &ActionDiff {
                    cursor_start_x: self.cursor_x,
                    cursor_start_y: self.cursor_y,
                    cursor_end_x: self.cursor_x,
                    cursor_end_y: self.cursor_y,
                    start_x: self.cursor_x,
                    start_y: self.cursor_y,
                    end_x: 0,
                    end_y: self.cursor_y + 1,
                    new: vec![],
                    old: vec!["".to_string(), "".to_string()],
                },
            );
        }
        Ok(())
    }

    fn get_indentation(&self) -> String {
        if self.cursor_y >= self.document.lines.len() {
            return String::new();
        }
        self.document.lines[self.cursor_y]
            .chars()
            .take_while(|&c| c.is_whitespace())
            .collect()
    }

    pub fn insert_newline(&mut self) -> Result<()> {
        self.last_action_was_kill = false;

        let y = self.cursor_y;
        let x = self.cursor_x;
        let current_line = self.document.lines[y].clone();

        // Check for command execution BEFORE committing the newline
        if x == current_line.len() && current_line.trim() == "/task" {
            self.mode = EditorMode::TaskSelection;
            self.find_unchecked_tasks();
            // Remove the "/task" command line itself
            self.commit(
                LastActionType::Other,
                &ActionDiff {
                    cursor_start_x: self.cursor_x,
                    cursor_start_y: self.cursor_y,
                    cursor_end_x: self.cursor_x,
                    cursor_end_y: self.cursor_y,
                    start_x: 0,
                    start_y: self.cursor_y,
                    end_x: current_line.len(),
                    end_y: self.cursor_y,
                    new: vec![],
                    old: vec![current_line.to_string()],
                },
            );
            return Ok(());
        }

        // Get indentation of the current line
        let indentation = self.get_indentation();
        let indentation_len = indentation.len();

        // If not a command, insert a regular newline
        self.commit(
            LastActionType::Newline,
            &ActionDiff {
                cursor_start_x: self.cursor_x,
                cursor_start_y: self.cursor_y,
                cursor_end_x: indentation_len,
                cursor_end_y: self.cursor_y + 1,
                start_x: self.cursor_x,
                start_y: self.cursor_y,
                end_x: indentation_len,
                end_y: self.cursor_y + 1,
                new: vec!["".to_string(), indentation],
                old: vec![],
            },
        );

        // Check for command execution
        if x == current_line.len() {
            match command::execute_command(&current_line) {
                command::CommandResult::Success {
                    new_line_content,
                    status_message,
                } => {
                    if let Some(new_content) = new_line_content {
                        self.commit(
                            LastActionType::Other,
                            &ActionDiff {
                                cursor_start_x: self.cursor_x,
                                cursor_start_y: self.cursor_y,
                                cursor_end_x: self.cursor_x,
                                cursor_end_y: self.cursor_y,
                                start_x: 0,
                                start_y: self.cursor_y - 1,
                                end_x: current_line.len(),
                                end_y: self.cursor_y - 1,
                                new: vec![new_content],
                                old: vec![current_line.to_string()],
                            },
                        );
                    }
                    self.status_message = status_message;
                }
                command::CommandResult::Error(message) => {
                    self.status_message = message.to_string();
                }
                command::CommandResult::NoCommand => {
                    // Do nothing, not a command
                }
            }
        }

        Ok(())
    }

    pub fn kill_line(&mut self) -> Result<()> {
        let y = self.cursor_y;
        let x = self.cursor_x;
        if y >= self.document.lines.len() {
            return Ok(());
        }

        let should_clear_kill_buffer = !self.last_action_was_kill;
        if should_clear_kill_buffer {
            self.kill_buffer.clear();
        }

        let current_line_len = self.document.lines[y].len();

        if x < current_line_len {
            // Case 1: Cursor is within the line (not at the very end)
            // Kill from cursor to end of line
            let current_line = self.document.lines[y].clone();
            let killed_text = current_line[x..].to_string();
            self.kill_buffer.push_str(&killed_text);
            self.commit(
                LastActionType::Deletion,
                &ActionDiff {
                    cursor_start_x: self.cursor_x,
                    cursor_start_y: self.cursor_y,
                    cursor_end_x: self.cursor_x,
                    cursor_end_y: self.cursor_y,
                    start_x: self.cursor_x,
                    start_y: self.cursor_y,
                    end_x: current_line_len,
                    end_y: self.cursor_y,
                    new: vec![],
                    old: vec![killed_text],
                },
            );
        } else {
            self.delete_forward_char()?;
            self.kill_buffer.push('\x0a');
        }

        self.set_clipboard(&self.kill_buffer.clone());

        self.last_action_was_kill = true;

        Ok(())
    }

    fn set_clipboard(&mut self, text: &str) {
        if !self.clipboard_enabled {
            return;
        }
        if let Ok(mut clipboard) = Clipboard::new() {
            if let Err(e) = clipboard.set_text(text.to_string()) {
                self.status_message = format!("Failed to set clipboard: {e}");
            }
        } else {
            self.status_message = "Failed to initialize clipboard.".to_string();
        }
    }

    pub fn yank(&mut self) -> Result<()> {
        if self.clipboard_enabled {
            if let Ok(mut clipboard) = Clipboard::new() {
                if let Ok(text) = clipboard.get_text() {
                    self.kill_buffer = text;
                }
            }
        }

        let text_to_yank = self.kill_buffer.clone();
        if text_to_yank.is_empty() {
            self.status_message = "Kill buffer is empty.".to_string();
            return Ok(());
        }

        let yank_lines: Vec<String> = text_to_yank.split('\x0a').map(|s| s.to_string()).collect();

        let line_count = yank_lines.len();
        let last_yank_line_count = yank_lines.last().unwrap().len();

        if line_count >= 2 {
            self.commit(
                LastActionType::Insertion,
                &ActionDiff {
                    cursor_start_x: self.cursor_x,
                    cursor_start_y: self.cursor_y,
                    cursor_end_x: last_yank_line_count,
                    cursor_end_y: self.cursor_y + line_count - 1,

                    start_x: self.cursor_x,
                    start_y: self.cursor_y,
                    end_x: last_yank_line_count,
                    end_y: self.cursor_y + line_count - 1,

                    new: yank_lines,
                    old: vec![],
                },
            );
        } else {
            self.commit(
                LastActionType::Insertion,
                &ActionDiff {
                    cursor_start_x: self.cursor_x,
                    cursor_start_y: self.cursor_y,
                    cursor_end_x: self.cursor_x + last_yank_line_count,
                    cursor_end_y: self.cursor_y,

                    start_x: self.cursor_x,
                    start_y: self.cursor_y,
                    end_x: self.cursor_x + last_yank_line_count,
                    end_y: self.cursor_y,

                    new: vec![text_to_yank.to_string()],
                    old: vec![],
                },
            );
        }

        self.last_action_was_kill = false;
        Ok(())
    }

    #[doc(hidden)]
    pub fn _set_clipboard_enabled_for_test(&mut self, enabled: bool) {
        self.clipboard_enabled = enabled;
    }

    pub fn hungry_delete(&mut self) -> Result<()> {
        let (x, y) = (self.cursor_x, self.cursor_y);
        if y >= self.document.lines.len() {
            return Ok(());
        }

        let current_line = &mut self.document.lines[y];

        if x == 0 {
            self.delete_char()?;
        } else {
            let start_delete_byte = find_word_boundary_left(current_line, x);

            // Need to clone the line content before modification for the Diff
            let line_content_before_delete = current_line.clone();
            let deleted_text = line_content_before_delete[start_delete_byte..x].to_string();
            self.commit(
                LastActionType::Deletion,
                &ActionDiff {
                    cursor_start_x: self.cursor_x,
                    cursor_start_y: self.cursor_y,
                    cursor_end_x: start_delete_byte,
                    cursor_end_y: self.cursor_y,

                    start_x: start_delete_byte,
                    start_y: self.cursor_y,
                    end_x: self.cursor_x,
                    end_y: self.cursor_y,

                    new: vec![],
                    old: vec![deleted_text],
                },
            );
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
        self.desired_cursor_x = self
            .scroll
            .get_display_width(&self.document.lines[y], self.cursor_x);
    }

    pub fn move_cursor_word_left(&mut self) -> Result<()> {
        self.last_action_was_kill = false;
        if self.cursor_x == 0 {
            if self.cursor_y > 0 {
                self.cursor_y -= 1;
                self.cursor_x = self.document.lines[self.cursor_y].len();
                self.desired_cursor_x = self
                    .scroll
                    .get_display_width(&self.document.lines[self.cursor_y], self.cursor_x);
            }
            return Ok(());
        }

        let line = &self.document.lines[self.cursor_y];
        let mut new_cursor_x = self.cursor_x;

        // 1. Skip whitespace to the left
        let mut boundary = new_cursor_x;
        for (idx, ch) in line[..new_cursor_x].char_indices().rev() {
            if get_char_type(ch) != CharType::Whitespace {
                break;
            }
            boundary = idx;
        }
        new_cursor_x = boundary;

        // 2. We are at the end of a word. Get its type.
        if new_cursor_x > 0 {
            let word_type = get_char_type(line[..new_cursor_x].chars().next_back().unwrap());
            // 3. Skip all chars of this type
            for (idx, ch) in line[..new_cursor_x].char_indices().rev() {
                if get_char_type(ch) != word_type {
                    break;
                }
                boundary = idx;
            }
            new_cursor_x = boundary;
        }

        self.cursor_x = new_cursor_x;
        self.desired_cursor_x = self.scroll.get_display_width(line, self.cursor_x);
        Ok(())
    }

    pub fn move_cursor_word_right(&mut self) -> Result<()> {
        self.last_action_was_kill = false;
        let current_line = &self.document.lines[self.cursor_y];
        let line_len = current_line.len();

        if self.cursor_x >= line_len {
            if self.cursor_y < self.document.lines.len() - 1 {
                self.cursor_y += 1;
                self.cursor_x = 0;
                self.desired_cursor_x = 0;
            }
            return Ok(());
        }

        let mut new_cursor_x = self.cursor_x;
        let mut iter = current_line[new_cursor_x..].char_indices().peekable();

        // 1. Skip whitespace
        while let Some((_, ch)) = iter.peek() {
            if get_char_type(*ch) == CharType::Whitespace {
                new_cursor_x += ch.len_utf8();
                iter.next();
            } else {
                break;
            }
        }

        // 2. We are at a word. Get its type.
        if let Some((_, first_word_char)) = iter.peek() {
            let word_type = get_char_type(*first_word_char);
            // 3. Skip all chars of this type
            while let Some((_, ch)) = iter.peek() {
                if get_char_type(*ch) == word_type {
                    new_cursor_x += ch.len_utf8();
                    iter.next();
                } else {
                    break;
                }
            }
        }

        self.cursor_x = new_cursor_x;
        self.desired_cursor_x = self.scroll.get_display_width(current_line, self.cursor_x);
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
        if let Some(file_path) = &self.document.filename {
            if let Ok(last_modified) = self.document.last_modified() {
                let cursor_pos = CursorPosition {
                    file_path: file_path.clone(),
                    last_modified,
                    cursor_x: self.cursor_x,
                    cursor_y: self.cursor_y,
                    scroll_row_offset: self.scroll.row_offset,
                    scroll_col_offset: self.scroll.col_offset,
                };
                debug!(
                    "Saving cursor position for {}: ({}, {}), scroll: ({}, {}), last_modified: {:?}",
                    file_path,
                    self.cursor_x,
                    self.cursor_y,
                    self.scroll.row_offset,
                    self.scroll.col_offset,
                    last_modified
                );
                if let Err(e) = persistence::save_cursor_position(cursor_pos) {
                    debug!("Failed to save cursor position: {e:?}");
                }
            } else {
                debug!(
                    "Could not get last modified date for {file_path}. Not saving cursor position."
                );
            }
        } else {
            debug!("No filename for current document. Not saving cursor position.");
        }
        self.should_quit = true;
        debug!("Editor quitting.");
        persistence::cleanup_old_cursor_position_files();
        Ok(())
    }

    pub fn cursor_pos(&self) -> (usize, usize) {
        (self.cursor_x, self.cursor_y)
    }

    pub fn set_cursor_pos(&mut self, x: usize, y: usize) {
        self.cursor_x = x;
        self.cursor_y = y;
        self.scroll
            .clamp_cursor_x(&mut self.cursor_x, &self.cursor_y, &self.document);
    }

    pub fn set_message(&mut self, message: &str) {
        self.status_message = message.to_string();
    }

    pub fn move_line_up(&mut self) {
        if self.cursor_y == 0 {
            self.status_message = "Cannot move line up further.".to_string();
            return;
        }
        let swapped_line0 = self.document.lines[self.cursor_y - 1].clone();
        let swapped_line1 = self.document.lines[self.cursor_y].clone();
        let current_cursor_x = self.cursor_x;

        // Delete 2 lines
        self.commit(
            LastActionType::LineMovement,
            &ActionDiff {
                cursor_start_x: self.cursor_x,
                cursor_start_y: self.cursor_y,
                cursor_end_x: 0,
                cursor_end_y: self.cursor_y - 1,

                start_x: 0,
                start_y: self.cursor_y - 1,
                end_x: self.document.lines[self.cursor_y].len(),
                end_y: self.cursor_y,

                new: vec![],
                old: vec![swapped_line0.clone(), swapped_line1.clone()],
            },
        );
        // Insert 2 lines
        self.commit(
            LastActionType::Ammend,
            &ActionDiff {
                cursor_start_x: self.cursor_x,
                cursor_start_y: self.cursor_y,
                cursor_end_x: current_cursor_x,
                cursor_end_y: self.cursor_y,

                start_x: 0,
                start_y: self.cursor_y,
                end_x: swapped_line0.len(),
                end_y: self.cursor_y + 1,

                new: vec![swapped_line1.clone(), swapped_line0.clone()],
                old: vec![],
            },
        );
        self.last_action_was_kill = false;
    }

    pub fn move_line_down(&mut self) {
        if self.cursor_y == self.document.lines.len() - 1 {
            self.status_message = "Cannot move line down further.".to_string();
            return;
        }

        let swapped_line0 = self.document.lines[self.cursor_y].clone();
        let swapped_line1 = self.document.lines[self.cursor_y + 1].clone();
        let current_cursor_x = self.cursor_x;
        // Delete 2 lines
        self.commit(
            LastActionType::LineMovement,
            &ActionDiff {
                cursor_start_x: self.cursor_x,
                cursor_start_y: self.cursor_y,
                cursor_end_x: 0,
                cursor_end_y: self.cursor_y,

                start_x: 0,
                start_y: self.cursor_y,
                end_x: self.document.lines[self.cursor_y + 1].len(),
                end_y: self.cursor_y + 1,

                new: vec![],
                old: vec![swapped_line0.clone(), swapped_line1.clone()],
            },
        );
        // Insert 2 lines
        self.commit(
            LastActionType::Ammend,
            &ActionDiff {
                cursor_start_x: self.cursor_x,
                cursor_start_y: self.cursor_y,
                cursor_end_x: current_cursor_x,
                cursor_end_y: self.cursor_y + 1,

                start_x: 0,
                start_y: self.cursor_y,
                end_x: swapped_line0.len(),
                end_y: self.cursor_y + 1,

                new: vec![swapped_line1.clone(), swapped_line0.clone()],
                old: vec![],
            },
        );
        self.last_action_was_kill = false;
    }

    pub fn scroll_page_down(&mut self) {
        self.scroll.scroll_page_down(
            &mut self.cursor_y,
            &mut self.cursor_x,
            &self.document,
            &mut self.last_action_was_kill,
        );
    }

    pub fn scroll_page_up(&mut self) {
        self.scroll.scroll_page_up(
            &mut self.cursor_y,
            &mut self.cursor_x,
            &self.document,
            &mut self.last_action_was_kill,
        );
    }

    pub fn go_to_start_of_file(&mut self) {
        self.scroll.go_to_start_of_file(
            &mut self.cursor_y,
            &mut self.cursor_x,
            &mut self.desired_cursor_x,
            &mut self.last_action_was_kill,
        );
    }

    pub fn go_to_end_of_file(&mut self) {
        self.scroll.go_to_end_of_file(
            &mut self.cursor_y,
            &mut self.cursor_x,
            &mut self.desired_cursor_x,
            &self.document,
            &mut self.last_action_was_kill,
        );
    }

    pub fn move_cursor_up(&mut self) {
        self.scroll.move_cursor_up(
            &mut self.cursor_y,
            &mut self.cursor_x,
            &mut self.desired_cursor_x,
            &self.document,
            &mut self.last_action_was_kill,
        );
    }

    pub fn move_cursor_down(&mut self) {
        self.scroll.move_cursor_down(
            &mut self.cursor_y,
            &mut self.cursor_x,
            &mut self.desired_cursor_x,
            &self.document,
            &mut self.last_action_was_kill,
        );
    }

    pub fn move_cursor_left(&mut self) {
        self.scroll.move_cursor_left(
            &mut self.cursor_y,
            &mut self.cursor_x,
            &mut self.desired_cursor_x,
            &self.document,
            &mut self.last_action_was_kill,
        );
    }

    pub fn move_cursor_right(&mut self) {
        self.scroll.move_cursor_right(
            &mut self.cursor_y,
            &mut self.cursor_x,
            &mut self.desired_cursor_x,
            &self.document,
            &mut self.last_action_was_kill,
        );
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
        let cursor_pos = self.cursor_pos();
        let (killed_text, action_diff_option) =
            self.selection.cut_selection(&self.document, cursor_pos)?;

        if let Some(action_diff) = action_diff_option {
            self.commit(LastActionType::Deletion, &action_diff);
        }

        self.kill_buffer = killed_text;
        self.set_clipboard(&self.kill_buffer.clone());
        self.status_message = "Selection cut to clipboard.".to_string();
        debug!("Selection cut. Kill buffer: '{}'", self.kill_buffer);

        Ok(())
    }

    pub fn copy_selection_action(&mut self) -> Result<()> {
        let cursor_pos = self.cursor_pos();
        self.kill_buffer = self.selection.copy_selection(&self.document, cursor_pos)?;
        self.set_clipboard(&self.kill_buffer.clone());
        self.status_message = "Selection copied to clipboard.".to_string();
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
            self.scroll.row_offset = self.cursor_y; // Scroll to make cursor at top
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
            self.scroll.row_offset = self.cursor_y; // Scroll to make cursor at top
        }
    }

    pub fn set_undo_debounce_threshold(&mut self, threshold_ms: u64) {
        self.undo_debounce_threshold = Duration::from_millis(threshold_ms);
    }

    pub fn set_no_exit_on_save(&mut self, value: bool) {
        self.no_exit_on_save = value;
    }

    // Method to calculate task UI height
    pub fn task_ui_height(&self) -> usize {
        (self.scroll.screen_rows as f32 * 0.4).round() as usize
    }

    pub fn enter_fuzzy_search_mode(&mut self) {
        self.mode = EditorMode::FuzzySearch;
        self.fuzzy_search.update_matches(&self.document);
    }

    pub fn handle_fuzzy_search_input(&mut self, key: pancurses::Input) {
        if !self.fuzzy_search.handle_input(
            key,
            &mut self.cursor_y,
            &mut self.cursor_x,
            &self.document,
        ) {
            self.mode = EditorMode::Normal;
            self.fuzzy_search.reset();
        }
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
enum CharType {
    Kanji,
    Hiragana,
    Katakana,
    Alphanumeric,
    Punctuation,
    Whitespace,
    Other,
}

fn get_char_type(ch: char) -> CharType {
    if ch.is_whitespace() {
        return CharType::Whitespace;
    }
    if ch == '。' || ch == '、' {
        return CharType::Punctuation;
    }
    // ASCII Alphanumeric
    if ch.is_ascii_alphanumeric() {
        return CharType::Alphanumeric;
    }
    // Hiragana
    if ('\u{3040}'..='\u{309F}').contains(&ch) {
        return CharType::Hiragana;
    }
    // Katakana
    if ('\u{30A0}'..='\u{30FF}').contains(&ch) {
        return CharType::Katakana;
    }
    // CJK Unified Ideographs (Kanji)
    if ('\u{4E00}'..='\u{9FFF}').contains(&ch) {
        return CharType::Kanji;
    }
    // Full-width digits
    if ('\u{FF10}'..='\u{FF19}').contains(&ch) {
        return CharType::Alphanumeric;
    }
    // Full-width uppercase
    if ('\u{FF21}'..='\u{FF3A}').contains(&ch) {
        return CharType::Alphanumeric;
    }
    // Full-width lowercase
    if ('\u{FF41}'..='\u{FF5A}').contains(&ch) {
        return CharType::Alphanumeric;
    }
    CharType::Other
}

fn find_word_boundary_left(line: &str, current_x: usize) -> usize {
    if current_x == 0 {
        return 0;
    }

    let mut boundary = current_x;

    // 1. Find char to the left and its type
    let (start_idx, start_char) = line[..boundary].char_indices().next_back().unwrap();
    let current_type = get_char_type(start_char);
    boundary = start_idx;

    // If it's NOT whitespace, it's a word. Find its beginning.
    if current_type != CharType::Whitespace {
        for (idx, ch) in line[..start_idx].char_indices().rev() {
            if get_char_type(ch) != current_type {
                break;
            }
            boundary = idx;
        }
    }

    // Now, `boundary` is at the beginning of the word/whitespace block.
    // Delete any preceding whitespace.
    let mut final_boundary = boundary;
    for (idx, ch) in line[..boundary].char_indices().rev() {
        if get_char_type(ch) == CharType::Whitespace {
            final_boundary = idx;
        } else {
            break;
        }
    }

    final_boundary
}
