# Project Overview

`dmacs` is a Rust-based text editor designed to run in the terminal, featuring Emacs-like keybindings. It leverages the `pancurses` library for terminal user interface interactions, `simplelog` for robust logging capabilities (including a debug mode), and `thiserror` for declarative error handling. The editor supports opening files, basic text editing operations, and handles terminal resizing and user input events.

# Building and Running

This project uses Cargo, Rust's package manager and build system.

## Prerequisites

*   Rust and Cargo (installable from [rust-lang.org](https://www.rust-lang.org/tools/install))

## Build

To build the project in release mode (optimized for performance):

```bash
cargo build --release
```

The executable will be generated at `target/release/dmacs`.

## Run

To run the editor without opening a file:

```bash
./target/release/dmacs
```

To open a specific file:

```bash
./target/release/dmacs <filename>
```

### Debug Mode

To run the editor with debug logging enabled (logs to `dmacs_debug.log`):

```bash
./target/release/dmacs --debug [filename]
```

## Testing

To run the project's tests:

```bash
cargo test
```

# Development Conventions

*   **Language Edition**: Rust 2024 Edition.
*   **Logging**: Utilizes `simplelog` for logging, with a configurable debug mode that outputs to `dmacs_debug.log`.
*   **Error Handling**: Employs the `thiserror` crate for defining and managing custom error types.
*   **Terminal UI**: Built upon the `pancurses` library for cross-platform terminal interface management.
*   **Project Structure**:
    *   Core library logic is encapsulated in `src/lib.rs`.
    *   The main application entry point is `src/main.rs`.
    *   Modules are organized within the `src/` directory (e.g., `document`, `editor`, `error`, `terminal`).
    *   Tests are located in the `tests/` directory, with editor-specific tests found under `tests/editor/mod.rs`.
*   **Verification**:
    *   Always run tests (`cargo test`) after completing a task to ensure no regressions.
    *   Ensure code formatting and linting are applied using:
        ```bash
        cargo fmt --all && cargo clippy --all-targets --fix --allow-dirty
        ```