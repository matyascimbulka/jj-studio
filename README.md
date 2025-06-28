# JJ Studio

A beautiful GUI application for Jujutsu VCS built with Tauri, React, and TypeScript.

## Features

- 🔍 **Repository Selection**: Browse and select JJ repositories with an intuitive file dialog
- 📋 **Change Visualization**: View repository changes with descriptions, commit IDs, and metadata
- 🎨 **Modern UI**: Beautiful gradient design with glassmorphism effects
- 🔒 **Secure**: Robust path validation and sanitization for security
- ⚡ **Fast**: Native Rust backend for efficient JJ operations

## Prerequisites

- [Jujutsu VCS](https://github.com/martinvonz/jj) installed and available in PATH
- [Node.js](https://nodejs.org/) (v16 or later)
- [Rust](https://rustup.rs/) (latest stable)

## Development

1. **Clone the repository**
   ```bash
   git clone <repository-url>
   cd jj-studio
   ```

2. **Install dependencies**
   ```bash
   npm install
   ```

3. **Run in development mode**
   ```bash
   npm run tauri dev
   ```

## Building

To build the application for production:

```bash
npm run tauri build
```

## Architecture

### Frontend (React + TypeScript)
- Modern React with hooks
- TypeScript for type safety
- Lucide React for beautiful icons
- CSS with modern features (backdrop-filter, gradients)

### Backend (Rust + Tauri)
- Secure path validation and sanitization
- Async command execution with proper error handling
- Structured data parsing from JJ commands
- Cross-platform file system operations

## Security Features

- Path sanitization to prevent directory traversal attacks
- Input validation for suspicious patterns
- Canonical path resolution
- Proper error handling with specific exit codes
- Timeout protection for long-running commands

## User Interface

The application features a modern glass-morphism design with:
- Gradient backgrounds
- Blur effects and transparency
- Responsive layout
- Intuitive navigation
- Clean typography
- Smooth animations

## JJ Integration

The application integrates with Jujutsu VCS by:
- Validating repository structure (`.jj` directory presence)
- Running `jj status` for verification
- Executing `jj log` with custom templates for change data
- Parsing structured output for display

## Recommended IDE Setup

- [VS Code](https://code.visualstudio.com/) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)

## License

MIT License - see LICENSE file for details.
