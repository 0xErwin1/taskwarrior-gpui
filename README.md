# Task Warrior GPUI

![Task Warrior GPUI Screenshot](docs/task-warrior-gpui.png)

A desktop GUI for [TaskWarrior](https://taskwarrior.org/) built with [GPUI](https://gpui.rs/), the GPU-accelerated UI framework from Zed.

## Features

- View and filter tasks by project, status, priority, and due date
- Project tree with task counts
- Tag filtering with multi-select
- Sortable task table with pagination
- Dark theme (Ayu-inspired)

## Requirements

- Rust 2024 edition
- TaskWarrior

## Build

```bash
cargo build --release
```

## Run

```bash
cargo run
```

## License

MIT OR Apache-2.0
