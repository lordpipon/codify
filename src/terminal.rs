use std::io::{Read, Write};
use std::path::PathBuf;

use gpui::{
    App, AsyncApp, Context, FocusHandle, Focusable, KeyDownEvent, MouseButton, MouseDownEvent,
    Render, SharedString, Window, div, prelude::*, px,
};
use portable_pty::{Child, CommandBuilder, MasterPty, PtySize, native_pty_system};

use crate::theme;

#[derive(Clone, Copy, PartialEq, Eq)]
enum EscState {
    Normal,
    Escape,
    Csi,
    Osc,
}

pub struct Terminal {
    focus_handle: FocusHandle,
    lines: Vec<String>,
    current: String,
    writer: Option<Box<dyn Write + Send>>,
    _master: Option<Box<dyn MasterPty + Send>>,
    child: Option<Box<dyn Child + Send + Sync>>,
    _task: Option<gpui::Task<()>>,
    _reader: Option<std::thread::JoinHandle<()>>,
    started: bool,
    pending: Vec<u8>,
    esc: EscState,
    cwd: Option<PathBuf>,
    shell_name: String,
}

impl Terminal {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let cwd = std::env::args()
            .nth(1)
            .map(PathBuf::from)
            .or_else(|| std::env::current_dir().ok());
        Self {
            focus_handle: cx.focus_handle(),
            lines: Vec::new(),
            current: String::new(),
            writer: None,
            _master: None,
            child: None,
            _task: None,
            _reader: None,
            started: false,
            pending: Vec::new(),
            esc: EscState::Normal,
            cwd,
            shell_name: String::new(),
        }
    }

    pub fn start(&mut self, cx: &mut Context<Self>) {
        if self.started {
            return;
        }
        self.started = true;

        let pty_system = native_pty_system();
        let pair = match pty_system.openpty(PtySize {
            rows: 24,
            cols: 100,
            pixel_width: 0,
            pixel_height: 0,
        }) {
            Ok(pair) => pair,
            Err(err) => {
                self.push_str(&format!("failed to open pty: {}\r\n", err));
                return;
            }
        };

        let shell = default_shell();
        self.shell_name = shell_display_name(&shell);
        let mut cmd = CommandBuilder::new(&shell);
        if let Some(cwd) = &self.cwd {
            cmd.cwd(cwd);
        }
        let child = match pair.slave.spawn_command(cmd) {
            Ok(child) => child,
            Err(err) => {
                self.push_str(&format!("failed to spawn shell: {}\r\n", err));
                return;
            }
        };
        drop(pair.slave);

        let mut reader = match pair.master.try_clone_reader() {
            Ok(reader) => reader,
            Err(err) => {
                self.push_str(&format!("failed to read pty: {}\r\n", err));
                return;
            }
        };
        let writer = match pair.master.take_writer() {
            Ok(writer) => writer,
            Err(err) => {
                self.push_str(&format!("failed to write pty: {}\r\n", err));
                return;
            }
        };

        self.writer = Some(writer);
        self.child = Some(child);
        self._master = Some(pair.master);

        let (tx, rx) = flume::unbounded::<Vec<u8>>();
        self._reader = Some(std::thread::spawn(move || {
            let mut buf = [0u8; 8192];
            loop {
                match reader.read(&mut buf) {
                    Ok(0) => break,
                    Ok(n) => {
                        if tx.send(buf[..n].to_vec()).is_err() {
                            break;
                        }
                    }
                    Err(_) => break,
                }
            }
        }));

        self._task = Some(cx.spawn(async move |this, cx: &mut AsyncApp| {
            while let Ok(data) = rx.recv_async().await {
                this.update(cx, |term, cx| {
                    term.process_bytes(&data);
                    cx.notify();
                })
                .ok();
            }
        }));
    }

    fn push_str(&mut self, text: &str) {
        for c in text.chars() {
            self.feed(c);
        }
    }

    fn process_bytes(&mut self, data: &[u8]) {
        self.pending.extend_from_slice(data);
        let consumed: usize;
        let mut chars: Vec<char> = Vec::new();
        {
            let pending = &self.pending;
            match std::str::from_utf8(pending) {
                Ok(s) => {
                    chars.extend(s.chars());
                    consumed = pending.len();
                }
                Err(err) => {
                    let valid = err.valid_up_to();
                    if let Ok(s) = std::str::from_utf8(&pending[..valid]) {
                        chars.extend(s.chars());
                    }
                    match err.error_len() {
                        Some(n) => consumed = valid + n,
                        None => consumed = valid,
                    }
                }
            }
        }
        self.pending.drain(..consumed);
        for c in chars {
            self.feed(c);
        }
        if self.lines.len() > 4000 {
            self.lines.drain(..2000);
        }
    }

    fn feed(&mut self, c: char) {
        match self.esc {
            EscState::Normal => match c {
                '\x1b' => self.esc = EscState::Escape,
                '\n' => {
                    let line = std::mem::take(&mut self.current);
                    self.lines.push(line);
                }
                '\r' => self.current.clear(),
                '\x08' => {
                    self.current.pop();
                }
                '\t' => self.current.push_str("    "),
                c if (c as u32) < 0x20 => {}
                c => self.current.push(c),
            },
            EscState::Escape => match c {
                '[' => self.esc = EscState::Csi,
                ']' => self.esc = EscState::Osc,
                _ => self.esc = EscState::Normal,
            },
            EscState::Csi => {
                let b = c as u32;
                if (0x40..=0x7e).contains(&b) {
                    self.esc = EscState::Normal;
                }
            }
            EscState::Osc => {
                if c == '\x07' {
                    self.esc = EscState::Normal;
                }
            }
        }
    }

    fn send(&mut self, data: &str) {
        if let Some(writer) = self.writer.as_mut() {
            let _ = writer.write_all(data.as_bytes());
            let _ = writer.flush();
        }
    }

    fn on_key_down(&mut self, event: &KeyDownEvent, _window: &mut Window, cx: &mut Context<Self>) {
        let keystroke = &event.keystroke;
        let ctrl = keystroke.modifiers.control;
        let bytes: Option<String> = match keystroke.key.as_str() {
            "enter" => Some("\r".to_string()),
            "backspace" => Some("\x7f".to_string()),
            "tab" => Some("\t".to_string()),
            "escape" => Some("\x1b".to_string()),
            "up" => Some("\x1b[A".to_string()),
            "down" => Some("\x1b[B".to_string()),
            "right" => Some("\x1b[C".to_string()),
            "left" => Some("\x1b[D".to_string()),
            "home" => Some("\x1b[H".to_string()),
            "end" => Some("\x1b[F".to_string()),
            "delete" => Some("\x1b[3~".to_string()),
            key => {
                if ctrl {
                    if key.len() == 1 {
                        let ch = key.chars().next().unwrap();
                        if ch.is_ascii_alphabetic() {
                            Some(((ch.to_ascii_lowercase() as u8 - b'a' + 1) as char).to_string())
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                } else {
                    keystroke.key_char.clone()
                }
            }
        };
        if let Some(bytes) = bytes {
            self.send(&bytes);
        }
        cx.notify();
    }

    fn on_mouse_down(
        &mut self,
        _event: &MouseDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.focus_handle.focus(window);
        cx.notify();
    }
}

impl Focusable for Terminal {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for Terminal {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let focused = self.focus_handle.is_focused(window);
        let visible = 80usize;
        let total = self.lines.len() + 1;
        let start = total.saturating_sub(visible);

        let mut rows: Vec<SharedString> = self.lines[start.min(self.lines.len())..]
            .iter()
            .map(|l| SharedString::from(l.clone()))
            .collect();
        rows.push(SharedString::from(self.current.clone()));

        div()
            .flex()
            .flex_col()
            .size_full()
            .bg(theme::color(theme::BG_DARKER))
            .border_t_1()
            .border_color(if focused {
                theme::color(theme::BLUE)
            } else {
                theme::color(theme::BG)
            })
            .track_focus(&self.focus_handle(cx))
            .on_key_down(cx.listener(Self::on_key_down))
            .on_mouse_down(MouseButton::Left, cx.listener(Self::on_mouse_down))
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap_2()
                    .px_3()
                    .h(px(24.0))
                    .child(
                        div()
                            .text_xs()
                            .text_color(theme::color(theme::FAINT))
                            .child(SharedString::from(format!(
                                "TERMINAL  \u{2022}  {}",
                                if self.shell_name.is_empty() {
                                    "shell"
                                } else {
                                    self.shell_name.as_str()
                                }
                            ))),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(theme::color(theme::GREEN))
                            .child(SharedString::from(if self.child.is_some() { "\u{25cf}" } else { "\u{25cb}" })),
                    ),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .flex_1()
                    .justify_end()
                    .px_3()
                    .py_1()
                    .overflow_hidden()
                    .text_size(px(13.0))
                    .line_height(px(18.0))
                    .children(rows.into_iter().map(|line| {
                        div()
                            .h(px(18.0))
                            .whitespace_nowrap()
                            .text_color(theme::color(theme::SUBTEXT))
                            .child(line)
                    })),
            )
    }
}

/// Find an executable on the `PATH`.
fn find_in_path(name: &str) -> Option<PathBuf> {
    let path_var = std::env::var_os("PATH")?;
    for dir in std::env::split_paths(&path_var) {
        let candidate = dir.join(name);
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    None
}

/// Pick the default shell for the host platform.
///
/// - Windows: PowerShell (`pwsh` or `powershell`), falling back to `cmd.exe`.
/// - macOS: the user's `$SHELL` (usually zsh), else `/bin/zsh` then `/bin/bash`.
/// - Linux/BSD: the user's `$SHELL`, else `bash`, then `sh`.
fn default_shell() -> String {
    #[cfg(not(target_os = "windows"))]
    {
        if let Ok(shell) = std::env::var("SHELL")
            && !shell.is_empty()
            && std::path::Path::new(&shell).exists()
        {
            return shell;
        }
    }

    #[cfg(target_os = "windows")]
    let candidates: &[&str] = &["pwsh.exe", "powershell.exe", "cmd.exe"];

    #[cfg(target_os = "macos")]
    let candidates: &[&str] = &["/bin/zsh", "/bin/bash", "/bin/sh"];

    #[cfg(all(unix, not(target_os = "macos")))]
    let candidates: &[&str] = &["/bin/bash", "/usr/bin/bash", "/bin/sh", "bash", "sh"];

    for candidate in candidates {
        if candidate.contains('/') || candidate.contains('\\') {
            if std::path::Path::new(candidate).exists() {
                return (*candidate).to_string();
            }
        } else if let Some(path) = find_in_path(candidate) {
            return path.to_string_lossy().to_string();
        }
    }

    #[cfg(target_os = "windows")]
    {
        "cmd.exe".to_string()
    }
    #[cfg(not(target_os = "windows"))]
    {
        "sh".to_string()
    }
}

/// A short, human-friendly label for a shell path, e.g. `/bin/zsh` -> `zsh`.
fn shell_display_name(shell: &str) -> String {
    std::path::Path::new(shell)
        .file_name()
        .and_then(|name| name.to_str())
        .map(|name| name.trim_end_matches(".exe").to_string())
        .unwrap_or_else(|| shell.to_string())
}
