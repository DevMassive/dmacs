use dmacs::error::Result;
use dmacs::run_editor;
use dmacs::terminal::Terminal;
use simplelog::{Config, LevelFilter, WriteLogger};
use std::env;
use std::fs::File;

use log::debug;

fn main() -> Result<()> {
    // Set up a custom panic hook to log panics
    std::panic::set_hook(Box::new(|panic_info| {
        let (filename, line) = panic_info
            .location()
            .map_or(("unknown", 0), |loc| (loc.file(), loc.line()));
        let message = panic_info
            .payload()
            .downcast_ref::<&str>()
            .unwrap_or(&"<unknown panic>");
        log::error!("Panic occurred in file '{filename}' at line {line}: {message}");
    }));

    let args: Vec<String> = env::args().collect();
    let mut filename: Option<String> = None;

    // Parse arguments to find filename and debug flag
    let mut debug_mode = false;
    for (i, arg) in args.iter().enumerate() {
        if arg == "--debug" {
            debug_mode = true;
        } else if i == 1 && !arg.starts_with("--") {
            // Assume the first non-flag argument is the filename
            filename = Some(arg.clone());
        }
    }

    if debug_mode {
        WriteLogger::init(
            LevelFilter::Debug,
            Config::default(),
            File::create("dmacs_debug.log").unwrap(),
        )
        .unwrap();
    }

    let absolute_filename = if let Some(fname) = filename {
        match std::fs::canonicalize(&fname) {
            Ok(path) => {
                debug!(
                    "Resolved filename '{}' to absolute path '{}'",
                    fname,
                    path.display()
                );
                Some(path.to_string_lossy().into_owned())
            }
            Err(e) => {
                debug!("Could not canonicalize filename '{fname}': {e}");
                // If canonicalization fails, use the original path, it might be a new file
                Some(fname)
            }
        }
    } else {
        None
    };

    let terminal = Terminal::new()?;
    run_editor(&terminal, absolute_filename)?;

    Ok(())
}
