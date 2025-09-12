use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub enum Action {
    // -- File operations --
    Save,
    Quit,

    // -- Cursor movement --
    MoveUp,
    MoveDown,
    MoveLeft,
    MoveRight,
    GoToStartOfLine,
    GoToEndOfLine,
    MoveWordLeft,
    MoveWordRight,
    PageUp,
    PageDown,
    GoToStartOfFile,
    GoToEndOfFile,
    MoveToNextDelimiter,
    MoveToPreviousDelimiter,

    // -- Text editing --
    InsertChar(char),
    InsertNewline,
    DeleteChar,         // Backspace
    DeleteForwardChar,  // Delete key
    DeleteWord,         // Alt-Backspace
    KillLine,
    Yank,
    Undo,
    Redo,
    Indent,
    Outdent,
    ToggleComment,
    ToggleCheckbox,

    // -- Selection --
    SetMarker,
    ClearMarker,
    CutSelection,
    CopySelection,

    // -- Search --
    EnterSearchMode,
    EnterFuzzySearchMode,

    // -- Task Management --
    EnterTaskSelectionMode,

    // -- Editor Modes --
    EnterNormalMode, // e.g., for Esc key

    // -- Miscellaneous --
    MoveLineUp,
    MoveLineDown,
    NoOp,
}
