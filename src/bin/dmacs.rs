use dmacs::Editor;
use std::env;
use std::fs::OpenOptions;
use std::io;

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    let filename = args.get(1).cloned();
    let tty = OpenOptions::new().read(true).write(true).open("/dev/tty")?;
    Editor::new(filename, tty).run()
}
