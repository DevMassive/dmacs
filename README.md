# dmacs

`dmacs` is a terminal text editor developed for DevMassive's daily use. It is built with Rust and designed for simplicity and efficiency.

## Features

`dmacs` provides not only basic text editing functions but also a variety of features to streamline daily note-taking and task management.

- **Basic Editing**: Covers all the essential functions expected of a modern editor, including file operations, undo/redo, rectangular selection, and copy/paste.
- **Optimized Word Movement for Japanese**: Word-based cursor movement correctly recognizes Japanese sentence structures.
- **Markdown Support**: Includes a checkbox toggle feature (`- [ ]`) that allows it to be used as a simple task list.
- **Incremental Search**: `Ctrl-S`, `Ctrl-R` for a comfortable search experience.
- **Fuzzy Search**: `Ctrl-F` allows you to fuzzy search for lines within the buffer and jump quickly.
- **Command Functions**: Easily insert the date and time with commands like `/today` and `/now`.
- **Task Management**: The `/task` command lists uncompleted tasks (`- [ ]`), allowing you to move, organize, or comment them out.
- **Automatic Backup**: Automatically creates a backup when saving a file.
- **Cursor Position Persistence**: Remembers the last cursor position for each file and restores it on the next launch.

## Installation

You need to have Rust and Cargo installed. Please install them from [rust-lang.org](https://www.rust-lang.org/tools/install).

```bash
git clone https://github.com/DevMassive/dmacs.git
cd dmacs
cargo install --path .
```

## Usage

To open a file:
```bash
dmacs <filename>
```

## Keybindings

### File Operations

| Key | Action |
|---|---|
| `Alt-S` | Save file |
| `Ctrl-X` | Save file and exit |

### Cursor Movement

| Key | Action |
|---|---|
| `↑` `↓` `←` `→` | Move cursor |
| `Ctrl-A` | Go to beginning of line |
| `Ctrl-E` | Go to end of line |
| `Alt-→` | Move to next word |
| `Alt-←` | Move to previous word |
| `Alt-Up` | Move line up |
| `Alt-Down` | Move line down |
| `Meta-V` / `Ctrl-V` | Scroll up/down by a page |
| `Ctrl-N` | Move to next delimiter (`---`) |
| `Ctrl-P` | Move to previous delimiter (`---`) |

### Text Editing

| Key | Action |
|---|---|
| `Ctrl-D` | Delete character at cursor position |
| `Backspace` | Delete character before cursor |
| `Alt-Backspace` | Delete word before cursor |
| `Ctrl-K` | Cut (Kill) from cursor to end of line |
| `Ctrl-Y` | Paste (Yank) cut text |
| `Ctrl-_` | Undo |
| `Alt-_` | Redo |
| `Tab` | Indent |
| `Shift-Tab` | Outdent |
| `Alt-/` | Toggle line comment |
| `Ctrl-T` | Toggle checkbox state |

### Selection

| Key | Action |
|---|---|
| `Ctrl-Space` | Mark start of selection |
| `Ctrl-W` | Cut (Kill) selection |
| `Alt-W` | Copy selection |
| `Ctrl-G` | Cancel selection |

### Search

| Key | Action |
|---|---|
| `Ctrl-S` | Incremental search (forward) |
| `Ctrl-R` | Incremental search (backward) |
| `Ctrl-F` | Fuzzy search for lines in the buffer |

### Commands

| Command | Action |
|---|---|
| `/today` | Insert current date in `YYYY-MM-DD` format |
| `/now` | Insert current date and time in `YYYY-MM-DD HH:MM` format |
| `/task` | Start task management mode |
