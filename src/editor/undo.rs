use crate::document::ActionDiff;
use crate::editor::LastActionType;
use std::time::{Duration, Instant};

const UNDO_DEBOUNCE_THRESHOLD: Duration = Duration::from_millis(500);

pub struct UndoManager {
    pub undo_stack: Vec<Vec<ActionDiff>>,
    pub redo_stack: Vec<Vec<ActionDiff>>,
    last_action_time: Option<Instant>,
    last_action_type: LastActionType,
    undo_debounce_threshold: Duration,
}

impl Default for UndoManager {
    fn default() -> Self {
        Self::new()
    }
}

impl UndoManager {
    pub fn new() -> Self {
        Self {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            last_action_time: None,
            last_action_type: LastActionType::None,
            undo_debounce_threshold: UNDO_DEBOUNCE_THRESHOLD,
        }
    }

    pub fn set_undo_debounce_threshold(&mut self, threshold_ms: u64) {
        self.undo_debounce_threshold = Duration::from_millis(threshold_ms);
    }

    pub fn save_state_for_undo(&mut self, current_action_type: LastActionType) {
        let now = Instant::now();
        let should_start_new_group = if self.last_action_time.is_none() {
            true
        } else if current_action_type == LastActionType::Ammend {
            false
        } else if current_action_type == LastActionType::ToggleCheckbox {
            true
        } else {
            let time_since_last_action = now.duration_since(self.last_action_time.unwrap());
            self.last_action_type != current_action_type
                || time_since_last_action >= self.undo_debounce_threshold
        };

        if should_start_new_group {
            self.undo_stack.push(Vec::new());
            self.redo_stack.clear();
        }
        self.last_action_time = Some(now);
        if current_action_type != LastActionType::Ammend {
            self.last_action_type = current_action_type;
        }
    }

    pub fn add_to_undo_group(&mut self, action_diff: &ActionDiff) {
        if let Some(last_transaction) = self.undo_stack.last_mut() {
            last_transaction.push(action_diff.clone());
        }
    }
}
