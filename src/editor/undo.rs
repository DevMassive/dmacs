use crate::document::{ActionDiff, Document};
use crate::editor::scroll::Scroll;
use log::debug;
use std::time::{Duration, Instant};

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum LastActionType {
    None,
    Insertion,
    Deletion,
    Newline,
    LineMovement,
    Ammend,
    ToggleCheckbox,
    ToggleComment,
    Other,
}

pub struct UndoRedo {
    pub undo_stack: Vec<Vec<ActionDiff>>,
    pub redo_stack: Vec<Vec<ActionDiff>>,
    last_action_time: Option<Instant>,
    last_action_type: LastActionType,
    undo_debounce_threshold: Duration,
}

impl Default for UndoRedo {
    fn default() -> Self {
        Self::new()
    }
}

impl UndoRedo {
    pub fn new() -> Self {
        Self {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            last_action_time: None,
            last_action_type: LastActionType::None,
            undo_debounce_threshold: Duration::from_millis(500),
        }
    }

    pub fn set_undo_debounce_threshold(&mut self, threshold_ms: u64) {
        self.undo_debounce_threshold = Duration::from_millis(threshold_ms);
    }

    pub fn record_action(&mut self, action_type: LastActionType, action_diff: &ActionDiff) {
        self.save_state_for_undo(action_type);
        if let Some(last_transaction) = self.undo_stack.last_mut() {
            last_transaction.push(action_diff.clone());
        }
    }

    fn save_state_for_undo(&mut self, current_action_type: LastActionType) {
        let now = Instant::now();
        debug!(
            "save_state_for_undo: current_action_type={:?}, last_action_type={:?}, undo_debounce_threshold={:?}",
            current_action_type, self.last_action_type, self.undo_debounce_threshold
        );

        let should_start_new_group = if self.last_action_time.is_none() {
            debug!("save_state_for_undo: First action ever");
            true
        } else if current_action_type == LastActionType::Ammend {
            debug!("save_state_for_undo: Ammend");
            false
        } else if current_action_type == LastActionType::ToggleCheckbox {
            debug!("save_state_for_undo: ToggleCheckbox always starts a new group");
            true
        } else {
            let time_since_last_action = now.duration_since(self.last_action_time.unwrap());
            debug!("save_state_for_undo: time_since_last_action={time_since_last_action:?}");
            self.last_action_type != current_action_type
                || time_since_last_action >= self.undo_debounce_threshold
        };

        if should_start_new_group {
            debug!("save_state_for_undo: Pushing new undo group");
            self.undo_stack.push(Vec::new());
            self.redo_stack.clear();
        }
        self.last_action_time = Some(now);
        if current_action_type != LastActionType::Ammend {
            self.last_action_type = current_action_type;
        }
    }

    pub fn undo(
        &mut self,
        document: &mut Document,
        cursor_x: &mut usize,
        cursor_y: &mut usize,
        desired_cursor_x: &mut usize,
        scroll: &Scroll,
    ) -> Result<(), String> {
        debug!(
            "Undo called. Current undo_stack length: {}. Current document: {:?}",
            self.undo_stack.len(),
            document.lines
        );
        if let Some(mut actions_to_undo) = self.undo_stack.pop() {
            let mut actions_for_redo = Vec::new();
            let mut current_cursor_x = *cursor_x;
            let mut current_cursor_y = *cursor_y;

            actions_to_undo.reverse();
            for action_diff in actions_to_undo.iter() {
                match document.apply_action_diff(action_diff, true) {
                    Ok((new_x, new_y)) => {
                        current_cursor_x = new_x;
                        current_cursor_y = new_y;
                        actions_for_redo.push(action_diff.clone());
                    }
                    Err(e) => {
                        debug!("Undo failed: {e:?}");
                        self.undo_stack.push(actions_to_undo);
                        return Err(format!("Undo failed: {e:?}"));
                    }
                }
            }
            actions_for_redo.reverse();
            self.redo_stack.push(actions_for_redo);

            *cursor_x = current_cursor_x;
            *cursor_y = current_cursor_y;
            *desired_cursor_x =
                scroll.get_display_width_from_bytes(&document.lines[*cursor_y], *cursor_x);
            debug!("Document after undo: {:?}", document.lines);
            Ok(())
        } else {
            debug!("Undo stack is empty. Nothing to undo.");
            Err("Nothing to undo.".to_string())
        }
    }

    pub fn redo(
        &mut self,
        document: &mut Document,
        cursor_x: &mut usize,
        cursor_y: &mut usize,
        desired_cursor_x: &mut usize,
        scroll: &Scroll,
    ) -> Result<(), String> {
        debug!(
            "Redo called. Current redo_stack length: {}. Current document: {:?}",
            self.redo_stack.len(),
            document.lines
        );
        if let Some(actions_to_redo) = self.redo_stack.pop() {
            let mut actions_for_undo = Vec::new();
            let mut current_cursor_x = *cursor_x;
            let mut current_cursor_y = *cursor_y;

            for action_diff in actions_to_redo.iter() {
                match document.apply_action_diff(action_diff, false) {
                    Ok((new_x, new_y)) => {
                        current_cursor_x = new_x;
                        current_cursor_y = new_y;
                        actions_for_undo.push(action_diff.clone());
                    }
                    Err(e) => {
                        debug!("Redo failed: {e:?}");
                        self.redo_stack.push(actions_to_redo);
                        return Err(format!("Redo failed: {e:?}"));
                    }
                }
            }
            self.undo_stack.push(actions_for_undo);

            *cursor_x = current_cursor_x;
            *cursor_y = current_cursor_y;
            *desired_cursor_x =
                scroll.get_display_width_from_bytes(&document.lines[*cursor_y], *cursor_x);
            debug!("Document after redo: {:?}", document.lines);
            Ok(())
        } else {
            debug!("Redo stack is empty. Nothing to redo.");
            Err("Nothing to redo.".to_string())
        }
    }
}
