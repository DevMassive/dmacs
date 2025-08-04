use dmacs::editor::state::Editor;
use pancurses::{
    COLOR_BLACK, COLOR_WHITE, curs_set, endwin, init_pair, initscr, noecho, start_color,
    use_default_colors,
};
use std::env;
use std::io::{self, stdin};
use std::os::unix::io::AsRawFd;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

// Import necessary types and functions from the libc crate
use libc::{
    _POSIX_VDISABLE, TCSANOW, VDSUSP, VLNEXT, VREPRINT, VSTOP, tcgetattr, tcsetattr, termios,
};

static CTRL_C_COUNT: AtomicUsize = AtomicUsize::new(0);

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    let filename = args.get(1).cloned();

    let window = initscr();
    window.keypad(true);
    noecho();
    curs_set(1);
    window.nodelay(true); // Make getch() non-blocking

    // termios settings change starts here
    let stdin_fd = stdin().as_raw_fd();
    let mut termios_settings: termios = unsafe { std::mem::zeroed() };

    // Get current termios settings
    if unsafe { tcgetattr(stdin_fd, &mut termios_settings) } != 0 {
        return Err(io::Error::last_os_error());
    }

    // Disable dsusp character
    termios_settings.c_cc[VDSUSP] = _POSIX_VDISABLE;

    // Disable lnext character (Ctrl+V)
    termios_settings.c_cc[VLNEXT] = _POSIX_VDISABLE;

    // Disable stop character (Ctrl+S)
    termios_settings.c_cc[VSTOP] = _POSIX_VDISABLE;

    // Disable reprint character (Ctrl+R)
    termios_settings.c_cc[VREPRINT] = _POSIX_VDISABLE;
    if unsafe { tcsetattr(stdin_fd, TCSANOW, &termios_settings) } != 0 {
        return Err(io::Error::last_os_error());
    }

    if pancurses::has_colors() {
        start_color();
        use_default_colors();
        init_pair(1, COLOR_WHITE, -1);
        init_pair(2, COLOR_BLACK, COLOR_WHITE); // For highlighting
    }

    let (tx, rx) = mpsc::channel();
    let tx_clone_for_handler = tx.clone();
    ctrlc::set_handler(move || {
        CTRL_C_COUNT.fetch_add(1, Ordering::SeqCst); // Always increment
        tx_clone_for_handler
            .send(())
            .expect("Could not send signal on channel."); // Always send signal
    })
    .expect("Error setting Ctrl-C handler");

    let mut editor = Editor::new(filename);

    loop {
        // Update screen dimensions on each loop iteration
        let (screen_rows, screen_cols) = (window.get_max_y() as usize, window.get_max_x() as usize);
        editor.update_screen_size(screen_rows, screen_cols);

        editor.draw(&window);

        // Check for Ctrl+C signal
        if rx.try_recv().is_ok() {
            let current_ctrl_c_count = CTRL_C_COUNT.load(Ordering::SeqCst);
            if current_ctrl_c_count == 1 {
                editor.set_message("Press Ctrl+C again to quit.");
                let tx_clone = tx.clone();
                thread::spawn(move || {
                    thread::sleep(Duration::from_secs(2)); // 2 seconds to press again
                    // If after 2 seconds, the count is still 1, it means no second Ctrl+C was pressed.
                    // Reset the atomic counter and send a signal to clear the message.
                    if CTRL_C_COUNT
                        .compare_exchange(1, 0, Ordering::SeqCst, Ordering::SeqCst)
                        .is_ok()
                    {
                        tx_clone
                            .send(())
                            .expect("Could not send signal on channel."); // Dummy signal to clear message
                    }
                });
            } else if current_ctrl_c_count >= 2 {
                editor.should_quit = true;
            } else if current_ctrl_c_count == 0 {
                // This means the timeout thread reset it
                editor.set_message(""); // Clear the message
            }
        }

        if let Some(key) = window.getch() {
            let next_key = if key == pancurses::Input::Character('\x1b') {
                window.getch()
            } else {
                None
            };
            let third_key = if next_key == Some(pancurses::Input::Character('[')) {
                window.getch()
            } else {
                None
            };
            editor.process_input(key, next_key, third_key);
        }
        if editor.should_quit {
            break;
        }
    }

    endwin();
    Ok(())
}
