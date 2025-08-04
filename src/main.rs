use dmacs::Event;
use dmacs::editor::Editor;
use dmacs::error::Result;
use dmacs::terminal::Terminal;
use std::env;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    let filename = args.get(1).cloned();

    let terminal = Terminal::new()?;
    let (screen_rows, screen_cols) = terminal.size();
    let mut editor = Editor::new(filename);
    editor.update_screen_size(screen_rows, screen_cols);

    loop {
        editor.update_screen_size(terminal.size().0, terminal.size().1);
        editor.draw(terminal.window());

        if let Some(event) = terminal.next_event()? {
            match event {
                Event::Key(key, is_alt_pressed) => {
                    editor.process_input(key, is_alt_pressed)?;
                }
                Event::Resize => {
                    // Handled by update_screen_size at the beginning of the loop
                }
                Event::Quit => {
                    let current_ctrl_c_count =
                        dmacs::terminal::CTRL_C_COUNT.load(std::sync::atomic::Ordering::SeqCst);
                    if current_ctrl_c_count == 1 {
                        editor.set_message("Press Ctrl+C again to quit.");
                        let tx_clone = terminal.get_tx_for_timeout();
                        std::thread::spawn(move || {
                            if let Err(e) = tx_clone.send(dmacs::Event::ClearMessage) {
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
