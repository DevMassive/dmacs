// src/editor/task.rs

use crate::document::ActionDiff;
use crate::editor::fuzzy_search::FuzzySearch;
use crate::editor::{Editor, EditorMode, LastActionType};
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use once_cell::sync::Lazy;
use pancurses::Input;

static MATCHER: Lazy<SkimMatcherV2> = Lazy::new(SkimMatcherV2::default);

#[derive(Debug)]
pub struct Task {
    pub tasks: Vec<(usize, String)>, // Store (original_line_index, content)
    pub all_tasks: Vec<(usize, String)>,
    pub selected_task_index: Option<usize>,
    pub task_display_offset: usize,
    pub fuzzy_search: FuzzySearch,
}

impl Default for Task {
    fn default() -> Self {
        Self::new()
    }
}

impl Task {
    pub fn new() -> Self {
        Self {
            tasks: Vec::new(),
            all_tasks: Vec::new(),
            selected_task_index: None,
            task_display_offset: 0,
            fuzzy_search: FuzzySearch::new(),
        }
    }
}

impl Editor {
    pub fn find_unchecked_tasks(&mut self) {
        self.task.tasks.clear();
        self.task.all_tasks.clear();
        self.task.selected_task_index = None;
        self.task.task_display_offset = 0;
        self.task.fuzzy_search.reset();

        let mut found_tasks = Vec::new();
        for (i, line) in self.document.lines.iter().enumerate() {
            if i > self.cursor_y && line.trim_start().starts_with("- [ ] ") {
                found_tasks.push((i, line.clone())); // Store (index, content)
            }
        }

        if !found_tasks.is_empty() {
            self.task.all_tasks = found_tasks.clone();
            self.task.tasks = found_tasks;
            self.task.selected_task_index = Some(0);
            self.set_message(&format!(
                "Found {} unchecked tasks. Use Up/Down to select, SPACE to move, ESC/ENTER to exit.",
                self.task.tasks.len()
            ));
        } else {
            self.set_message("No unchecked tasks found below current line.");
        }
    }

    fn update_task_matches(&mut self) {
        let query = &self.task.fuzzy_search.query;
        if query.is_empty() {
            self.task.tasks = self.task.all_tasks.clone();
        } else {
            self.task.tasks = self.task
                .all_tasks
                .iter()
                .filter_map(|(line_idx, line_content)| {
                    MATCHER
                        .fuzzy_match(line_content, query)
                        .map(|_score| (*line_idx, line_content.clone()))
                })
                .collect();
        }

        if self.task.tasks.is_empty() {
            self.task.selected_task_index = None;
        } else {
            self.task.selected_task_index = Some(0);
        }
        self.task.task_display_offset = 0;
    }

    pub fn handle_task_selection_input(&mut self, key: Input) {
        match key {
            Input::KeyUp => {
                let task_ui_height = self.task_ui_height();
                let task_list_visible_rows = task_ui_height.saturating_sub(1);
                if let Some(idx) = self.task.selected_task_index {
                    if idx > 0 {
                        self.task.selected_task_index = Some(idx - 1);
                        // Adjust scroll offset if selected task goes above visible area
                        if idx - 1 < self.task.task_display_offset {
                            self.task.task_display_offset = idx - 1;
                        }
                    } else if !self.task.tasks.is_empty() {
                        // Wrap around to the last task if at the top
                        self.task.selected_task_index = Some(self.task.tasks.len() - 1);
                        // Adjust scroll offset to show the last task
                        let max_offset =
                            self.task.tasks.len().saturating_sub(task_list_visible_rows);
                        self.task.task_display_offset = max_offset;
                    }
                }
            }
            Input::KeyDown => {
                let task_ui_height = self.task_ui_height();
                let task_list_visible_rows = task_ui_height.saturating_sub(1);
                if let Some(idx) = self.task.selected_task_index {
                    if idx < self.task.tasks.len() - 1 {
                        self.task.selected_task_index = Some(idx + 1);
                        // Adjust scroll offset if selected task goes below visible area
                        if idx + 1 >= self.task.task_display_offset + task_list_visible_rows {
                            self.task.task_display_offset = idx + 1 - task_list_visible_rows + 1;
                        }
                    } else if !self.task.tasks.is_empty() {
                        // Wrap around to the first task if at the bottom
                        self.task.selected_task_index = Some(0);
                        self.task.task_display_offset = 0;
                    }
                } else if !self.task.tasks.is_empty() {
                    self.task.selected_task_index = Some(0); // Select first if nothing selected
                    self.task.task_display_offset = 0;
                }
            }
            Input::Character(' ') => {
                // SPACE key
                if let Some(selected_idx) = self.task.selected_task_index {
                    if let Some((original_line_idx, task_content)) =
                        self.task.tasks.get(selected_idx).cloned()
                    {
                        let current_cursor_y = self.cursor_y;
                        let current_cursor_x = self.cursor_x;

                        // Remove the task from its original position
                        self.cursor_x = 0;
                        self.cursor_y = original_line_idx;
                        {
                            // kill line
                            let y = self.cursor_y;
                            let x = 0;
                            let task_line_len = self.document.lines[y].len();

                            let current_line = self.document.lines[y].clone();
                            let killed_text = current_line[x..].to_string();
                            self.kill_buffer.push_str(&killed_text);
                            self.commit(
                                LastActionType::Other,
                                &ActionDiff {
                                    cursor_start_x: current_cursor_x,
                                    cursor_start_y: current_cursor_y,
                                    cursor_end_x: self.cursor_x,
                                    cursor_end_y: self.cursor_y,
                                    start_x: self.cursor_x,
                                    start_y: self.cursor_y,
                                    end_x: task_line_len,
                                    end_y: self.cursor_y,
                                    new: vec![],
                                    old: vec![killed_text],
                                },
                            );
                        }

                        // backspace
                        self.commit(
                            LastActionType::Ammend,
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

                        // Insert the task at the current cursor position
                        self.cursor_y = current_cursor_y;
                        self.cursor_x = current_cursor_x;
                        self.commit(
                            LastActionType::Ammend,
                            &ActionDiff {
                                cursor_start_x: 0,
                                cursor_start_y: self.cursor_y,
                                cursor_end_x: 0,
                                cursor_end_y: self.cursor_y + 1,
                                start_x: 0,
                                start_y: self.cursor_y,
                                end_x: 0,
                                end_y: self.cursor_y + 1,
                                new: vec![task_content, "".to_string()],
                                old: vec![],
                            },
                        );

                        // Remove the task from the task.tasks list and update selected_task_index
                        self.task.tasks.remove(selected_idx);
                        self.task.all_tasks.retain(|(idx, _)| *idx != original_line_idx);

                        // Adjust original_line_index for subsequent tasks
                        for (line_idx, _) in self.task.tasks.iter_mut() {
                            if *line_idx < original_line_idx {
                                *line_idx += 1;
                            }
                        }
                        if self.task.tasks.is_empty() {
                            self.task.selected_task_index = None;
                            self.set_message("All tasks moved. Exiting task selection mode.");
                            self.mode = EditorMode::Normal; // Exit if no more tasks
                        } else {
                            if selected_idx >= self.task.tasks.len() {
                                self.task.selected_task_index = Some(self.task.tasks.len() - 1);
                            } else {
                                self.task.selected_task_index = Some(selected_idx);
                            }
                            self.set_message(&format!(
                                "Task moved. {} tasks remaining.",
                                self.task.tasks.len()
                            ));
                        }
                    }
                }
            }
            Input::Character('#') => {
                if let Some(selected_idx) = self.task.selected_task_index {
                    if let Some((original_line_idx, _)) = self.task.tasks.get(selected_idx).cloned()
                    {
                        self.commit(
                            LastActionType::ToggleComment,
                            &ActionDiff {
                                cursor_start_x: self.cursor_x,
                                cursor_start_y: self.cursor_y,
                                cursor_end_x: self.cursor_x,
                                cursor_end_y: self.cursor_y,
                                start_x: 0,
                                start_y: original_line_idx,
                                end_x: "# ".len(),
                                end_y: original_line_idx,
                                new: vec!["# ".to_string()],
                                old: vec![],
                            },
                        );

                        self.task.tasks.remove(selected_idx);
                        self.task.all_tasks.retain(|(idx, _)| *idx != original_line_idx);

                        if self.task.tasks.is_empty() {
                            self.task.selected_task_index = None;
                            self.set_message("All tasks handled. Exiting task selection mode.");
                            self.mode = EditorMode::Normal;
                        } else {
                            if selected_idx >= self.task.tasks.len() {
                                self.task.selected_task_index = Some(self.task.tasks.len() - 1);
                            }
                            self.set_message(&format!(
                                "Task commented out. {} tasks remaining.",
                                self.task.tasks.len()
                            ));
                        }
                    }
                }
            }
            Input::Character('\u{1b}')
            | Input::Character('\n')
            | Input::Character('\r')
            | Input::Character('\x07') => {
                // Escape or Enter or Ctrl+G to exit task selection mode
                self.mode = EditorMode::Normal;
                self.task.tasks.clear();
                self.task.all_tasks.clear();
                self.task.selected_task_index = None;
                self.task.task_display_offset = 0;
                self.task.fuzzy_search.reset();
                self.set_message("Exited task selection mode.");
            }
            Input::KeyBackspace
            | Input::KeyDC
            | Input::Character('\x7f')
            | Input::Character('\x08') => {
                if self.task.fuzzy_search.query.pop().is_some() {
                    self.update_task_matches();
                }
            }
            Input::Character(c) => {
                self.task.fuzzy_search.query.push(c);
                self.update_task_matches();
            }
            _ => {
                // Ignore other keys in task selection mode
                self.set_message("Task selection mode. Use Up/Down, SPACE, ESC/ENTER.");
            }
        }
    }
}
