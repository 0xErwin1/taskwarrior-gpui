# Keymap System

This document describes how keyboard input is mapped to commands in TaskWarrior GPUI.

## Components

- `src/keymap/command.rs`: `Command` enum lists every action the UI can handle.
- `src/keymap/context.rs`: `ContextId` defines the active key contexts (Global, Table, TableHeaders, SidebarProjects, SidebarTags, Modal, FilterBar, TextInput).
- `src/keymap/chord.rs`: `Key`, `Mods`, `KeyChord` normalize keys. `KeyChord::from_gpui` builds chords from `gpui::KeyDownEvent`, `KeyChord::parse` parses strings like `Ctrl+F`, and `Display` formats chords as strings.
- `src/keymap/keymap.rs`: `KeymapLayer` stores `ContextId -> (KeyChord -> Command)` bindings. `KeymapStack` resolves by checking the top-most layer first and falling back to `Global` if nothing matches.
- `src/keymap/defaults.rs`: `build_default_keymap` defines all default bindings and is the only layer pushed today.
- `src/keymap/active_context.rs`: `FocusTarget` maps UI focus to `ContextId`.
- `src/keymap/dispatcher.rs`: `CommandDispatcher` trait abstracts command handling.

## Runtime flow

1. The root UI in `src/view/app_layout.rs` registers `on_key_down` on the app container.
2. `App::handle_key_down` in `src/app.rs` receives the event and:
   - Converts it to `KeyChord` via `KeyChord::from_gpui`.
   - Computes the active `ContextId` via `App::active_context`:
     - If the task detail modal is open, the context is `Modal`.
     - If focus is on the table and the filter bar is active, the context becomes `TextInput` or `FilterBar` based on `TaskTable::get_active_filter_context`.
     - Otherwise it uses `FocusTarget::to_context`.
   - Resolves the command with `KeymapStack::resolve`, which checks the latest layer first and falls back to `Global`.
3. If the modal is open, `App::handle_key_down` only allows `CloseModal`, `SaveModal`, `Sync`, and modal scroll commands; all other commands are ignored.
4. Some commands are handled inline in `App::handle_key_down` (focus transitions around the table, filter bar, and search input).
5. Everything else is routed through `CommandDispatcher`:
   - `App` implements it in `src/dispatcher.rs` and forwards commands to `TaskTable` or `Sidebar` based on focus.
   - `TaskTable` and `Sidebar` implement `CommandDispatcher` to apply selection, filter, and navigation changes.

## Focus, context, and scope

The same keymap can yield different commands depending on the active context. The context is computed from focus and UI state:

- `FocusTarget` (in `src/keymap/active_context.rs`) represents which major area owns focus (table, headers, sidebars).
- `App::active_context` (in `src/app.rs`) converts that focus into a `ContextId` and overrides it when:
  - The modal is open (`ContextId::Modal`).
  - The filter bar is active (`ContextId::TextInput` or `ContextId::FilterBar` based on `TaskTable::get_active_filter_context`).
- `KeymapStack::resolve` uses that `ContextId` to find a command, so the same key (like `j`) can mean "select next row" in the table context or "scroll down" in the modal context.

## How commands are interpreted by App

- `App::handle_key_down` is the gatekeeper: it resolves the command, enforces modal-only commands when the modal is open, and handles focus-related commands itself.
- Commands that mutate app-level state (focus switching, opening the modal) are handled directly in `App::handle_key_down` or helper methods on `App` (`open_task_detail`, `open_selected_task`).
- All other commands go through `App`'s `CommandDispatcher` implementation in `src/dispatcher.rs`, which decides whether to route the command to `TaskTable`, `Sidebar`, or perform app-level actions (sync, close modal, scroll modal, filter state changes).

## Event flow (short)

1. UI emits `KeyDownEvent` on the root container.
2. `App::handle_key_down` converts it to `KeyChord`, resolves `ContextId`, and maps to a `Command`.
3. `App` either:
   - Handles the command directly (focus changes, modal open/close), or
   - Dispatches it to `TaskTable`/`Sidebar`, which update their local state and call `cx.notify()`.

## Implementing a new keymap entry

1. Add a new `Command` variant in `src/keymap/command.rs`.
2. Bind a key in `src/keymap/defaults.rs` (or create a new `KeymapLayer` and push it on `KeymapStack` in `App::run`).
3. Handle the command in `App::handle_key_down` (for focus/app-level behavior) or in `src/dispatcher.rs` and the appropriate component dispatcher (`TaskTable` or `Sidebar`).
4. Update `docs/keyboard-shortcuts.md` to reflect the new binding.

## Notes

- The search input (`src/components/input/mod.rs`) handles text editing directly and updates filters on change; it early-returns on `Ctrl+H`/`Ctrl+L` so focus-navigation shortcuts still reach the app-level keymap.
- `src/keymap/mod.rs` still contains legacy `KeyBinding`/`TableAction`/`GlobalAction` types for compatibility; they are not used in the current flow.
