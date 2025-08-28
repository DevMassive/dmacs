use chrono::Local;
use std::process::Command;

pub enum CommandResult {
    Success {
        new_line_content: Option<String>,
        status_message: String,
    },
    Error(String),
    NoCommand,
}

pub fn execute_command(line: &str) -> CommandResult {
    if !line.starts_with('/') {
        return CommandResult::NoCommand;
    }

    match line.trim() {
        "/today" => CommandResult::Success {
            new_line_content: Some(Local::now().format("%Y-%m-%d").to_string()),
            status_message: "/today".to_string(),
        },
        "/now" => CommandResult::Success {
            new_line_content: Some(Local::now().format("%Y-%m-%d %H:%M").to_string()),
            status_message: "/now".to_string(),
        },
        _ if line.starts_with("/tweet ") => {
            let message = line.trim_start_matches("/tweet ").trim();
            let tweet_text = format!("{{\"text\":\"{message}\"}}");

            let output = Command::new("xurl")
                .arg("-X")
                .arg("POST")
                .arg("/2/tweets")
                .arg("-d")
                .arg(&tweet_text)
                .output();

            match output {
                Ok(output) => {
                    if output.status.success() {
                        CommandResult::Success {
                            new_line_content: Some(format!("# {line}")),
                            status_message: "/tweet".to_string(),
                        }
                    } else {
                        CommandResult::Error(format!(
                            "Tweet failed: {} {}",
                            String::from_utf8_lossy(&output.stderr),
                            String::from_utf8_lossy(&output.stdout)
                        ))
                    }
                }
                Err(e) => CommandResult::Error(format!("Failed to execute xurl: {e}")),
            }
        }
        _ => CommandResult::NoCommand,
    }
}
