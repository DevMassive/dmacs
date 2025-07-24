use dmacs::Editor;
use pancurses::{endwin, initscr, noecho, curs_set};
use std::env;
use std::io;

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    let filename = args.get(1).cloned();

    let window = initscr();
    window.keypad(true);
    noecho();
    curs_set(1);

    let mut editor = Editor::new(filename);

    loop {
        editor.draw(&window);
        if let Some(key) = window.getch() {
            editor.handle_keypress(key);
        }
        if editor.should_quit {
            break;
        }
    }

    endwin();
    Ok(())
}