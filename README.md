# Task Warrior GPUI

![Task Warrior GPUI Screenshot](docs/task-warrior-gpui.png)

A desktop GUI for [TaskWarrior](https://taskwarrior.org/) built with [GPUI](https://gpui.rs/), the GPU-accelerated UI framework from Zed.

## Features

### Task Management
- **Create, Edit, Delete**: Full CRUD operations with keyboard shortcuts and mouse support
- **View Task Details**: Rich modal view with complete task information and metadata
- **Annotations**: Add, edit, copy, and delete annotations with visual feedback
- **Smart Autocomplete**: Hierarchical project suggestions and tag filtering

### Navigation & Filtering
- **Project Tree**: Collapsible tree view with task counts and keyboard navigation
- **Tag Filtering**: Multi-select tag filtering with task counts
- **Advanced Filters**: Filter by status, priority, due date, and search text
- **Sortable Table**: Click headers or use keyboard to sort by any column

### Keyboard-First Design
- **Complete Keyboard Navigation**: Navigate the entire UI without touching the mouse
- **Context-Aware Shortcuts**: Different shortcuts for different contexts (table, modal, filters)
- **Vim-Like Bindings**: `j`/`k` navigation, `h`/`l` for horizontal movement
- **Modal Edit Mode**: Full keyboard workflow for editing with undo/redo support

### Visual Polish
- **Dark Theme**: Ayu-inspired color scheme optimized for readability
- **Toast Notifications**: Non-intrusive feedback for actions
- **Smooth Scrolling**: GPU-accelerated rendering for buttery-smooth performance
- **Pagination**: Navigate large task lists efficiently

## Requirements

- Rust (2024 edition)
- TaskWarrior installed and configured

## Installation

```bash
# Clone the repository
git clone https://github.com/0xErwin1/taskwarrior-gpui.git
cd taskwarrior-gpui

# Option 1: Run directly
cargo run --release

# Option 2: Install globally
cargo install --path .
```

The application will use your existing TaskWarrior data directory.

## Quick Start

### Keyboard Shortcuts

| Action | Shortcut |
|--------|----------|
| Create task | `c` |
| Edit task | `e` |
| Delete task | `Delete` |
| View task | `Enter` |
| Navigate | `j`/`k` or arrow keys |
| Focus search | `Ctrl+F` |
| Sync tasks | `Ctrl+R` |
| Close modal | `Escape` |

For a complete list of keyboard shortcuts, see [docs/keyboard-shortcuts.md](docs/keyboard-shortcuts.md).

### Basic Workflow

1. **View Tasks**: Tasks are displayed in the main table with filtering options at the top
2. **Create Task**: Press `c` or click "+ New" button, fill in details, and save with `Ctrl+S`
3. **Edit Task**: Select a task and press `e`, or double-click to view then press `e`
4. **Filter**: Use the sidebar to filter by project/tag, or use the filter bar for more options
5. **Sync**: Press `Ctrl+R` to sync changes with TaskWarrior

## Documentation

- [Keyboard Shortcuts](docs/keyboard-shortcuts.md) - Complete keyboard shortcut reference
- [Keymap Guide](docs/keymap.md) - Understanding the keymap system and adding custom bindings

## License

MIT OR Apache-2.0
