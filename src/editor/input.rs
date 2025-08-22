use log::debug;
use pancurses::Input;

use crate::editor::Editor;
use crate::error::Result;

impl Editor {
    pub fn process_input(&mut self, key: Input, is_alt_pressed: bool) -> Result<()> {
        debug!("Processing input: {key:?}, Alt pressed: {is_alt_pressed}");
        self.set_alt_pressed(is_alt_pressed);

        if self.search.mode {
            self.handle_search_input(key);
            return Ok(());
        }

        match key {
            // Alt/Option + V for page up (often sends ESC v)
            Input::Character('v') if is_alt_pressed => self.scroll_page_up(),
            // Alt/Option + <
            Input::Character('<') if is_alt_pressed => self.go_to_start_of_file(),
            // Alt/Option + >
            Input::Character('>') if is_alt_pressed => self.go_to_end_of_file(),
            // Alt/Option + Left Arrow (often sends ESC b)
            Input::Character('b') if is_alt_pressed => self.move_cursor_word_left()?,
            // Alt/Option + Right Arrow (often sends ESC f)
            Input::Character('f') if is_alt_pressed => self.move_cursor_word_right()?,
            // Alt/Option + Up Arrow (often sends ESC [A)
            Input::KeyUp if is_alt_pressed => self.move_line_up(),
            // Alt/Option + Down Arrow (often sends ESC [B)
            Input::KeyDown if is_alt_pressed => self.move_line_down(),
            // Alt/Option + Backspace
            Input::KeyBackspace if is_alt_pressed => self.hungry_delete()?,
            Input::Character('w') if is_alt_pressed => self.copy_selection_action()?, // Option-W
            Input::Character('_') if is_alt_pressed => self.redo(), // Alt + _ for redo
            _ => self.handle_keypress(key)?,
        }
        Ok(())
    }

    fn handle_keypress(&mut self, key: Input) -> Result<()> {
        self.status_message.clear();
        match key {
            Input::Character(c) => match c {
                '\x18' => self.quit()?,
                '\x13' => self.enter_search_mode(), // Ctrl + S
                '\x01' => self.go_to_start_of_line(),
                '\x05' => self.go_to_end_of_line(),
                '\x04' => self.delete_forward_char()?,
                '\x0b' => {
                    let _ = self.kill_line();
                    self.last_action_was_kill = true;
                }
                '\x19' => self.yank()?,                      // Ctrl + Y
                '\x16' => self.scroll_page_down(),           // Ctrl + V
                '\x0e' => self.move_to_next_delimiter(),     // Ctrl + N
                '\x10' => self.move_to_previous_delimiter(), // Ctrl + P
                '\x7f' | '\x08' => self.delete_char()?,      // Backspace
                '\x0a' | '\x0d' => self.insert_newline()?,
                '\x02' => self.move_cursor_word_left()?, // Ctrl + B
                '\x06' => self.move_cursor_word_right()?, // Ctrl + F
                '\x14' => self.toggle_checkbox()?,             // Ctrl + T
                '\x1f' => self.undo(),                   // Ctrl + _ for undo
                '\x03' | // Ctrl+C
                '\x0c' | // Ctrl+L
                '\x0f' | // Ctrl+O
                '\x11' | // Ctrl+Q
                '\x12' | // Ctrl+R
                '\x15' | // Ctrl+U
                '\x17' => self.cut_selection_action()?, // Ctrl+W
                '\x00' => self.set_marker_action(), // Ctrl+Space
                '\x07' => self.clear_marker_action(), // Ctrl+G
                '\x1a' | // Ctrl+Z
                '\x1b' | // Ctrl+[ (ESC)
                '\x1c' | // Ctrl+\
                '\x1d' | // Ctrl+] 
                '\x1e' => {}, // Ctrl+^
                _ => self.insert_text(&c.to_string())?,
            },
            Input::KeyBackspace => self.delete_char()?,
            Input::KeyUp => self.move_cursor_up(),
            Input::KeyDown => self.move_cursor_down(),
            Input::KeyLeft => self.move_cursor_left(),
            Input::KeyRight => self.move_cursor_right(),
            _ => {}
        }
        self.clamp_cursor_x();
        Ok(())
    }
}
