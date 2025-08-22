use pancurses::{
    COLOR_BLACK, COLOR_WHITE, Input, Window, curs_set, endwin, init_pair, initscr, noecho,
    start_color, use_default_colors,
};
use std::io::{self, stdin};
use std::os::unix::io::AsRawFd;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::{self, Receiver};

use crate::Event;
use crate::error::{DmacsError, Result};

// Import necessary types and functions from the libc crate
use libc::{
    _POSIX_VDISABLE, TCSANOW, VDSUSP, VLNEXT, VREPRINT, VSTATUS, VSTOP, tcgetattr, tcsetattr,
    termios,
};

pub static CTRL_C_COUNT: AtomicUsize = AtomicUsize::new(0);

pub struct Terminal {
    window: Window,
    original_termios: termios,
    event_rx: Receiver<Event>,
    event_tx: mpsc::Sender<Event>,
}

impl Terminal {
    pub fn new() -> Result<Self> {
        let window = initscr();
        window.keypad(true);
        noecho();
        curs_set(1);
        window.nodelay(true); // Make getch() non-blocking

        // termios settings change starts here
        let stdin_fd = stdin().as_raw_fd();
        let mut termios_settings: termios = unsafe { std::mem::zeroed() };
        let mut original_termios: termios = unsafe { std::mem::zeroed() };

        // Get current termios settings
        if unsafe { tcgetattr(stdin_fd, &mut termios_settings) } != 0 {
            return Err(DmacsError::Io(io::Error::last_os_error()));
        }
        original_termios.clone_from(&termios_settings);

        // Disable dsusp character
        termios_settings.c_cc[VDSUSP] = _POSIX_VDISABLE;

        // Disable lnext character (Ctrl+V)
        termios_settings.c_cc[VLNEXT] = _POSIX_VDISABLE;

        // Disable stop character (Ctrl+S)
        termios_settings.c_cc[VSTOP] = _POSIX_VDISABLE;

        // Disable reprint character (Ctrl+R)
        termios_settings.c_cc[VREPRINT] = _POSIX_VDISABLE;

        // Disable status character (Ctrl+T)
        termios_settings.c_cc[VSTATUS] = _POSIX_VDISABLE;
        if unsafe { tcsetattr(stdin_fd, TCSANOW, &termios_settings) } != 0 {
            return Err(DmacsError::Io(io::Error::last_os_error()));
        }

        if pancurses::has_colors() {
            start_color();
            use_default_colors();
            init_pair(1, COLOR_WHITE, -1);
            init_pair(2, COLOR_BLACK, COLOR_WHITE); // For highlighting
        }

        let (tx, rx) = mpsc::channel();
        let tx_clone_for_handler = tx.clone();

        // Ctrl+C handler
        ctrlc::set_handler(move || {
            let _current_count = CTRL_C_COUNT.fetch_add(1, Ordering::SeqCst) + 1;
            if let Err(e) = tx_clone_for_handler.send(Event::Quit) {
                // Log the error or handle it appropriately, but don't return a Result
                eprintln!("Could not send signal on channel: {e}");
            }
        })
        .map_err(|e| DmacsError::Terminal(format!("Error setting Ctrl-C handler: {e}")))?;

        Ok(Self {
            window,
            original_termios,
            event_rx: rx,
            event_tx: tx,
        })
    }

    pub fn window(&self) -> &Window {
        &self.window
    }

    pub fn size(&self) -> (usize, usize) {
        (
            self.window.get_max_y() as usize,
            self.window.get_max_x() as usize,
        )
    }

    pub fn get_tx_for_timeout(&self) -> std::sync::mpsc::Sender<Event> {
        self.event_tx.clone()
    }

    pub fn next_event(&self) -> Result<Option<Event>> {
        // Try to receive an event from the channel first
        match self.event_rx.try_recv() {
            Ok(event) => return Ok(Some(event)),
            Err(mpsc::TryRecvError::Empty) => {}
            Err(mpsc::TryRecvError::Disconnected) => {
                return Err(DmacsError::Terminal(
                    "Event channel disconnected".to_string(),
                ));
            }
        }

        // If no channel event, check for key input
        if let Some(key) = self.window.getch() {
            let mut is_alt_pressed = false;
            let processed_key = match key {
                Input::Character('\x1b') => {
                    // This is an escape character. Could be a standalone ESC, or part of an Alt/Meta sequence, or an arrow key.
                    let next_key = self.window.getch();

                    match next_key {
                        Some(Input::Character('[')) => {
                            // This could be an arrow key sequence (e.g., ESC [ A for Up Arrow)
                            let third_key = self.window.getch();
                            match third_key {
                                Some(Input::Character('A')) => {
                                    is_alt_pressed = true;
                                    Input::KeyUp
                                }
                                Some(Input::Character('B')) => {
                                    is_alt_pressed = true;
                                    Input::KeyDown
                                }
                                _ => Input::Character('\x1b'), // Fallback if not an arrow key sequence
                            }
                        }
                        Some(Input::KeyLeft) => {
                            is_alt_pressed = true;
                            Input::KeyLeft
                        }
                        Some(Input::KeyRight) => {
                            is_alt_pressed = true;
                            Input::KeyRight
                        }
                        Some(Input::KeyUp) => {
                            is_alt_pressed = true;
                            Input::KeyUp
                        }
                        Some(Input::KeyDown) => {
                            is_alt_pressed = true;
                            Input::KeyDown
                        }
                        Some(Input::Character('\x7f') | Input::KeyBackspace) => {
                            is_alt_pressed = true;
                            Input::KeyBackspace
                        }
                        Some(Input::Character(c)) => {
                            // If the next character is a printable character, it's likely an Alt/Meta sequence.
                            is_alt_pressed = true;
                            Input::Character(c)
                        }
                        _ => Input::Character('\x1b'), // Just an escape key
                    }
                }
                Input::KeyResize => {
                    return Ok(Some(Event::Resize));
                }
                _ => key,
            };
            return Ok(Some(Event::Key(processed_key, is_alt_pressed)));
        }
        Ok(None)
    }
}

impl Drop for Terminal {
    fn drop(&mut self) {
        let stdin_fd = stdin().as_raw_fd();
        if unsafe { tcsetattr(stdin_fd, TCSANOW, &self.original_termios) } != 0 {
            eprintln!(
                "Error restoring terminal settings: {}",
                DmacsError::Io(io::Error::last_os_error())
            );
        }
        endwin();
    }
}
