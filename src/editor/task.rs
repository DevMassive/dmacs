// src/editor/task.rs

#[derive(PartialEq, Debug)]
pub struct Task {
    pub tasks: Vec<(usize, String)>, // Store (original_line_index, content)
    pub selected_task_index: Option<usize>,
    pub task_display_offset: usize,
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
            selected_task_index: None,
            task_display_offset: 0,
        }
    }
}
