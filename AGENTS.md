# Repository Guidelines

## Project Structure & Module Organization
- `src/` holds the React + TypeScript UI (main window and overlay).
- `index.html` and `overlay.html` are the Vite entry points for the two windows.
- `src-tauri/` contains the Rust backend, `src-tauri/tauri.conf.json`, and app icons in `src-tauri/icons/`.
- `ui/` is for auxiliary UI assets or prototypes used by the frontend.
- `scripts/` has build/release helpers; `dist/` is generated frontend output.

## Build, Test, and Development Commands
- `npm install` installs frontend dependencies.
- `npm run dev` starts the Vite dev server for the UI.
- `npm run build` type-checks and builds the frontend bundle.
- `npm run preview` serves the built UI locally.
- `npm run tauri dev` runs the desktop app in dev mode; run as Administrator so global hotkeys work.
- `npm run tauri build` builds the NSIS installer only; output in `src-tauri/target/release/bundle/`.
- `cd src-tauri` then `cargo build`, `cargo check`, or `cargo test` for the Rust backend.
- `cd src-tauri` then `cargo run --bin test_api` to manually verify ASR API behavior.

## Coding Style & Naming Conventions
- TypeScript/React: 2-space indent, double quotes, and semicolons; components use `PascalCase`, hooks use `useX`, and UI files live in `*.tsx`.
- Rust: 4-space indent, `snake_case` for modules/functions and `CamelCase` for types; run `cargo fmt` before pushing.
- Tailwind CSS is used in JSX; keep class ordering consistent with nearby files.

## Testing Guidelines
- Backend: run `cargo test` in `src-tauri/` for Rust tests.
- API checks: use `cargo run --bin test_api` when touching ASR integrations.
- Frontend: no dedicated JS test runner yet; smoke-test via `npm run dev` and `npm run build`.

## Windows-Only & Architecture Notes
- This repo targets Windows 10/11 only; avoid cross-platform abstractions and `#[cfg(target_os = ...)]` branches unless required.
- Prefer Win32 APIs for hotkeys/input (GetAsyncKeyState, SendInput) and registry for auto-start.
- Global hotkeys require admin rights; preserve ghost-key detection and the 500ms watchdog when editing hotkey logic.
- Keep clipboard/focus timing safeguards (100ms delay before capture, 150ms delay before insert) in assistant/overlay flows.
- Config lives at `%APPDATA%\PushToTalk\config.json`; migration logic is in `src-tauri/src/config.rs`.

## Commit & Pull Request Guidelines
- Follow Conventional Commit-style prefixes seen in history: `feat:`, `fix:`, `perf:`, `refactor:`; short summaries can be Chinese or English.
- PRs should include a clear description, test steps, and screenshots for UI changes; link related issues when possible.
- Keep changes scoped and call out any Windows/admin-impacting behavior.

## Security & Configuration Tips
- Do not commit API keys or local config files.
- Auto-update uses NSIS; avoid reintroducing MSI or multi-instance installers.
- For deeper architecture details, see `CLAUDE.md`.
