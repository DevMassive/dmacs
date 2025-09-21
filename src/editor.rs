use crate::document::{ActionDiff, Document};
use crate::editor::search::Search;
use crate::error::Result;
use crate::persistence::{self, CursorPosition};
use log::debug;

pub mod checkbox;
pub mod clipboard;
pub mod command;
pub mod comment;
pub mod indent;
pub mod input;
pub mod scroll;
pub mod search;
pub mod selection;
pub mod task;
pub mod ui;
pub mod undo;
use crate::editor::scroll::Scroll;
pub mod actions;
pub mod fuzzy_search;
use crate::config::Keymap;
use crate::editor::actions::Action;
use crate::editor::task::Task;
use crate::editor::undo::{LastActionType, UndoRedo};

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
    pub undo_redo: UndoRedo,
    pub clipboard: clipboard::Clipboard,
    pub is_alt_pressed: bool,
    pub search: Search,
    pub selection: selection::Selection,
    pub no_exit_on_save: bool,
    // New fields for task command
    pub mode: EditorMode,
    pub task: Task,
    pub fuzzy_search: fuzzy_search::FuzzySearch,
    pub keymap: Keymap,
}

impl Editor {
    pub fn new(
        filename: Option<String>,
        line: Option<usize>,
        column: Option<usize>,
    ) -> Self {
        let (document, restored_pos) = match filename {
            Some(fname) => {
                if let Ok(doc) = Document::open(&fname) {
                    let last_modified = doc.last_modified().ok();
                    let restored = if let Some(lm) = last_modified {
                        persistence::get_cursor_position(&fname, lm)
                    } else {
                        None
                    };
                    (doc, restored)
                } else {
                    let mut doc = Document::new_empty();
                    doc.filename = Some(fname);
                    (doc, None)
                }
            }
            None => (Document::default(), None),
        };

        let mut editor = Self {
            should_quit: false,
            document,
            cursor_x: 0,
            cursor_y: 0,
            desired_cursor_x: 0,
            status_message: "".to_string(),
            scroll: Scroll::new(),
            undo_redo: UndoRedo::new(),
            clipboard: clipboard::Clipboard::new(),
            is_alt_pressed: false,
            search: Search::new(),
            selection: selection::Selection::new(),
            no_exit_on_save: false,
            mode: EditorMode::Normal,
            task: Task::new(),
            fuzzy_search: fuzzy_search::FuzzySearch::new(),
            keymap: Keymap::default(),
        };

        if let Some((x, y, scroll_row, scroll_col)) = restored_pos {
            editor.cursor_x = x;
            editor.cursor_y = y;
            if y < editor.document.lines.len() {
                editor.desired_cursor_x = editor
                    .scroll
                    .get_display_width_from_bytes(&editor.document.lines[y], x);
            }
            editor.scroll = Scroll::new_with_offset(scroll_row, scroll_col);
        }

        if let Some(line) = line {
            let y = line.saturating_sub(1); // Convert 1-based to 0-based
            if y < editor.document.lines.len() {
                editor.cursor_y = y;

                let col = column.unwrap_or(1).saturating_sub(1); // 0-based char index
                let line_content = &editor.document.lines[y];

                let mut byte_offset = 0;
                let mut current_col = 0;
                for (i, c) in line_content.char_indices() {
                    if current_col == col {
                        byte_offset = i;
                        break;
                    }
                    current_col += 1;
                    byte_offset = i + c.len_utf8();
                }
                if current_col < col {
                    // if col is out of bounds
                    byte_offset = line_content.len();
                }

                editor.cursor_x = byte_offset;
                editor.desired_cursor_x = editor
                    .scroll
                    .get_display_width_from_bytes(line_content, byte_offset);
            } else {
                // If line is out of bounds, just go to the end of the file.
                let num_lines = editor.document.lines.len();
                if num_lines > 0 {
                    editor.cursor_y = num_lines - 1;
                    editor.cursor_x = editor.document.lines[num_lines - 1].len();
                }
            }
        }

        editor
    }

    pub fn execute_action(&mut self, action: Action) -> Result<()> {
        self.status_message.clear();
        match action {
            // File
            Action::Save => {
                self.document.save(None)?;
                self.status_message = "File saved!".to_string();
            }
            Action::Quit => {
                if self.no_exit_on_save {
                    self.save_document()?;
                    self.set_message("File saved. Editor will not exit.");
                } else {
                    self.quit()?;
                }
            }
            // Cursor
            Action::MoveUp => self.move_cursor_up(),
            Action::MoveDown => self.move_cursor_down(),
            Action::MoveLeft => self.move_cursor_left(),
            Action::MoveRight => self.move_cursor_right(),
            Action::GoToStartOfLine => self.go_to_start_of_line(),
            Action::GoToEndOfLine => self.go_to_end_of_line(),
            Action::MoveWordLeft => self.move_cursor_word_left()?,
            Action::MoveWordRight => self.move_cursor_word_right()?,
            Action::PageUp => self.scroll_page_up(),
            Action::PageDown => self.scroll_page_down(),
            Action::GoToStartOfFile => self.go_to_start_of_file(),
            Action::GoToEndOfFile => self.go_to_end_of_file(),
            Action::MoveToNextDelimiter => self.move_to_next_delimiter(),
            Action::MoveToPreviousDelimiter => self.move_to_previous_delimiter(),
            // Editing
            Action::InsertChar(c) => self.insert_text(&c.to_string())?,
            Action::InsertNewline => self.insert_newline()?,
            Action::DeleteChar => self.delete_char()?,
            Action::DeleteForwardChar => self.delete_forward_char()?,
            Action::DeleteWord => self.hungry_delete()?,
            Action::KillLine => {
                let _ = self.kill_line();
                self.clipboard.last_action_was_kill = true;
            }
            Action::Yank => self.yank()?,
            Action::Undo => self.undo(),
            Action::Redo => self.redo(),
            Action::Indent => self.indent_line()?,
            Action::Outdent => self.outdent_line()?,
            Action::ToggleComment => self.toggle_comment()?,
            Action::ToggleCheckbox => self.toggle_checkbox()?,
            // Selection
            Action::SetMarker => self.set_marker_action(),
            Action::ClearMarker => self.clear_marker_action(),
            Action::CutSelection => self.cut_selection_action()?,
            Action::CopySelection => self.copy_selection_action()?,
            // Search
            Action::EnterSearchMode => self.enter_search_mode(),
            Action::EnterFuzzySearchMode => self.enter_fuzzy_search_mode(),
            // Modes
            Action::EnterNormalMode => {
                if self.mode != EditorMode::Normal {
                    self.mode = EditorMode::Normal;
                }
            }
            // Misc
            Action::MoveLineUp => self.move_line_up(),
            Action::MoveLineDown => self.move_line_down(),
            _ => { /* NoOp, etc. */ }
        }
        self.scroll
            .clamp_cursor_x(&mut self.cursor_x, &self.cursor_y, &self.document);
        Ok(())
    }

    pub fn update_screen_size(&mut self, screen_rows: usize, screen_cols: usize) {
        self.scroll.update_screen_size(screen_rows, screen_cols);
    }

    pub fn undo(&mut self) {
        self.clipboard.last_action_was_kill = false;
        match self.undo_redo.undo(
            &mut self.document,
            &mut self.cursor_x,
            &mut self.cursor_y,
            &mut self.desired_cursor_x,
            &self.scroll,
        ) {
            Ok(_) => self.status_message = "Undo successful.".to_string(),
            Err(msg) => self.status_message = msg,
        }
    }

    pub fn redo(&mut self) {
        self.clipboard.last_action_was_kill = false;
        match self.undo_redo.redo(
            &mut self.document,
            &mut self.cursor_x,
            &mut self.cursor_y,
            &mut self.desired_cursor_x,
            &self.scroll,
        ) {
            Ok(_) => self.status_message = "Redo successful.".to_string(),
            Err(msg) => self.status_message = msg,
        }
    }

    pub(super) fn commit(&mut self, action_type: LastActionType, action_diff: &ActionDiff) {
        self.undo_redo.record_action(action_type, action_diff);
        let (new_x, new_y) = self.document.apply_action_diff(action_diff, false).unwrap();
        self.cursor_x = new_x;
        self.cursor_y = new_y;
        self.desired_cursor_x = self
            .scroll
            .get_display_width_from_bytes(&self.document.lines[self.cursor_y], self.cursor_x);
    }

    pub fn insert_text(&mut self, text: &str) -> Result<()> {
        // Special case for inserting " " at the end of a line followed by a space
        // Insert "-> "
        if text == " " {
            let y = self.cursor_y;
            let x = self.cursor_x;
            if x > 0
                && x == self.document.lines[y].len()
                && !self.document.lines[y][0..x].trim().is_empty()
            {
                let last_char = self.document.lines[y].chars().last().unwrap();
                if last_char == ' ' {
                    self.commit(
                        LastActionType::Insertion,
                        &ActionDiff {
                            cursor_start_x: self.cursor_x,
                            cursor_start_y: self.cursor_y,
                            cursor_end_x: self.cursor_x + 3,
                            cursor_end_y: self.cursor_y,
                            start_x: self.cursor_x,
                            start_y: self.cursor_y,
                            end_x: self.cursor_x + 3,
                            end_y: self.cursor_y,
                            new: vec!["-> ".to_string()],
                            old: vec![],
                        },
                    );
                    self.status_message = "->".to_string();
                    return Ok(());
                }
            }
        }
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
        self.clipboard.last_action_was_kill = false;
        // Backspace
        if self.cursor_x > 0 {
            let line = self.document.lines[self.cursor_y].clone();
            // Only apply if cursor is at the end of the line
            if self.cursor_x == line.len() {
                let trimmed_line = line.trim();
                let patterns = ["- [x]", "- [ ]", "-"];
                for pattern in &patterns {
                    if trimmed_line == *pattern {
                        let indentation_len = line.len() - line.trim_start().len();
                        let start_x = indentation_len;
                        let end_x = line.len();

                        self.commit(
                            LastActionType::Deletion,
                            &ActionDiff {
                                cursor_start_x: self.cursor_x,
                                cursor_start_y: self.cursor_y,
                                cursor_end_x: indentation_len,
                                cursor_end_y: self.cursor_y,
                                start_x,
                                start_y: self.cursor_y,
                                end_x,
                                end_y: self.cursor_y,
                                new: vec![],
                                old: vec![line[start_x..end_x].to_string()],
                            },
                        );
                        return Ok(());
                    }
                }
            }

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
        self.clipboard.last_action_was_kill = false;
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
        self.clipboard.last_action_was_kill = false;

        let y = self.cursor_y;
        let x = self.cursor_x;
        let current_line = self.document.lines[y].clone();

        // Delete empty list item
        if x == current_line.len() {
            let indentation_len = current_line.len() - current_line.trim_start().len();
            let content = &current_line[indentation_len..];

            let patterns = ["- [x] ", "- [ ] ", "- "];
            for pattern in &patterns {
                if content == *pattern {
                    self.commit(
                        LastActionType::Newline,
                        &ActionDiff {
                            cursor_start_x: self.cursor_x,
                            cursor_start_y: self.cursor_y,
                            cursor_end_x: 0,
                            cursor_end_y: self.cursor_y,
                            start_x: 0,
                            start_y: self.cursor_y,
                            end_x: self.document.lines[self.cursor_y].len(),
                            end_y: self.cursor_y,
                            new: vec![],
                            old: vec![current_line],
                        },
                    );
                    return Ok(());
                }
            }
        }

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
        let trimmed_line = current_line.trim_start();

        let mut new_line_prefix = indentation.clone();

        if (trimmed_line.starts_with("- [ ] ") || trimmed_line.starts_with("- [x] "))
            && self.cursor_x >= new_line_prefix.len() + 6
        {
            new_line_prefix.push_str("- [ ] ");
        } else if trimmed_line.starts_with("- ") && self.cursor_x >= new_line_prefix.len() + 2 {
            new_line_prefix.push_str("- ");
        }

        let indentation_len = new_line_prefix.len();

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
                                start_y: self.cursor_y,
                                end_x: current_line.len(),
                                end_y: self.cursor_y,
                                new: vec![],
                                old: vec![current_line.to_string()],
                            },
                        );
                        self.commit(
                            LastActionType::Ammend,
                            &ActionDiff {
                                cursor_start_x: self.cursor_x,
                                cursor_start_y: self.cursor_y,
                                cursor_end_x: 0,
                                cursor_end_y: self.cursor_y + 1,
                                start_x: 0,
                                start_y: self.cursor_y,
                                end_x: 0,
                                end_y: self.cursor_y + 1,
                                new: vec![new_content, "".to_string()],
                                old: vec![],
                            },
                        );
                    }
                    self.status_message = status_message;
                    return Ok(());
                }
                command::CommandResult::Error(message) => {
                    self.status_message = message.to_string();
                    return Ok(());
                }
                command::CommandResult::NoCommand => {
                    // Do nothing, not a command
                }
            }
        }

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
                new: vec!["".to_string(), new_line_prefix],
                old: vec![],
            },
        );

        Ok(())
    }

    pub fn kill_line(&mut self) -> Result<()> {
        let y = self.cursor_y;
        let x = self.cursor_x;
        if y >= self.document.lines.len() {
            return Ok(());
        }

        let should_clear_kill_buffer = !self.clipboard.last_action_was_kill;
        if should_clear_kill_buffer {
            self.clipboard.kill_buffer.clear();
        }

        let current_line_len = self.document.lines[y].len();

        if x < current_line_len {
            // Case 1: Cursor is within the line (not at the very end)
            // Kill from cursor to end of line
            let current_line = self.document.lines[y].clone();
            let killed_text = current_line[x..].to_string();
            self.clipboard.kill_buffer.push_str(&killed_text);
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
            self.clipboard.kill_buffer.push('\x0a');
        }

        self.set_clipboard(&self.clipboard.kill_buffer.clone());

        self.clipboard.last_action_was_kill = true;

        Ok(())
    }

    fn set_clipboard(&mut self, text: &str) {
        if let Err(e) = self.clipboard.set_clipboard(text) {
            self.status_message = format!("Failed to set clipboard: {e}");
        }
    }

    pub fn yank(&mut self) -> Result<()> {
        if let Some(text) = self.clipboard.get_clipboard_text() {
            self.clipboard.kill_buffer = text;
        }

        let text_to_yank = self.clipboard.kill_buffer.clone();
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

        self.clipboard.last_action_was_kill = false;
        Ok(())
    }

    #[doc(hidden)]
    pub fn _set_clipboard_enabled_for_test(&mut self, enabled: bool) {
        self.clipboard._set_clipboard_enabled_for_test(enabled);
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
        self.clipboard.last_action_was_kill = false;
        self.cursor_x = 0;
        self.desired_cursor_x = 0;
    }

    pub fn go_to_end_of_line(&mut self) {
        self.clipboard.last_action_was_kill = false;
        let y = self.cursor_y;
        self.cursor_x = self.document.lines[y].len();
        self.desired_cursor_x = self
            .scroll
            .get_display_width_from_bytes(&self.document.lines[y], self.cursor_x);
    }

    pub fn move_cursor_word_left(&mut self) -> Result<()> {
        self.clipboard.last_action_was_kill = false;
        if self.cursor_x == 0 {
            if self.cursor_y > 0 {
                self.cursor_y -= 1;
                self.cursor_x = self.document.lines[self.cursor_y].len();
                self.desired_cursor_x = self.scroll.get_display_width_from_bytes(
                    &self.document.lines[self.cursor_y],
                    self.cursor_x,
                );
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
        self.desired_cursor_x = self
            .scroll
            .get_display_width_from_bytes(line, self.cursor_x);
        Ok(())
    }

    pub fn move_cursor_word_right(&mut self) -> Result<()> {
        self.clipboard.last_action_was_kill = false;
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
        self.desired_cursor_x = self
            .scroll
            .get_display_width_from_bytes(current_line, self.cursor_x);
        Ok(())
    }

    pub fn save_document(&mut self) -> Result<()> {
        self.clipboard.last_action_was_kill = false;
        self.document.save(None)?;
        self.status_message = "File saved successfully.".to_string();
        debug!("Document saved.");
        Ok(())
    }

    pub fn quit(&mut self) -> Result<()> {
        self.clipboard.last_action_was_kill = false;
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
        self.clipboard.last_action_was_kill = false;
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
        self.clipboard.last_action_was_kill = false;
    }

    pub fn scroll_page_down(&mut self) {
        self.scroll.scroll_page_down(
            &mut self.cursor_y,
            &mut self.cursor_x,
            &self.document,
            &mut self.clipboard.last_action_was_kill,
        );
    }

    pub fn scroll_page_up(&mut self) {
        self.scroll.scroll_page_up(
            &mut self.cursor_y,
            &mut self.cursor_x,
            &self.document,
            &mut self.clipboard.last_action_was_kill,
        );
    }

    pub fn go_to_start_of_file(&mut self) {
        self.scroll.go_to_start_of_file(
            &mut self.cursor_y,
            &mut self.cursor_x,
            &mut self.desired_cursor_x,
            &mut self.clipboard.last_action_was_kill,
        );
    }

    pub fn go_to_end_of_file(&mut self) {
        self.scroll.go_to_end_of_file(
            &mut self.cursor_y,
            &mut self.cursor_x,
            &mut self.desired_cursor_x,
            &self.document,
            &mut self.clipboard.last_action_was_kill,
        );
    }

    pub fn move_cursor_up(&mut self) {
        self.scroll.move_cursor_up(
            &mut self.cursor_y,
            &mut self.cursor_x,
            &mut self.desired_cursor_x,
            &self.document,
            &mut self.clipboard.last_action_was_kill,
        );
    }

    pub fn move_cursor_down(&mut self) {
        self.scroll.move_cursor_down(
            &mut self.cursor_y,
            &mut self.cursor_x,
            &mut self.desired_cursor_x,
            &self.document,
            &mut self.clipboard.last_action_was_kill,
        );
    }

    pub fn move_cursor_left(&mut self) {
        self.scroll.move_cursor_left(
            &mut self.cursor_y,
            &mut self.cursor_x,
            &mut self.desired_cursor_x,
            &self.document,
            &mut self.clipboard.last_action_was_kill,
        );
    }

    pub fn move_cursor_right(&mut self) {
        self.scroll.move_cursor_right(
            &mut self.cursor_y,
            &mut self.cursor_x,
            &mut self.desired_cursor_x,
            &self.document,
            &mut self.clipboard.last_action_was_kill,
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

        self.clipboard.kill_buffer = killed_text;
        self.set_clipboard(&self.clipboard.kill_buffer.clone());
        self.status_message = "Selection cut to clipboard.".to_string();
        debug!(
            "Selection cut. Kill buffer: '{}'",
            self.clipboard.kill_buffer
        );

        Ok(())
    }

    pub fn copy_selection_action(&mut self) -> Result<()> {
        let cursor_pos = self.cursor_pos();
        self.clipboard.kill_buffer = self.selection.copy_selection(&self.document, cursor_pos)?;
        self.set_clipboard(&self.clipboard.kill_buffer.clone());
        self.status_message = "Selection copied to clipboard.".to_string();
        debug!(
            "Selection copied. Kill buffer: '{}'",
            self.clipboard.kill_buffer
        );
        Ok(())
    }

    pub fn move_to_next_delimiter(&mut self) {
        self.clipboard.last_action_was_kill = false;
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
        self.clipboard.last_action_was_kill = false;
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
        self.undo_redo.set_undo_debounce_threshold(threshold_ms);
    }

    pub fn set_no_exit_on_save(&mut self, value: bool) {
        self.no_exit_on_save = value;
    }

    pub fn set_keymap(&mut self, keymap: Keymap) {
        self.keymap = keymap;
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
