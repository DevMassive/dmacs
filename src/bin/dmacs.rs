use dmacs::Editor;
use std::env;
use std::fs::OpenOptions;
use std::io::{self};
use std::os::unix::io::AsRawFd;
use isatty::stdin_isatty;

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    let filename = args.get(1).cloned();

    if !stdin_isatty() {
        // If stdin is not a TTY, try to open /dev/tty and redirect stdin to it
        let tty_file = OpenOptions::new().read(true).write(true).open("/dev/tty")?;
        unsafe {
            libc::dup2(tty_file.as_raw_fd(), libc::STDIN_FILENO);
        }
    }

    Editor::new(filename).run()
}
