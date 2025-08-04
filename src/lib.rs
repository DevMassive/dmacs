pub mod document;
pub mod editor;
pub mod error;
pub mod terminal;

pub enum Event {
    Key(pancurses::Input, bool), // Input, is_alt_pressed
    Resize,
    Quit,
    ClearMessage,
}
