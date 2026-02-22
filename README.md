# Get Clocked

A desktop time-tracking app built with Tauri 2 and a Rust/WASM frontend.

## Features

- Register daily work entries with hours and category tags (Project, Task)
- Live total-hours display
- Copy workday data to clipboard as TSV
- Export to CSV or XLSX — files named `workday_{date}.csv` / `workday_{date}.xlsx`
- Persistent settings: export folder and format preference

## Tech Stack

| Technology | Role |
|---|---|
| Tauri 2 | Native app shell |
| Rust + WASM (Trunk) | Frontend compiled to WebAssembly |
| `dominator` | Reactive DOM builder |
| `dwind` | Tailwind-style utility CSS |
| `futures-signals` | Reactive state/signals |
| `rust_xlsxwriter` + `csv` | File export (XLSX and CSV) |

## Project Structure

```
Cargo.toml          # Workspace root
frontend/
  src/lib.rs        # Frontend entrypoint (WASM)
  index.html        # Trunk entry point
  Trunk.toml        # Trunk config (serves :8080, builds to dist/)
src-tauri/
  src/main.rs       # Tauri backend entrypoint
  tauri.conf.json   # App config
```

## Getting Started

**Prerequisites:**

- Rust toolchain (`rustup`)
- `trunk` — WASM bundler: `cargo install trunk`
- Tauri CLI: `cargo install tauri-cli`

**Dev commands:**

```sh
# Run the full app (native window + Trunk dev server)
cargo tauri dev

# Frontend only (browser at http://localhost:8080)
cd frontend && trunk serve

# Production build
cargo tauri build
```

## App Window

- Size: 900×600
- Theme: dark
