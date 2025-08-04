use pancurses::Input;

use crate::editor::state::Editor;

impl Editor {
    pub fn process_input(&mut self, key: Input, next_key: Option<Input>, third_key: Option<Input>) {
        if self.search.mode {
            self.handle_search_input(key);
            return;
        }

        match key {
            pancurses::Input::Character('\x1b') => {
                // Escape key, potential start of Alt/Option sequence
                if let Some(next_key_val) = next_key {
                    match next_key_val {
                        pancurses::Input::Character('v') => self.scroll_page_up(), // Alt/Option + V for page up (often sends ESC v)
                        pancurses::Input::Character('<') => self.go_to_start_of_file(), // Alt/Option + <
                        pancurses::Input::Character('>') => self.go_to_end_of_file(), // Alt/Option + >
                        pancurses::Input::Character('b') => self.move_cursor_word_left(), // Alt/Option + Left Arrow (often sends ESC b)
                        pancurses::Input::Character('f') => self.move_cursor_word_right(), // Alt/Option + Right Arrow (often sends ESC f)
                        pancurses::Input::Character('[') => {
                            if let Some(third_key_val) = third_key {
                                match third_key_val {
                                    pancurses::Input::Character('A') => self.move_line_up(), // Alt/Option + Up Arrow (often sends ESC [A)
                                    pancurses::Input::Character('B') => self.move_line_down(), // Alt/Option + Down Arrow (often sends ESC [B)
                                    _ => self.handle_keypress(pancurses::Input::Character('\x1b')), // Pass Escape if not a recognized sequence
                                }
                            } else {
                                self.handle_keypress(pancurses::Input::Character('\x1b')); // Pass Escape if no third key
                            }
                        }
                        pancurses::Input::Character('\x7f') | pancurses::Input::KeyBackspace => {
                            self.hungry_delete()
                        } // Alt/Option + Backspace
                        _ => self.handle_keypress(pancurses::Input::Character('\x1b')), // Pass Escape if not followed by Backspace
                    }
                } else {
                    self.handle_keypress(pancurses::Input::Character('\x1b')); // If no next_key, treat as plain Escape
                }
            }
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
