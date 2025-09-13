use arboard;

pub struct Clipboard {
    pub kill_buffer: String,
    pub last_action_was_kill: bool,
    clipboard_enabled: bool,
}

impl Default for Clipboard {
    fn default() -> Self {
        Self::new()
    }
}

impl Clipboard {
    pub fn new() -> Self {
        Self {
            kill_buffer: String::new(),
            last_action_was_kill: false,
            clipboard_enabled: true,
        }
    }

    pub fn set_clipboard(&self, text: &str) -> std::result::Result<(), arboard::Error> {
        if !self.clipboard_enabled {
            return Ok(());
        }
        if let Ok(mut clipboard) = arboard::Clipboard::new() {
            clipboard.set_text(text.to_string())
        } else {
            log::debug!("Failed to initialize clipboard.");
            Ok(())
        }
    }

    pub fn get_clipboard_text(&self) -> Option<String> {
        if !self.clipboard_enabled {
            return None;
        }
        if let Ok(mut clipboard) = arboard::Clipboard::new() {
            clipboard.get_text().ok()
        } else {
            None
        }
    }

    #[doc(hidden)]
    pub fn _set_clipboard_enabled_for_test(&mut self, enabled: bool) {
        self.clipboard_enabled = enabled;
    }
}
