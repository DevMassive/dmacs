use dmacs::error::Result;
use dmacs::run_editor;
use dmacs::terminal::Terminal;
use std::env;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    let filename = args.get(1).cloned();

    let terminal = Terminal::new()?;
    run_editor(&terminal, filename)?;

    Ok(())
}
