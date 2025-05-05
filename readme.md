# MyTerm - Modern Terminal Emulator

A custom terminal emulator built in Rust with a built-in shell implementation.

## Features

- Custom shell implementation with common commands (ls, cd, cat, etc.)
- Beautiful syntax highlighting and colorful output
- Directory navigation with Git branch integration
- File operations (copy, move, create, delete)
- Modern UI with title bar and status bar
- VT100/ANSI escape sequence support

## Installation

### From Homebrew

```bash
brew tap yourname/myterm
brew install myterm
```

### From Source

```bash
git clone https://github.com/yourname/myterm.git
cd myterm
cargo install --path .
```

## Usage

Launch MyTerm from your applications folder or run `myterm` from the command line.

### Keyboard Shortcuts

- `Ctrl+Q`: Exit the terminal
- `Ctrl+B`: Toggle status bar
- `Ctrl+R`: Toggle raw mode (for debugging)

## Development

MyTerm is built with Rust and uses the following libraries:
- `crossterm` for terminal manipulation
- `portable-pty` for pseudo-terminal support
- `vt100` for terminal emulation
- `tokio` for async operations

## License

MIT License