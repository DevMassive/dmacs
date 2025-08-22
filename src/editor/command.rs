use chrono::Local;

pub fn execute_command(line: &str) -> Option<String> {
    if !line.starts_with('/') {
        return None;
    }

    match line.trim() {
        "/today" => Some(Local::now().format("%Y-%m-%d").to_string()),
        "/now" => Some(Local::now().format("%Y-%m-%d %H:%M").to_string()),
        _ => None,
    }
}
