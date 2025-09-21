pub mod backup;
pub mod config;
pub mod document;
pub mod editor;
pub mod error;
pub mod persistence;
pub mod terminal;

pub enum Event {
    Key(pancurses::Input, bool), // Input, is_alt_pressed
    Resize,
    Quit,
    ClearMessage,
}

use editor::Editor;
use error::Result;
use terminal::Terminal;

pub fn run_editor(
    terminal: &Terminal,
    filename: Option<String>,
    line: Option<usize>,
    column: Option<usize>,
    no_exit_on_save: bool,
    keymap: config::Keymap,
) -> Result<()> {
    let (screen_rows, screen_cols) = terminal.size();
    let mut editor = Editor::new(filename, line, column);
    editor.set_keymap(keymap);
    editor.set_no_exit_on_save(no_exit_on_save);
    editor.update_screen_size(screen_rows, screen_cols);

    loop {
        editor.update_screen_size(terminal.size().0, terminal.size().1);
        editor.draw(terminal.window());

        if let Some(event) = terminal.next_event()? {
            match event {
                Event::Key(key, is_alt_pressed) => {
                    editor.process_input(key, is_alt_pressed)?;
                    terminal::CTRL_C_COUNT.store(0, std::sync::atomic::Ordering::SeqCst);
                }
                Event::Resize => {
                    // Handled by update_screen_size at the beginning of the loop
                }
                Event::Quit => {
                    let current_ctrl_c_count =
                        terminal::CTRL_C_COUNT.load(std::sync::atomic::Ordering::SeqCst);
                    if current_ctrl_c_count == 1 {
                        editor.set_message("Press Ctrl+C again to quit.");
                        let tx_clone = terminal.get_tx_for_timeout();
                        std::thread::spawn(move || {
                            std::thread::sleep(std::time::Duration::from_secs(2));
                            if let Err(e) = tx_clone.send(Event::ClearMessage) {
                                eprintln!("Could not send clear message signal: {e}");
                            }
                        });
                    } else if current_ctrl_c_count >= 2 {
                        editor.should_quit = true;
                    }
                }
                Event::ClearMessage => {
                    editor.set_message("");
                }
            }
        }

        if editor.should_quit {
            break;
        }
    }

    Ok(())
}
