# Development

Your new bare-bones project includes minimal organization with a single `main.rs` file and a few assets.

```
project/
├─ assets/ # Any assets that are used by the app should be placed here
├─ src/
│  ├─ main.rs # main.rs is the entry point to your application and currently contains all components for the app
├─ Cargo.toml # The Cargo.toml file defines the dependencies and feature flags for your project
```

### Tailwind
1. Install npm: https://docs.npmjs.com/downloading-and-installing-node-js-and-npm
2. Install the Tailwind CSS CLI: https://tailwindcss.com/docs/installation
3. Run the following command in the root of the project to start the Tailwind CSS compiler:

```bash
npx tailwindcss -i ./tailwind.css -o ./assets/tailwind.css --watch
```

### Serving Your App

Run the following command in the root of your project to start developing with the default platform:

```bash
dx serve
```

To run for a different platform, use the `--platform platform` flag. E.g.
```bash
dx serve --platform desktop
```

## Overview

dx_todo_app is a Dioxus desktop to-do manager with project-based organization, tasks, and subtasks. It supports switching between projects and exporting a project's tasks to PDF.

## Features

- __Projects__: Create/select multiple projects; tasks are scoped to the active project.
- __Tasks & Subtasks__: Add, edit, complete; basic keyboard and accessible controls.
- __Reorder__: Drag handle for task ordering (desktop).
- __Export to PDF__: Header button exports the active project's tasks/subtasks to a PDF.
- __Persistence__: Data saved as JSON in the OS app data directory.

## Project Structure

```
src/
  main.rs                # App, routes, state, PDF export wiring
  models.rs              # Project, Todo, Subtask, Filter models
  storage.rs             # Load/save projects, migration from old todos
  components/
    header.rs            # Header with Switch/Export
    projects.rs          # Projects screen (list/create/open)
    add_form.rs          # Input row for adding tasks
    filter_bar.rs        # Filter controls
    todo_item.rs         # A single task row
assets/
  main.css               # App styles
  favicon.ico
```

## Build & Run (Desktop)

Prerequisites: Rust toolchain, cargo.

```bash
cargo run --release
```

Notes:
- On first run, a default project is created. Select a project from the Projects screen.
- The header shows the active project and provides Switch/Export actions.

## Export to PDF

- Click the header "Export" button in the List view.
- Choose a destination in the native save dialog.
- A simple A4 PDF is generated listing tasks and subtasks with [ ]/[x].

Implementation details:
- Uses `printpdf` with built-in Helvetica; no external font files.
- Uses `rfd` for the native file save dialog.

## Data Storage

- Projects are stored as JSON in the OS-specific app data directory (via `directories`).
- There is automatic migration from legacy `todos.json` to project-based storage.

## Development Tips

- Logs in the terminal trace key actions (project switching, export status).
- Styles are injected at the app root so the Projects screen is styled on first load.
- If you see warnings like "variable does not need to be mutable," you can run:

```bash
cargo fix
```

## Mobile Roadmap (Native iOS/Android)

This repository targets desktop. To add native mobile apps while sharing UI/state:

- Split into a Cargo workspace:
  - `app_core`: shared UI/state/services (no platform code).
  - `app_desktop`: `dioxus-desktop` + desktop services (file dialogs, FS writes).
  - `app_mobile`: `dioxus-mobile` + mobile services (save to app docs dir, share sheet).
- Abstract platform needs behind traits (StorageService, ExportService) and provide per-platform impls.
- Replace mouse-only drag with touch-friendly reorder on mobile.

References:
- dioxus-mobile crate: https://crates.io/crates/dioxus-mobile
- Dioxus 0.6 release (mentions mobile templates): https://dioxuslabs.com/blog/release-060/

## Dependencies

- dioxus, dioxus-desktop, dioxus-router
- serde, serde_json
- directories
- rfd (native dialogs)
- printpdf (PDF generation)

## License

MIT or Apache-2.0, at your option.

