use dmacs::Editor;
use std::env;
use std::io;

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    let filename = args.get(1).cloned();

    let mut editor = Editor::new(filename)?;
    editor.run()
}
