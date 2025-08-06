use dmacs::error::Result;
use dmacs::run_editor;
use dmacs::terminal::Terminal;
use simplelog::{Config, LevelFilter, WriteLogger};
use std::env;
use std::fs::File;

fn main() -> Result<()> {
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
