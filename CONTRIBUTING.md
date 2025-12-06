# Contributing to Thunderlink

First off, thank you for considering contributing to Thunderlink! 🎉

Every contribution helps make Thunderlink better for everyone. Whether you're fixing a typo, reporting a bug, or implementing a new feature – we appreciate your effort.

## 📋 Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [How Can I Contribute?](#how-can-i-contribute)
- [Development Workflow](#development-workflow)
- [Style Guidelines](#style-guidelines)
- [Pull Request Process](#pull-request-process)

## 📜 Code of Conduct

This project follows a simple code of conduct:

- **Be respectful** – Treat everyone with respect and kindness
- **Be inclusive** – Welcome contributors of all backgrounds and skill levels
- **Be constructive** – Focus on improving the project, not criticizing people
- **Be patient** – Remember that maintainers are volunteers

## 🚀 Getting Started

### Prerequisites

1. **Fork** the repository on GitHub
2. **Clone** your fork locally:
   ```bash
   git clone https://github.com/YOUR_USERNAME/thunderlink.git
   cd thunderlink
   ```
3. **Set up** the development environment:
   ```bash
   # Install Node.js dependencies
   npm install
   
   # Make sure Rust is installed
   rustc --version  # Should be 1.70+
   ```
4. **Create a branch** for your work:
   ```bash
   git checkout -b feature/your-feature-name
   ```

### Running the Development Server

```bash
# Start the app in development mode with hot-reload
npm run tauri dev
```

### Building for Production

```bash
npm run tauri build
```

## 🤔 How Can I Contribute?

### 🐛 Reporting Bugs

Before creating a bug report, please check if the issue already exists.

**Great bug reports include:**
- A clear, descriptive title
- Steps to reproduce the issue
- Expected vs. actual behavior
- Your environment (OS, Rust version, etc.)
- Screenshots or logs if applicable

### 💡 Suggesting Features

Feature requests are welcome! Please provide:
- A clear description of the feature
- Why it would be useful
- Possible implementation approaches (optional)

### 🔧 Pull Requests

We actively welcome pull requests for:
- Bug fixes
- New features
- Documentation improvements
- Performance optimizations
- Code refactoring

## 💻 Development Workflow

### Project Structure

```
thunderlink/
├── src/                    # Frontend
│   ├── main.js            # UI logic, event handling
│   └── styles.css         # Styling (CSS variables, animations)
│
├── src-tauri/             # Backend (Rust)
│   └── src/
│       ├── lib.rs         # Tauri commands (API)
│       ├── network.rs     # Network interface detection
│       ├── discovery.rs   # Peer discovery (UDP broadcast)
│       ├── transfer.rs    # File transfer (TCP streaming)
│       └── state.rs       # Application state
│
├── package.json           # Node.js dependencies
└── src-tauri/Cargo.toml   # Rust dependencies
```

### Architecture Overview

```
┌─────────────────────────────────────────────────────────┐
│                      Frontend (JS)                       │
│                    src/main.js                          │
├─────────────────────────────────────────────────────────┤
│                    Tauri Bridge                          │
│               invoke() / listen()                        │
├─────────────────────────────────────────────────────────┤
│                    Backend (Rust)                        │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐              │
│  │ Discovery │  │ Transfer │  │ Network  │              │
│  │   (UDP)   │  │  (TCP)   │  │Detection │              │
│  └──────────┘  └──────────┘  └──────────┘              │
└─────────────────────────────────────────────────────────┘
```

### Common Tasks

#### Adding a new Tauri command

1. Define the command in `src-tauri/src/lib.rs`:
   ```rust
   #[tauri::command]
   async fn my_command(arg: String) -> Result<String, String> {
       Ok(format!("Hello, {}", arg))
   }
   ```

2. Register it in the handler:
   ```rust
   .invoke_handler(tauri::generate_handler![
       // ... existing commands
       my_command,
   ])
   ```

3. Call it from the frontend:
   ```javascript
   const result = await invoke('my_command', { arg: 'World' });
   ```

#### Modifying the UI

- Styles are in `src/styles.css` using CSS variables
- UI components are built in `src/main.js`
- Follow the existing cyber/dark theme aesthetic

## 📝 Style Guidelines

### Rust Code

- Use `cargo fmt` before committing
- Run `cargo clippy` and address warnings
- Follow [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- Document public functions with `///` comments

```rust
/// Calculates the transfer speed in human-readable format.
///
/// # Arguments
/// * `bytes_per_second` - Transfer speed in bytes/second
///
/// # Returns
/// Formatted string like "1.5 MB/s"
pub fn format_speed(bytes_per_second: u64) -> String {
    // ...
}
```

### JavaScript Code

- Use modern ES6+ syntax
- Prefer `const` over `let`, avoid `var`
- Use meaningful variable names
- Add comments for complex logic

### CSS Code

- Use CSS variables for colors and sizes
- Follow BEM-like naming: `.component-name`, `.component-name-element`
- Keep animations performant (use `transform`, `opacity`)

### Commit Messages

Follow [Conventional Commits](https://conventionalcommits.org/):

```
<type>(<scope>): <description>

[optional body]

[optional footer]
```

**Types:**
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation only
- `style`: Formatting, no code change
- `refactor`: Code restructuring
- `perf`: Performance improvement
- `test`: Adding tests
- `chore`: Maintenance tasks

**Examples:**
```
feat(transfer): add resume capability for interrupted transfers
fix(discovery): handle timeout when peer disconnects
docs(readme): add Linux installation instructions
```

## 🔄 Pull Request Process

1. **Update documentation** if your change affects usage
2. **Add tests** for new functionality (where applicable)
3. **Ensure CI passes** – all checks must be green
4. **Request review** from maintainers
5. **Address feedback** promptly and constructively

### PR Title Format

Use the same format as commit messages:
```
feat(scope): add cool new feature
```

### PR Description Template

```markdown
## Description
Brief description of changes

## Type of Change
- [ ] Bug fix
- [ ] New feature
- [ ] Documentation update
- [ ] Performance improvement
- [ ] Other (please describe)

## Testing
How was this tested?

## Screenshots (if applicable)
Add screenshots for UI changes

## Checklist
- [ ] Code follows style guidelines
- [ ] Self-review completed
- [ ] Documentation updated
- [ ] Tests added/updated
```

## ❓ Questions?

Feel free to:
- Open a [Discussion](https://github.com/yourusername/thunderlink/discussions)
- Ask in an Issue
- Reach out to maintainers

---

**Thank you for contributing to Thunderlink!** ⚡

Every contribution, no matter how small, makes a difference.

