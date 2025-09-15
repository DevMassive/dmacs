use crate::editor::actions::Action;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use toml;

#[derive(Deserialize, Debug, Default)]
struct PartialConfig {
    #[serde(default)]
    colors: PartialColors,
    #[serde(default)]
    keymap: Keymap,
}

#[derive(Deserialize, Debug, Default)]
struct PartialColors {
    bg: Option<String>,
    fg: Option<String>,
    bold: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Colors {
    pub bg: String,
    pub fg: String,
    pub bold: String,
}

impl Default for Colors {
    fn default() -> Self {
        Self {
            bg: "#33302d".to_string(),
            fg: "#d0d0d0".to_string(),
            bold: "#f5c373".to_string(),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct Config {
    pub colors: Colors,
    pub keymap: Keymap,
}

impl Config {
    pub fn load() -> Self {
        let mut config = Config::default();

        if let Some(home_dir) = dirs::home_dir() {
            let config_path = home_dir.join(".dmacs").join("config.toml");
            if config_path.exists() {
                if let Ok(contents) = fs::read_to_string(&config_path) {
                    match toml::from_str::<PartialConfig>(&contents) {
                        Ok(user_config) => {
                            if let Some(bg) = user_config.colors.bg {
                                config.colors.bg = bg;
                            }
                            if let Some(fg) = user_config.colors.fg {
                                config.colors.fg = fg;
                            }
                            if let Some(bold) = user_config.colors.bold {
                                config.colors.bold = bold;
                            }
                            config.keymap.bindings.extend(user_config.keymap.bindings);
                        }
                        Err(e) => {
                            log::error!("Failed to parse config.toml: {e}");
                        }
                    }
                }
            } else {
                // Backward compatibility: load old keymap.toml if config.toml doesn't exist
                let keymap_path = home_dir.join(".dmacs").join("keymap.toml");
                if keymap_path.exists() {
                    if let Ok(contents) = fs::read_to_string(&keymap_path) {
                        match toml::from_str::<Keymap>(&contents) {
                            Ok(user_keymap) => {
                                config.keymap.bindings.extend(user_keymap.bindings);
                            }
                            Err(e) => {
                                log::error!("Failed to parse keymap.toml: {e}");
                            }
                        }
                    }
                }
            }
        }
        config
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct Keymap {
    #[serde(flatten)]
    pub bindings: HashMap<String, Action>,
}

impl Keymap {
    pub fn new() -> Self {
        Self {
            bindings: HashMap::new(),
        }
    }
}

impl Default for Keymap {
    fn default() -> Self {
        let mut bindings = HashMap::new();

        // File Operations
        bindings.insert("alt-s".to_string(), Action::Save);
        bindings.insert("ctrl-x".to_string(), Action::Quit);

        // Cursor Movement
        bindings.insert("up".to_string(), Action::MoveUp);
        bindings.insert("down".to_string(), Action::MoveDown);
        bindings.insert("left".to_string(), Action::MoveLeft);
        bindings.insert("right".to_string(), Action::MoveRight);
        bindings.insert("ctrl-a".to_string(), Action::GoToStartOfLine);
        bindings.insert("ctrl-e".to_string(), Action::GoToEndOfLine);
        bindings.insert("alt-f".to_string(), Action::MoveWordRight); // alt-right
        bindings.insert("alt-b".to_string(), Action::MoveWordLeft); // alt-left
        bindings.insert("ctrl-b".to_string(), Action::MoveWordLeft);
        bindings.insert("alt-up".to_string(), Action::MoveLineUp);
        bindings.insert("alt-down".to_string(), Action::MoveLineDown);
        bindings.insert("ctrl-v".to_string(), Action::PageDown);
        bindings.insert("alt-v".to_string(), Action::PageUp);
        bindings.insert("ctrl-n".to_string(), Action::MoveToNextDelimiter);
        bindings.insert("ctrl-p".to_string(), Action::MoveToPreviousDelimiter);
        bindings.insert("alt->".to_string(), Action::GoToEndOfFile);
        bindings.insert("alt-<".to_string(), Action::GoToStartOfFile);

        // Text Editing
        bindings.insert("backspace".to_string(), Action::DeleteChar);
        bindings.insert("delete".to_string(), Action::DeleteForwardChar);
        bindings.insert("ctrl-d".to_string(), Action::DeleteForwardChar);
        bindings.insert("alt-backspace".to_string(), Action::DeleteWord);
        bindings.insert("ctrl-k".to_string(), Action::KillLine);
        bindings.insert("ctrl-y".to_string(), Action::Yank);
        bindings.insert("ctrl-_".to_string(), Action::Undo);
        bindings.insert("alt-_".to_string(), Action::Redo);
        bindings.insert("tab".to_string(), Action::Indent);
        bindings.insert("shift-tab".to_string(), Action::Outdent);
        bindings.insert("alt-/".to_string(), Action::ToggleComment);
        bindings.insert("ctrl-t".to_string(), Action::ToggleCheckbox);
        bindings.insert("enter".to_string(), Action::InsertNewline);

        // Selection
        bindings.insert("ctrl-space".to_string(), Action::SetMarker);
        bindings.insert("ctrl-w".to_string(), Action::CutSelection);
        bindings.insert("alt-w".to_string(), Action::CopySelection);
        bindings.insert("ctrl-g".to_string(), Action::ClearMarker);

        // Search
        bindings.insert("ctrl-s".to_string(), Action::EnterSearchMode);
        bindings.insert("ctrl-f".to_string(), Action::EnterFuzzySearchMode);

        // Modes
        bindings.insert("esc".to_string(), Action::EnterNormalMode);

        Self { bindings }
    }
}
