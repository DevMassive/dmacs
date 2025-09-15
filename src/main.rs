use dmacs::backup::BackupManager;
use dmacs::config::Config as DmacsConfig;
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
    let mut debug_mode = false;
    let mut no_exit_on_save = false;
    let mut restore_path: Option<String> = None;

    // Simple argument parsing
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--debug" => debug_mode = true,
            "--no-exit-on-save" => no_exit_on_save = true,
            "--restore" => {
                if i + 1 < args.len() {
                    restore_path = Some(args[i + 1].clone());
                    i += 1; // Skip next argument
                } else {
                    eprintln!("Error: --restore requires a file path.");
                    return Ok(());
                }
            }
            arg if !arg.starts_with('-') && filename.is_none() => {
                filename = Some(arg.to_string());
            }
            _ => {}
        }
        i += 1;
    }

    if debug_mode {
        WriteLogger::init(
            LevelFilter::Debug,
            Config::default(),
            File::create("dmacs_debug.log").unwrap(),
        )
        .unwrap();
    }

    if let Some(path) = restore_path {
        let backup_manager = BackupManager::new()?;
        match backup_manager.restore_backup(&path) {
            Ok(_) => println!("Successfully restored {path}"),
            Err(e) => eprintln!("Failed to restore {path}: {e}"),
        }
        return Ok(());
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

    let dmacs_config = DmacsConfig::load();

    let terminal = Terminal::new(&dmacs_config.colors)?;
    run_editor(
        &terminal,
        absolute_filename,
        no_exit_on_save,
        dmacs_config.keymap,
    )?;

    Ok(())
}
