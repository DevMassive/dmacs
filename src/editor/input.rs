use crate::editor::Editor;
use crate::editor::EditorMode;
use crate::editor::actions::Action;
use crate::error::Result;
use log::debug;
use pancurses::Input;

fn key_to_string(key: Input, is_alt_pressed: bool) -> String {
    // Handle keys that should ignore the 'alt' modifier first.
    if let Input::Character(c) = key {
        // These are control characters, their meaning is fixed and not combined with Alt.
        if c.is_control() {
            return match c {
                '\x00' => "ctrl-space".to_string(),
                '\t' => "tab".to_string(),
                '\x0a' | '\x0d' => "enter".to_string(),
                '\x1b' => "esc".to_string(),
                '\x1f' => "ctrl-_".to_string(),
                '\x7f' | '\x08' => "backspace".to_string(),
                // Map other Ctrl+<char> combinations
                '\x01'..='\x1a' => format!("ctrl-{}", ((c as u8 - 1 + b'a') as char)),
                // Other control chars are unknown
                _ => "unknown".to_string(),
            };
        }
    }
    // Shift-Tab is also special and ignores alt
    if let Input::KeySTab = key {
        return "shift-tab".to_string();
    }
    if let Input::KeyBTab = key {
        return "shift-tab".to_string();
    }

    // For all other keys, the 'alt' modifier is relevant.
    let mut key_str = String::new();
    if is_alt_pressed {
        key_str.push_str("alt-");
    }

    match key {
        Input::Character(c) => {
            // It's a normal character, append it.
            // Control characters were already handled above.
            key_str.push(c);
        }
        Input::KeyUp => key_str.push_str("up"),
        Input::KeyDown => key_str.push_str("down"),
        Input::KeyLeft => key_str.push_str("left"),
        Input::KeyRight => key_str.push_str("right"),
        Input::KeyHome => key_str.push_str("home"),
        Input::KeyEnd => key_str.push_str("end"),
        Input::KeyBackspace => key_str.push_str("backspace"),
        Input::KeyDC => key_str.push_str("delete"),
        Input::KeyPPage => key_str.push_str("pageup"),
        Input::KeyNPage => key_str.push_str("pagedown"),
        // KeySTab and Character(control) are handled above
        _ => {
            if key_str.is_empty() {
                return "unknown".to_string();
            }
        }
    }

    key_str
}

impl Editor {
    pub fn process_input(&mut self, key: Input, is_alt_pressed: bool) -> Result<()> {
        debug!("Processing input: {key:?}, Alt pressed: {is_alt_pressed}");
        self.set_alt_pressed(is_alt_pressed);

        // Handle mode-specific inputs first
        if self.search.mode {
            self.handle_search_input(key);
            return Ok(());
        }
        if self.mode == EditorMode::TaskSelection {
            self.handle_task_selection_input(key);
            return Ok(());
        }
        if self.mode == EditorMode::FuzzySearch {
            self.handle_fuzzy_search_input(key);
            return Ok(());
        }

        // Normal mode input handling using keymap
        let key_string = key_to_string(key, is_alt_pressed);
        debug!("Key string: '{key_string}'");

        if let Some(action) = self.keymap.bindings.get(&key_string).cloned() {
            self.execute_action(action)?;
        } else if let Input::Character(c) = key {
            // If no specific action is bound, and it's a character, insert it.
            // We exclude control characters from being inserted directly.
            if !c.is_control() {
                self.execute_action(Action::InsertChar(c))?;
            }
        }
        // If no binding and not a character, do nothing.

        Ok(())
    }
}
