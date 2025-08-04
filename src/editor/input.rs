use pancurses::Input;

use crate::editor::state::Editor;

impl Editor {
    pub fn process_input(&mut self, key: Input, is_alt_pressed: bool) {
        self.set_alt_pressed(is_alt_pressed);

        if self.search.mode {
            self.handle_search_input(key);
            return;
        }

        match key {
            // Alt/Option + V for page up (often sends ESC v)
            Input::Character('v') if is_alt_pressed => self.scroll_page_up(),
            // Alt/Option + <
            Input::Character('<') if is_alt_pressed => self.go_to_start_of_file(),
            // Alt/Option + >
            Input::Character('>') if is_alt_pressed => self.go_to_end_of_file(),
            // Alt/Option + Left Arrow (often sends ESC b)
            Input::Character('b') if is_alt_pressed => self.move_cursor_word_left(),
            // Alt/Option + Right Arrow (often sends ESC f)
            Input::Character('f') if is_alt_pressed => self.move_cursor_word_right(),
            // Alt/Option + Up Arrow (often sends ESC [A)
            Input::KeyUp if is_alt_pressed => self.move_line_up(),
            // Alt/Option + Down Arrow (often sends ESC [B)
            Input::KeyDown if is_alt_pressed => self.move_line_down(),
            // Alt/Option + Backspace
            Input::KeyBackspace if is_alt_pressed => self.hungry_delete(),
            _ => self.handle_keypress(key),
        }
    }

    pub fn handle_keypress(&mut self, key: Input) {
        self.status_message.clear();
        match key {
            Input::Character(c) => match c {
                '\x18' => self.quit(),
                '\x13' => self.enter_search_mode(), // Ctrl + S
                '\x01' => self.go_to_start_of_line(),
                '\x05' => self.go_to_end_of_line(),
                '\x04' => self.delete_forward_char(),
                '\x0b' => {
                    self.kill_line();
                    self.last_action_was_kill = true;
                }
                '\x19' => self.yank(),                 // Ctrl + Y
                '\x16' => self.scroll_page_down(),     // Ctrl + V
                '\x7f' | '\x08' => self.delete_char(), // Backspace
                '\x0a' | '\x0d' => self.insert_newline(),
                '\x00' => {}
                '\x02' => self.move_cursor_word_left(), // Ctrl + B
                '\x06' => self.move_cursor_word_right(), // Ctrl + F
                '\x1f' => self.undo(),                  // Ctrl + _ for undo
                _ => self.insert_char(c),
            },
            Input::KeyBackspace => self.delete_char(),
            Input::KeyUp => self.move_cursor_up(),
            Input::KeyDown => self.move_cursor_down(),
            Input::KeyLeft => self.move_cursor_left(),
            Input::KeyRight => self.move_cursor_right(),
            _ => {}
        }
        self.clamp_cursor_x();
    }
}
