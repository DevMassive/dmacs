use arboard::Clipboard;

pub struct ClipboardManager {
    pub kill_buffer: String,
    pub last_action_was_kill: bool,
    clipboard_enabled: bool,
}

impl ClipboardManager {
    pub fn new() -> Self {
        Self {
            kill_buffer: String::new(),
            last_action_was_kill: false,
            clipboard_enabled: true,
        }
    }

    pub fn copy_text(&mut self, text: &str) -> Result<(), String> {
        self.kill_buffer = text.to_string();
        self.set_system_clipboard(text)?;
        self.last_action_was_kill = false;
        Ok(())
    }

    pub fn kill_text(&mut self, text: &str) -> Result<(), String> {
        if self.last_action_was_kill {
            self.kill_buffer.push_str(text);
        } else {
            self.kill_buffer = text.to_string();
        }
        self.set_system_clipboard(&self.kill_buffer)?;
        self.last_action_was_kill = true;
        Ok(())
    }

    pub fn get_yank_text(&mut self) -> &str {
        if self.clipboard_enabled {
            if let Ok(mut clipboard) = Clipboard::new() {
                if let Ok(text) = clipboard.get_text() {
                    self.kill_buffer = text;
                }
            }
        }
        &self.kill_buffer
    }

    fn set_system_clipboard(&self, text: &str) -> Result<(), String> {
        if !self.clipboard_enabled {
            return Ok(());
        }
        if let Ok(mut clipboard) = Clipboard::new() {
            clipboard.set_text(text).map_err(|e| e.to_string())
        } else {
            Err("Failed to initialize clipboard.".to_string())
        }
    }

    #[doc(hidden)]
    pub fn _set_clipboard_enabled_for_test(&mut self, enabled: bool) {
        self.clipboard_enabled = enabled;
    }
}

impl Default for ClipboardManager {
    fn default() -> Self {
        Self::new()
    }
}
