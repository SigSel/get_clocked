# Get Clocked

A multi-page dark-themed desktop app built with Tauri 2 (native shell) and a Rust/WASM frontend for registering daily work entries and exporting them.

## Architecture

**Two-crate Cargo workspace:**
- `frontend/` — frontend compiled to WASM via Trunk. Organized into a `pages/` module with five pages: `Home`, `Settings`, `RegisterWorkday`, `TemplateMaker`, and `CategoryManager`. Uses `dominator` for reactive DOM, `dwind` for Tailwind-style utility classes, and `futures-signals` for reactivity.
- `src-tauri/` — Tauri 2 native backend. Handles the app window and exposes Tauri commands to the frontend via `tauri-wasm`.

**Build pipeline:** Trunk bundles the WASM frontend into `frontend/dist/`, which Tauri serves as the app's UI.

## Key Dependencies

| Crate | Purpose |
|---|---|
| `dominator` | Reactive DOM builder for WASM |
| `dwind` + `dwind-macros` | Tailwind-like utility CSS via `dwclass!` macro |
| `futures-signals` | Reactive signals/state |
| `tauri-wasm` | Call Tauri commands from WASM frontend |
| `wasm-bindgen` | Rust ↔ JS interop |
| `tauri` 2 | Native app shell (backend) |
| `tauri-plugin-dialog` | Native file/folder picker dialogs |
| `rust_xlsxwriter` | XLSX file generation |
| `csv` | CSV file generation |
| `serde` + `serde_json` | Serialization/deserialization |
| `web-sys` | Browser/DOM APIs (clipboard, inputs) |

## Dev Commands

```sh
# Run the full app (Tauri opens a window, Trunk serves frontend at :8080)
cargo tauri dev

# Frontend only (browser at http://localhost:8080)
cd frontend && trunk serve

# Production build
cargo tauri build
```

## Project Structure

```
Cargo.toml          # Workspace root
.github/workflows/
  ci.yml            # CI: cross-platform build verification
  release.yml       # Release: builds + GitHub Release on version tags
frontend/
  Cargo.toml        # Frontend WASM crate
  index.html        # Trunk entry point (links the Rust crate)
  Trunk.toml        # Trunk config (serves :8080, builds to dist/)
  src/
    lib.rs          # Frontend entrypoint
    app.rs          # App state and routing
    pages/
      mod.rs
      home.rs
      settings.rs
      register_workday.rs
      template_maker.rs
      category_manager.rs
  dist/             # Trunk build output (gitignored)
src-tauri/
  src/main.rs       # Tauri backend entrypoint
  tauri.conf.json   # App config (name, window size, build commands)
  Cargo.toml        # Backend crate (v0.1.3)
  capabilities/     # Tauri permission capabilities
  icons/            # App icons (PNG, ICNS, ICO)
target/             # Rust build artifacts (gitignored)
```

## App Config

- Version: 0.1.3
- Window: 900×600, title "Get Clocked"
- Identifier: `com.getclocked.app`
- Dev URL: `http://localhost:8080` (Trunk)
- Platforms: Linux (AppImage), Windows (NSIS), macOS (DMG)

**Backend commands:** `get_settings`, `save_settings`, `pick_folder`, `export_workday`, `export_monthly`, `save_template`, `list_templates`, `get_categories`, `save_categories`, `pick_categories_file`, `import_categories`

**Settings** (`$APP_CONFIG_DIR/settings.json`): `export_folder`, `export_format`, `template_folder`

**Category definitions** (`$APP_CONFIG_DIR/category_definitions.json`): list of `{ name, values[] }` entries used for autocomplete in RegisterWorkday and TemplateMaker; can be imported from CSV/XLSX via CategoryManager.

---

<!-- refstore -->
## refstore

This project uses refstore to manage reference documentation. Read files in `.references/` for project-relevant context — each subdirectory maps to an entry in `refstore.toml`.

References can be added individually or via **bundles** (named groups of references defined in the central repository, e.g. a tech stack or project template). Bundles are listed under `bundles = [...]` in `refstore.toml` and expanded at sync time.

Commands: `refstore status`, `refstore sync`, `refstore list`, `refstore search <query>`, `refstore add <name>`, `refstore add --bundle <name>`, `refstore remove <name> --purge`

MCP tools: `list_references`, `get_reference`, `read_reference_file`, `list_reference_files`, `search_references`, `list_bundles`, `get_bundle`

