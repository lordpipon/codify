# Codify

A fast, minimal IDE built on **Rust + GPUI** — the same GPU-accelerated UI engine that powers Zed.

No Electron, no web tech. It's a native desktop app, built to be *yours*.

## Features

- **File explorer** — browse and open any directory (expand/collapse folders), with the project name and current **git branch** in the header
- **Tabbed editor** — multiple open files with dirty indicators, plus **back/forward** navigation and a breadcrumb path bar
- **Code editor** — cursor, selection, multiline editing, line-number gutter, mouse hit-testing
- **Status bar** — live cursor position (`line:col`), active language, and git branch
- **Syntax highlighting** for Rust, Python, JavaScript/TypeScript, C, C++, C#, Java, Go, Ruby, PHP, Swift, Kotlin, Lua, Perl, R, Scala, Haskell, Elixir, Dart, SQL, HTML/XML, Vue, Svelte (with `<script>`/`<style>` blocks), CSS, JSON, YAML, TOML, Shell, Dockerfile, Makefile, Markdown — and plain text
- **Extension store** — a Zed-style extension panel (▦ button in the title bar) with one-click install/uninstall. Languages are data-driven: installed extensions plug new languages straight into the tokenizer. Ships with Zig, Nim, Groovy, Crystal, Racket, Nix, and Julia
- **Integrated terminal** — a real PTY (via `portable-pty`) with a session header (`<project> — <shell>`) and pane controls (new / split / zoom / close), auto-detecting your shell:
  - Windows → PowerShell (`pwsh` → `powershell`) → `cmd`
  - macOS → `$SHELL` (zsh) → `/bin/zsh` → `/bin/bash`
  - Linux → `$SHELL` → `bash` → `sh`
- **Themes** — System / Light / Dark, plus **10 accent colours** (mauve, dark blue, green, red, orange, pink, light blue, yellow, white, black)
- **Settings persistence** — your theme survives restarts (`~/.config/codify/settings.toml`)
- **Extensions on disk** — installed extensions live in `~/.config/codify/extensions/*.json`
- **Native window controls** — custom title bar with minimize / maximize / close, drag-to-move, and resize handles (client-side decorations, like Zed)

## Keyboard shortcuts

| Shortcut        | Action            |
|-----------------|-------------------|
| `Ctrl+S`        | Save file         |
| `Ctrl+A/C/V/X`  | Select all / copy / paste / cut |
| `Ctrl+Enter`    | New line below    |
| `Ctrl+Q`        | Quit              |
| `Tab` / arrows  | Indent / navigate |
| `Shift+arrows`  | Extend selection  |

## Layout

```
┌ Codify ───────────────┬ tabs · breadcrumb ─────────┐
│ explorer + git branch │  editor                     │
│                       ├ terminal (can — zsh) + pane │
├───────────────────────┴─────────────────────────────┤
│ status: cursor 18:1 · C · main                      │
└─────────────────────────────────────────────────────┘
```

## Building from source

Requires Rust **1.85+** (edition 2024) and the usual native build deps of GPUI (C compiler, `clang` optional on Linux).

```sh
cargo run --release               # opens the current directory
cargo run --release -- /path/to   # or a specific project folder
```

## Where's my IDE?

The settings gear (⚙) in the title bar opens Appearance + Accent settings.
The puzzle-piece button (▦) opens the **Extension store**.
The window controls are in the top-right of the title bar.

## Installers

Prebuilt installers are produced by CI (`GitHub Actions`) on tagged releases:

- **macOS** — `.dmg`
- **Windows** — `.exe` (Inno Setup) / `.msi` (WiX)
- **Linux** — `.deb` and `.rpm`
- **Flatpak** — `org.codify.Codify` (Flathub build)
- **Arch** — `PKGBUILD` in `packaging/linux/` (build from source)
- **Sources** — alongside every release

No logos are embedded in any installer — just the name **Codify**.

## Packaging pieces

```
packaging/
  linux/          codify.desktop, PKGBUILD, codify.spec, flatpak manifest
  macos/          dmg build script
  windows/        WiX (msi) + Inno Setup (exe)
```

## How it works

```
src/
  main.rs         entry point, keybindings, window options
  app.rs          window shell: title bar, sidebar, editor, terminal, settings panel
  explorer.rs     file-tree view
  editor.rs       custom GPUI element: rendering, cursor, selection, input
  language.rs     tokenizer / syntax highlighting + dynamic-language hooks
  extensions.rs   extension store: catalog, install/uninstall, persistence
  terminal.rs     PTY terminal + ANSI handling
  theme.rs        colour system: light/dark/system + accents + persistence
```

## Author

**lordpipon**

## License

MIT