use dmacs::Event;
use dmacs::editor::state::Editor;
use dmacs::terminal::Terminal;
use std::env;
use std::io;

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    let filename = args.get(1).cloned();

    let terminal = Terminal::new()?;
    let (screen_rows, screen_cols) = terminal.size();
    let mut editor = Editor::new(filename);
    editor.update_screen_size(screen_rows, screen_cols);

    loop {
        editor.update_screen_size(terminal.size().0, terminal.size().1);
        editor.draw(terminal.window());

        match terminal.next_event() {
            Some(Event::Key(key, is_alt_pressed)) => {
                editor.process_input(key, is_alt_pressed);
            }
            Some(Event::Resize) => {
                // Handled by update_screen_size at the beginning of the loop
            }
            Some(Event::Quit) => {
                let current_ctrl_c_count =
                    dmacs::terminal::CTRL_C_COUNT.load(std::sync::atomic::Ordering::SeqCst);
                if current_ctrl_c_count == 1 {
                    editor.set_message("Press Ctrl+C again to quit.");
                    let tx_clone = terminal.get_tx_for_timeout();
                    std::thread::spawn(move || {
                        std::thread::sleep(std::time::Duration::from_secs(2));
                        if dmacs::terminal::CTRL_C_COUNT
                            .compare_exchange(
                                1,
                                0,
                                std::sync::atomic::Ordering::SeqCst,
                                std::sync::atomic::Ordering::SeqCst,
                            )
                            .is_ok()
                        {
                            tx_clone
                                .send(dmacs::Event::ClearMessage)
                                .expect("Could not send clear message signal.");
                        }
                    });
                } else if current_ctrl_c_count >= 2 {
                    editor.should_quit = true;
                }
            }
            Some(Event::ClearMessage) => {
                editor.set_message("");
            }
            None => {
                // No event, continue loop
            }
        }

        if editor.should_quit {
            break;
        }
    }

    Ok(())
}
