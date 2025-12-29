# Keyboard Shortcuts

This document lists the keyboard shortcuts available in the TaskWarrior GPUI application.
Shortcuts are context-aware and map to the default keymap in `src/keymap/defaults.rs`.

## Global Shortcuts

These shortcuts work from any context:

| Shortcut | Action |
|----------|--------|
| `Ctrl+R` | Sync tasks with TaskWarrior |
| `Ctrl+F` | Focus search input |
| `Tab` | Cycle focus forward (Table → Sidebar Projects → Sidebar Tags → Table) |
| `Shift+Tab` | Cycle focus backward (Table → Sidebar Tags → Sidebar Projects → Table) |
| `Ctrl+C` | Clear all active filters |
| `Ctrl+P` | Clear project filter |
| `Ctrl+T` | Clear tag filter |
| `Ctrl+X` | Clear search and dropdown filters |
| `Escape` | Close modal (if open) |

## Focus Navigation

The UI is fully navigable with the keyboard. Focus moves between the main areas (table and sidebars),
and within the table area you can move to headers or the filter bar.

### Main Focus Cycle

| Shortcut | Action |
|----------|--------|
| `Tab` | Table → Sidebar Projects → Sidebar Tags → Table |
| `Shift+Tab` | Table → Sidebar Tags → Sidebar Projects → Table |

### Focus Movement by Area

| From | Shortcut | Action |
|------|----------|--------|
| Task Table | `Ctrl+K` | Focus table headers |
| Task Table | `Ctrl+H` | Focus sidebar projects |
| Table Headers | `Ctrl+J` | Focus task table |
| Table Headers | `Ctrl+K` | Focus search input |
| Search Input | `Ctrl+J` | Focus table headers |
| Search Input | `Ctrl+L` / `Ctrl+H` | Next / previous filter dropdown |
| Filter Dropdowns | `Ctrl+J` | Focus table headers |
| Filter Dropdowns | `Ctrl+L` / `Ctrl+H` | Next / previous filter dropdown |
| Sidebar Projects | `Ctrl+L` | Focus task table |
| Sidebar Projects | `Ctrl+J` | Focus sidebar tags |
| Sidebar Tags | `Ctrl+L` | Focus task table |
| Sidebar Tags | `Ctrl+K` | Focus sidebar projects |

## Task Table

These shortcuts work when the task table has focus:

### Navigation

| Shortcut | Action |
|----------|--------|
| `j` / `↓` | Select next row |
| `k` / `↑` | Select previous row |
| `g` / `Home` | Select first row |
| `Shift+G` / `End` | Select last row |
| `l` / `PageDown` | Next page |
| `h` / `PageUp` | Previous page |
| `Escape` | Clear selection |

### Actions

| Shortcut | Action |
|----------|--------|
| `Enter` | Open selected task (not yet wired to a details view) |
| `←` | Collapse current project |
| `→` | Expand current project |

### Focus Movement

| Shortcut | Action |
|----------|--------|
| `Ctrl+K` | Focus table headers |
| `Ctrl+H` | Focus sidebar projects |

## Table Headers

These shortcuts work when table column headers have focus:

### Column Navigation

| Shortcut | Action |
|----------|--------|
| `h` / `←` | Move to previous column header |
| `l` / `→` | Move to next column header |

### Sorting

| Shortcut | Action |
|----------|--------|
| `j` | Cycle sort order for current column |
| `k` | Cycle sort order for current column |
| `Enter` | Cycle sort order for current column |
| `Space` | Cycle sort order for current column |

### Focus Movement

| Shortcut | Action |
|----------|--------|
| `Ctrl+J` | Focus task table |
| `Ctrl+K` | Focus filter bar (search input) |

## Sidebar - Projects

These shortcuts work when the projects sidebar has focus:

### Navigation

| Shortcut | Action |
|----------|--------|
| `j` / `↓` | Select next project |
| `k` / `↑` | Select previous project |
| `g` | Select first project |
| `Shift+G` | Select last project |
| `h` / `←` | Collapse current project |
| `l` / `→` | Expand current project |

### Actions

| Shortcut | Action |
|----------|--------|
| `Enter` | Filter by selected project |
| `Space` | Filter by selected project |

### Focus Movement

| Shortcut | Action |
|----------|--------|
| `Ctrl+L` | Focus task table |
| `Ctrl+J` | Focus sidebar tags |

## Sidebar - Tags

These shortcuts work when the tags sidebar has focus:

### Navigation

| Shortcut | Action |
|----------|--------|
| `j` / `↓` | Select next tag |
| `k` / `↑` | Select previous tag |
| `g` | Select first tag |
| `Shift+G` | Select last tag |

### Actions

| Shortcut | Action |
|----------|--------|
| `Enter` | Filter by selected tag |
| `Space` | Filter by selected tag |

### Focus Movement

| Shortcut | Action |
|----------|--------|
| `Ctrl+L` | Focus task table |
| `Ctrl+K` | Focus sidebar projects |

## Filter Bar - Text Input

These shortcuts work when the search input has focus:

| Shortcut | Action |
|----------|--------|
| `Escape` | Blur input (return focus to table) |
| `Ctrl+L` | Focus next filter dropdown |
| `Ctrl+H` | Focus previous filter dropdown |
| `Ctrl+J` | Focus table headers |

Note: search filtering updates as you type (no explicit "apply" key needed).

## Filter Bar - Dropdowns

These shortcuts work when a filter dropdown (Status/Priority/Due) has focus:

### Dropdown Control

| Shortcut | Action |
|----------|--------|
| `Enter` | Toggle dropdown open/closed |
| `Space` | Toggle dropdown open/closed |
| `Escape` | Close dropdown and blur |

### Option Selection

| Shortcut | Action |
|----------|--------|
| `j` / `↓` | Select next option |
| `k` / `↑` | Select previous option |

### Focus Movement

| Shortcut | Action |
|----------|--------|
| `Ctrl+L` | Focus next filter element |
| `Ctrl+H` | Focus previous filter element |
| `Ctrl+J` | Focus table headers |

## Modal (Task Details)

These shortcuts work when viewing task details:

| Shortcut | Action |
|----------|--------|
| `Escape` | Close modal |
| `Ctrl+Enter` | Save changes and close (not yet wired) |

## Search Input Editing

These are handled by the input component while the search input is focused:

| Shortcut | Action |
|----------|--------|
| `Left` / `Right` | Move cursor |
| `Ctrl+Left` / `Ctrl+Right` | Move by word |
| `Home` / `End` | Jump to start/end |
| `Backspace` / `Delete` | Delete character |
| `Ctrl+Backspace` / `Ctrl+Delete` | Delete word |
| `Ctrl+A` / `Ctrl+E` | Jump to start/end |
| `Ctrl+U` / `Ctrl+K` | Delete to start/end |
