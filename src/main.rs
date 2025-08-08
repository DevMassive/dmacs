use dmacs::error::Result;
use dmacs::run_editor;
use dmacs::terminal::Terminal;
use simplelog::{Config, LevelFilter, WriteLogger};
use std::env;
use std::fs::File;

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
        log::error!(
            "Panic occurred in file '{filename}' at line {line}: {message}"
        );
    }));

    let args: Vec<String> = env::args().collect();
    let filename = args.get(1).cloned();

    if args.contains(&"--debug".to_string()) {
        WriteLogger::init(
            LevelFilter::Debug,
            Config::default(),
            File::create("dmacs_debug.log").unwrap(),
        )
        .unwrap();
    }

    let terminal = Terminal::new()?;
    run_editor(&terminal, filename)?;

    Ok(())
}
