# GitRep - Git Repository Inspector

A high-performance terminal user interface (TUI) for exploring and analyzing Git repositories.

## Features

### Phase 1 - Foundations
- [x] TUI skeleton with multi-pane layout
- [x] Git backend abstraction using libgit2
- [x] Commit list with navigation
- [x] Commit detail view with metadata
- [x] Diff viewer with syntax highlighting
- [x] Vim-style keyboard navigation
- [x] Help overlay

### Phase 2 (Current) - Diff & Staging
- [x] Staging area view with staged/unstaged sections
- [x] Stage/unstage individual files
- [x] Stage/unstage all changes
- [x] Commit creation with message input
- [x] Working tree status display

### Planned Features
- **Phase 3**: Branch/tag browser, checkout operations
- **Phase 4**: Repository analytics (commit heatmap, contributor stats)
- **Phase 5**: Configuration system, themes, performance tuning

## Installation

### Building from Source

```bash
# Clone the repository
git clone https://github.com/example/gitrep.git
cd gitrep

# Build release version
cargo build --release

# The binary will be at target/release/gitrep
```

## Usage

```bash
# Run in current directory
gitrep

# Run in a specific directory
gitrep /path/to/repository
```

## Keyboard Shortcuts

### Navigation
| Key | Action |
|-----|--------|
| `j` / `↓` | Move down |
| `k` / `↑` | Move up |
| `g` | Go to first commit |
| `G` | Go to last commit |
| `Ctrl+d` / `PgDn` | Page down |
| `Ctrl+u` / `PgUp` | Page up |

### Actions
| Key | Action |
|-----|--------|
| `Enter` | Toggle detail view |
| `Tab` | Next pane |
| `Shift+Tab` | Previous pane |
| `s` | Open staging area |
| `/` | Search |
| `?` | Toggle help |

### Staging Mode
| Key | Action |
|-----|--------|
| `Space` / `Enter` | Stage/unstage file |
| `a` | Stage all changes |
| `u` | Unstage all changes |
| `c` | Create commit |
| `r` | Refresh status |

### General
| Key | Action |
|-----|--------|
| `q` | Quit |
| `Ctrl+c` | Force quit |
| `Esc` | Go back / Cancel |

## Configuration

Configuration file location: `~/.config/gitrep/config.yml`

```yaml
theme:
  name: default
  unicode: true

keybindings:
  vim_mode: true

performance:
  commit_batch_size: 100
  enable_cache: true
```

## Requirements

- Git 2.20 or later
- Terminal with 256-color support recommended

## Tech Stack

- **Language**: Rust
- **TUI Framework**: ratatui + crossterm
- **Git Backend**: git2-rs (libgit2 bindings)
- **Async Runtime**: tokio
- **Configuration**: serde + serde_yaml

## License

MIT License
