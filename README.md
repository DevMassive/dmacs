# dmacs

dmacs is a Rust-based text editor.

## Installation

To install dmacs, you need to have Rust and Cargo installed. If you don't have them, you can install them from [rust-lang.org](https://www.rust-lang.org/tools/install).

Once Rust and Cargo are set up, clone the repository and build the project:

```bash
git clone https://github.com/your-username/dmacs.git
cd dmacs
cargo build --release
```

The executable will be located at `target/release/dmacs`. You might want to add this path to your system's `PATH` environment variable for easier access.

## Usage

To open a file:

```bash
dmacs <filename>
```

To start dmacs without opening a file:

```bash
dmacs
```

Inside the editor, you can use standard Emacs-like keybindings for navigation and editing.
