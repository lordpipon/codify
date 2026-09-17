use std::path::PathBuf;

use gpui::{
    App, Context, CursorStyle, Entity, FocusHandle, Focusable, FontWeight, MouseButton, Render,
    ResizeEdge, SharedString, Window, div, prelude::*, px,
};

use crate::editor::{DocState, Editor};
use crate::explorer::FileTree;
use crate::extensions::Extension;
use crate::terminal::Terminal;
use crate::theme;

pub struct RootView {
    explorer: Entity<FileTree>,
    editor: Entity<Editor>,
    terminal: Entity<Terminal>,
    focus_handle: FocusHandle,
    settings_open: bool,
    extensions_open: bool,
    tabs: Vec<DocState>,
    active_tab: usize,
    last_open_id: u64,
    nav_history: Vec<PathBuf>,
    nav_pos: usize,
    terminal_open: bool,
    terminal_zoom: bool,
}

const TITLEBAR_H: f32 = 38.0;
const TABBAR_H: f32 = 34.0;
const BREADCRUMB_H: f32 = 24.0;
const STATUS_H: f32 = 24.0;
const RESIZE_EDGE: f32 = 6.0;
const TERMINAL_H: f32 = 220.0;
const TERMINAL_ZOOM_H: f32 = 520.0;

#[cfg(target_os = "macos")]
const IS_MACOS: bool = true;
#[cfg(not(target_os = "macos"))]
const IS_MACOS: bool = false;

#[cfg(target_os = "macos")]
const TRAFFIC_LIGHT_GUTTER: f32 = 78.0;
#[cfg(not(target_os = "macos"))]
const TRAFFIC_LIGHT_GUTTER: f32 = 0.0;

impl RootView {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let editor = cx.new(Editor::new);
        let explorer = cx.new(|cx| FileTree::new(editor.clone(), cx));
        let terminal = cx.new(Terminal::new);
        terminal.update(cx, |terminal, cx| terminal.start(cx));

        cx.observe(&editor, |this, editor, cx| {
            let (open_id, state) = {
                let editor = editor.read(cx);
                (editor.open_id(), editor.save_state())
            };
            if open_id != this.last_open_id {
                this.last_open_id = open_id;
                this.open_tab(state, cx);
            } else {
                if this.tabs.is_empty() {
                    this.tabs.push(DocState::empty());
                }
                if let Some(tab) = this.tabs.get_mut(this.active_tab) {
                    *tab = state;
                }
                cx.notify();
            }
        })
        .detach();

        Self {
            explorer,
            editor,
            terminal,
            focus_handle: cx.focus_handle(),
            settings_open: false,
            extensions_open: false,
            tabs: vec![DocState::empty()],
            active_tab: 0,
            last_open_id: 0,
            nav_history: Vec::new(),
            nav_pos: 0,
            terminal_open: true,
            terminal_zoom: false,
        }
    }

    pub fn focus_editor(&self, window: &mut Window, cx: &App) {
        let handle = self.editor.read(cx).focus_handle(cx);
        window.focus(&handle);
    }

    // ── Panels ────────────────────────────────────────────────────

    fn toggle_settings(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        self.settings_open = !self.settings_open;
        self.extensions_open = false;
        cx.notify();
    }

    fn close_settings(&mut self, cx: &mut Context<Self>) {
        self.settings_open = false;
        cx.notify();
    }

    fn toggle_extensions(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        self.extensions_open = !self.extensions_open;
        self.settings_open = false;
        cx.notify();
    }

    fn close_extensions(&mut self, cx: &mut Context<Self>) {
        self.extensions_open = false;
        cx.notify();
    }

    fn toggle_terminal(&mut self, cx: &mut Context<Self>) {
        self.terminal_open = !self.terminal_open;
        cx.notify();
    }

    fn toggle_terminal_zoom(&mut self, cx: &mut Context<Self>) {
        self.terminal_zoom = !self.terminal_zoom;
        cx.notify();
    }

    // ── Tabs & navigation ─────────────────────────────────────────

    /// Store the live editor buffer back into the active tab.
    fn sync_active(&mut self, cx: &App) {
        if self.tabs.is_empty() {
            self.tabs.push(DocState::empty());
            self.active_tab = 0;
        }
        let state = self.editor.read(cx).save_state();
        if let Some(tab) = self.tabs.get_mut(self.active_tab) {
            *tab = state;
        }
    }

    fn push_history(&mut self, path: PathBuf) {
        if self.nav_history.get(self.nav_pos) == Some(&path) {
            return;
        }
        self.nav_history.truncate(self.nav_pos + 1);
        self.nav_history.push(path);
        self.nav_pos = self.nav_history.len() - 1;
    }

    fn open_tab(&mut self, state: DocState, cx: &mut Context<Self>) {
        let path = state.path.clone();
        let existing = path
            .as_ref()
            .and_then(|p| self.tabs.iter().position(|t| t.path.as_ref() == Some(p)));

        match existing {
            Some(index) => {
                // Already open: activate the existing buffer, keeping its edits.
                // The editor currently holds the freshly-read file, so we must
                // NOT copy it back into the previously active tab.
                self.active_tab = index;
                let saved = self.tabs[index].clone();
                self.editor
                    .update(cx, |editor, cx| editor.load_state(saved, cx));
            }
            None => {
                // Replace the lone empty placeholder tab if present.
                if self.tabs.len() == 1
                    && self.tabs[0].path.is_none()
                    && self.tabs[0].text.is_empty()
                {
                    self.tabs[0] = state;
                    self.active_tab = 0;
                } else {
                    self.tabs.push(state);
                    self.active_tab = self.tabs.len() - 1;
                }
            }
        }

        if let Some(path) = path {
            self.push_history(path);
        }
        cx.notify();
    }

    fn switch_tab(&mut self, index: usize, cx: &mut Context<Self>) {
        if index >= self.tabs.len() || index == self.active_tab {
            return;
        }
        self.sync_active(cx);
        self.active_tab = index;
        let state = self.tabs[index].clone();
        self.editor
            .update(cx, |editor, cx| editor.load_state(state, cx));
        if let Some(path) = self.tabs[index].path.clone() {
            self.push_history(path);
        }
        cx.notify();
    }

    fn close_tab(&mut self, index: usize, cx: &mut Context<Self>) {
        if index >= self.tabs.len() {
            return;
        }
        self.sync_active(cx);
        self.tabs.remove(index);

        if self.tabs.is_empty() {
            self.tabs.push(DocState::empty());
            self.active_tab = 0;
            let blank = DocState::empty();
            self.editor
                .update(cx, |editor, cx| editor.load_state(blank, cx));
        } else {
            let active = if index <= self.active_tab {
                self.active_tab.saturating_sub(1)
            } else {
                self.active_tab
            };
            self.active_tab = active.min(self.tabs.len() - 1);
            let state = self.tabs[self.active_tab].clone();
            self.editor
                .update(cx, |editor, cx| editor.load_state(state, cx));
        }
        cx.notify();
    }

    fn activate_path(&mut self, path: PathBuf, cx: &mut Context<Self>) {
        if let Some(index) = self
            .tabs
            .iter()
            .position(|t| t.path.as_ref() == Some(&path))
        {
            if index != self.active_tab {
                self.sync_active(cx);
                self.active_tab = index;
                let state = self.tabs[index].clone();
                self.editor
                    .update(cx, |editor, cx| editor.load_state(state, cx));
            }
        }
        cx.notify();
    }

    fn go_back(&mut self, cx: &mut Context<Self>) {
        if self.nav_pos == 0 {
            return;
        }
        self.nav_pos -= 1;
        let path = self.nav_history[self.nav_pos].clone();
        self.activate_path(path, cx);
    }

    fn go_forward(&mut self, cx: &mut Context<Self>) {
        if self.nav_pos + 1 >= self.nav_history.len() {
            return;
        }
        self.nav_pos += 1;
        let path = self.nav_history[self.nav_pos].clone();
        self.activate_path(path, cx);
    }
}

impl Focusable for RootView {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

fn resize_edge(edge: ResizeEdge, cursor: CursorStyle) -> impl IntoElement {
    let base = div()
        .absolute()
        .cursor(cursor)
        .on_mouse_down(MouseButton::Left, move |_, window, _cx| {
            window.start_window_resize(edge);
        });
    match edge {
        ResizeEdge::Top => base.top_0().left_0().right_0().h(px(RESIZE_EDGE)).into_any_element(),
        ResizeEdge::Bottom => base.bottom_0().left_0().right_0().h(px(RESIZE_EDGE)).into_any_element(),
        ResizeEdge::Left => base.top_0().left_0().bottom_0().w(px(RESIZE_EDGE)).into_any_element(),
        ResizeEdge::Right => base.top_0().right_0().bottom_0().w(px(RESIZE_EDGE)).into_any_element(),
        _ => base.into_any_element(),
    }
}

/// A small icon button used across the chrome.
fn icon_button(
    id: impl Into<gpui::ElementId>,
    glyph: &'static str,
    enabled: bool,
    accent: gpui::Hsla,
) -> gpui::Stateful<gpui::Div> {
    div()
        .id(id)
        .w(px(24.0))
        .h(px(22.0))
        .flex()
        .items_center()
        .justify_center()
        .rounded_md()
        .text_xs()
        .cursor_pointer()
        .text_color(if enabled {
            theme::color(theme::SUBTEXT)
        } else {
            theme::color(theme::SURFACE_HI)
        })
        .hover(move |s| s.bg(theme::color(theme::SURFACE)))
        .child(glyph)
        .when(enabled, |el| el.hover(move |s| s.text_color(accent)))
}

fn breadcrumb_parts(root: Option<&PathBuf>, path: Option<&PathBuf>) -> Vec<String> {
    let Some(path) = path else {
        return Vec::new();
    };
    let rel = root
        .and_then(|root| path.strip_prefix(root).ok())
        .unwrap_or(path);
    rel.components()
        .filter_map(|c| c.as_os_str().to_str().map(|s| s.to_string()))
        .collect()
}

impl Render for RootView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let title = self.editor.read(cx).title();
        let accent = theme::accent();
        let current_mode = theme::current_mode();
        let current_accent = theme::current_accent();

        let root = self.explorer.read(cx).root();
        let root_name = self.explorer.read(cx).root_name();
        let shell = self.terminal.read(cx).shell_name().to_string();
        let branch = root.as_ref().and_then(|r| crate::explorer::git_branch(r));
        let (cursor_line, cursor_col) = self.editor.read(cx).cursor_line_col();
        let language = self.editor.read(cx).language_label();
        let active_path = self.tabs.get(self.active_tab).and_then(|t| t.path.clone());
        let crumbs = breadcrumb_parts(root.as_ref(), active_path.as_ref());

        let can_back = self.nav_pos > 0;
        let can_forward = self.nav_pos + 1 < self.nav_history.len();

        // ── Tabs ────────────────────────────────────────────────
        let tabs: Vec<_> = self
            .tabs
            .iter()
            .enumerate()
            .map(|(index, tab)| {
                let is_active = index == self.active_tab;
                let name = tab.file_name();
                let dirty = tab.dirty;
                let id = index;
                div()
                    .id(("tab", id))
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap_1()
                    .h(px(TABBAR_H - 6.0))
                    .px_3()
                    .rounded_md()
                    .cursor_pointer()
                    .bg(if is_active {
                        theme::color(theme::BG)
                    } else {
                        theme::color(theme::BG_DARKER)
                    })
                    .when(is_active, |el| el.border_t_1().border_color(accent))
                    .hover(|s| s.bg(theme::color(theme::SURFACE)))
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.switch_tab(id, cx);
                    }))
                    .child(
                        div()
                            .text_sm()
                            .text_color(if is_active {
                                theme::color(theme::TEXT)
                            } else {
                                theme::color(theme::FAINT)
                            })
                            .child(name),
                    )
                    .when(dirty, |el| {
                        el.child(
                            div()
                                .text_xs()
                                .text_color(accent)
                                .child("\u{25cf}"),
                        )
                    })
                    .child(
                        div()
                            .id(("tab-close", id))
                            .px_1()
                            .rounded_sm()
                            .text_xs()
                            .text_color(theme::color(theme::FAINT))
                            .hover(|s| s.bg(theme::color(theme::SURFACE_HI)))
                            .on_click(cx.listener(move |this, _, _, cx| {
                                cx.stop_propagation();
                                this.close_tab(id, cx);
                            }))
                            .child("\u{2715}"),
                    )
            })
            .collect();

        // ── Status bar ──────────────────────────────────────────
        let status_left = div()
            .flex()
            .flex_row()
            .items_center()
            .gap_1()
            .child(icon_button("status-gear", "\u{2699}", true, accent))
            .child(icon_button("status-term", "\u{25a3}", true, accent))
            .child(icon_button("status-search", "\u{2315}", true, accent))
            .child(icon_button("status-zoom", "\u{26f6}", true, accent))
            .child(icon_button("status-check", "\u{2713}", true, accent));

        let status_right = div()
            .flex()
            .flex_row()
            .items_center()
            .gap_3()
            .child(
                div()
                    .text_xs()
                    .text_color(theme::color(theme::SUBTEXT))
                    .child(SharedString::from(format!("{}:{}", cursor_line, cursor_col))),
            )
            .child(
                div()
                    .text_xs()
                    .text_color(theme::color(theme::SUBTEXT))
                    .child(language),
            )
            .when_some(branch, |el, branch| {
                el.child(
                    div()
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap_1()
                        .text_xs()
                        .text_color(theme::color(theme::SUBTEXT))
                        .child("\u{2387}")
                        .child(SharedString::from(branch)),
                )
            })
            .child(icon_button("status-debug", "\u{25b6}", true, accent))
            .child(icon_button("status-split", "\u{25eb}", true, accent));

        div()
            .relative()
            .flex()
            .flex_col()
            .size_full()
            .bg(theme::color(theme::BG))
            .font_family("DejaVu Sans Mono")

            // === Resize edges ===
            .child(resize_edge(ResizeEdge::Top, CursorStyle::ResizeUp))
            .child(resize_edge(ResizeEdge::Bottom, CursorStyle::ResizeDown))
            .child(resize_edge(ResizeEdge::Left, CursorStyle::ResizeLeft))
            .child(resize_edge(ResizeEdge::Right, CursorStyle::ResizeRight))

            // === Title bar (draggable) ===
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .justify_between()
                    .h(px(TITLEBAR_H))
                    .px_4()
                    .when(IS_MACOS, |d| d.pl(px(TRAFFIC_LIGHT_GUTTER)))
                    .bg(theme::color(theme::BG_DARKER))
                    .border_b_1()
                    .border_color(theme::color(theme::BG))
                    .on_mouse_down(MouseButton::Left, |event, window, _cx| {
                        if event.click_count == 2 {
                            window.zoom_window();
                        } else {
                            window.start_window_move();
                        }
                    })
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .gap_2()
                            .child(
                                div()
                                    .text_color(accent)
                                    .font_weight(FontWeight::BOLD)
                                    .child("Codify"),
                            )
                            .child(
                                div()
                                    .text_color(theme::color(theme::FAINT))
                                    .child(SharedString::from("\u{2502}")),
                            )
                            .child(
                                div()
                                    .text_color(theme::color(theme::SUBTEXT))
                                    .child(title),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .gap_1()
                            // Stop propagation so these don't trigger title-bar drag
                            .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
                            .on_mouse_down(MouseButton::Right, |_, _, cx| cx.stop_propagation())

                            // Search
                            .child(icon_button("qa-search", "\u{2315}", true, accent))

                            // Panel toggle
                            .child(
                                icon_button("qa-panel", "\u{25a4}", self.terminal_open, accent)
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.toggle_terminal(cx);
                                    })),
                            )

                            // Info
                            .child(icon_button("qa-info", "\u{24d8}", true, accent))

                            // Extensions
                            .child(
                                div()
                                    .id("extensions-btn")
                                    .px_2()
                                    .py_1()
                                    .rounded_md()
                                    .hover(|s| s.bg(theme::color(theme::SURFACE)))
                                    .cursor_pointer()
                                    .on_click(cx.listener(|this, _, window, cx| {
                                        this.toggle_extensions(window, cx);
                                    }))
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(theme::color(theme::FAINT))
                                            .child("\u{25a6}"),
                                    ),
                            )

                            // Settings
                            .child(
                                div()
                                    .id("settings-btn")
                                    .px_2()
                                    .py_1()
                                    .rounded_md()
                                    .hover(|s| s.bg(theme::color(theme::SURFACE)))
                                    .cursor_pointer()
                                    .on_click(cx.listener(|this, _, window, cx| {
                                        this.toggle_settings(window, cx);
                                    }))
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(theme::color(theme::FAINT))
                                            .child("\u{2699}"),
                                    ),
                            )

                            // Native traffic lights on macOS; custom controls elsewhere.
                            .when(!IS_MACOS, |bar| {
                                bar.child(
                                    div()
                                        .w(px(28.0))
                                        .h(px(24.0))
                                        .flex()
                                        .items_center()
                                        .justify_center()
                                        .rounded_md()
                                        .hover(|s| s.bg(theme::color(theme::SURFACE)))
                                        .cursor_pointer()
                                        .id("win-min")
                                        .on_click(|_, window, _cx| window.minimize_window())
                                        .child(
                                            div()
                                                .text_sm()
                                                .text_color(theme::color(theme::SUBTEXT))
                                                .child("\u{2013}"),
                                        ),
                                )
                                .child(
                                    div()
                                        .w(px(28.0))
                                        .h(px(24.0))
                                        .flex()
                                        .items_center()
                                        .justify_center()
                                        .rounded_md()
                                        .hover(|s| s.bg(theme::color(theme::SURFACE)))
                                        .cursor_pointer()
                                        .id("win-max")
                                        .on_click(|_, window, _cx| window.zoom_window())
                                        .child(
                                            div()
                                                .text_sm()
                                                .text_color(theme::color(theme::SUBTEXT))
                                                .child("\u{25a1}"),
                                        ),
                                )
                                .child(
                                    div()
                                        .w(px(28.0))
                                        .h(px(24.0))
                                        .flex()
                                        .items_center()
                                        .justify_center()
                                        .rounded_md()
                                        .hover(|s| s.bg(theme::color(theme::RED)))
                                        .cursor_pointer()
                                        .id("win-close")
                                        .on_click(|_, window, _cx| window.remove_window())
                                        .child(
                                            div()
                                                .text_sm()
                                                .text_color(theme::color(theme::SUBTEXT))
                                                .child("\u{2715}"),
                                        ),
                                )
                            }),
                    ),
            )

            // === Main body ===
            .child(
                div()
                    .flex()
                    .flex_row()
                    .flex_1()
                    .min_h(px(0.0))
                    // File explorer
                    .child(
                        div()
                            .w(px(240.0))
                            .h_full()
                            .flex_none()
                            .child(self.explorer.clone()),
                    )
                    // Editor column
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .flex_1()
                            .h_full()
                            .min_w(px(0.0))
                            // Tab bar with navigation arrows
                            .child(
                                div()
                                    .flex()
                                    .flex_row()
                                    .items_center()
                                    .gap_1()
                                    .h(px(TABBAR_H))
                                    .px_2()
                                    .bg(theme::color(theme::BG_DARKER))
                                    .border_b_1()
                                    .border_color(theme::color(theme::BG))
                                    .child(
                                        div()
                                            .id("nav-back")
                                            .w(px(24.0))
                                            .h(px(22.0))
                                            .flex()
                                            .items_center()
                                            .justify_center()
                                            .rounded_md()
                                            .cursor_pointer()
                                            .text_sm()
                                            .text_color(if can_back {
                                                theme::color(theme::SUBTEXT)
                                            } else {
                                                theme::color(theme::SURFACE_HI)
                                            })
                                            .hover(|s| s.bg(theme::color(theme::SURFACE)))
                                            .on_click(cx.listener(|this, _, _, cx| {
                                                this.go_back(cx);
                                            }))
                                            .child("\u{2190}"),
                                    )
                                    .child(
                                        div()
                                            .id("nav-forward")
                                            .w(px(24.0))
                                            .h(px(22.0))
                                            .flex()
                                            .items_center()
                                            .justify_center()
                                            .rounded_md()
                                            .cursor_pointer()
                                            .text_sm()
                                            .text_color(if can_forward {
                                                theme::color(theme::SUBTEXT)
                                            } else {
                                                theme::color(theme::SURFACE_HI)
                                            })
                                            .hover(|s| s.bg(theme::color(theme::SURFACE)))
                                            .on_click(cx.listener(|this, _, _, cx| {
                                                this.go_forward(cx);
                                            }))
                                            .child("\u{2192}"),
                                    )
                                    .child(
                                        div()
                                            .flex()
                                            .flex_row()
                                            .items_center()
                                            .gap_1()
                                            .flex_1()
                                            .min_w(px(0.0))
                                            .overflow_hidden()
                                            .children(tabs),
                                    ),
                            )
                            // Breadcrumb
                            .child(
                                div()
                                    .flex()
                                    .flex_row()
                                    .items_center()
                                    .gap_1()
                                    .h(px(BREADCRUMB_H))
                                    .px_3()
                                    .bg(theme::color(theme::BG))
                                    .border_b_1()
                                    .border_color(theme::color(theme::BG_DARKER))
                                    .text_xs()
                                    .text_color(theme::color(theme::FAINT))
                                    .children(
                                        crumbs.into_iter().enumerate().map(|(i, part)| {
                                            let sep = if i == 0 { "" } else { "\u{203a}" };
                                            div()
                                                .flex()
                                                .flex_row()
                                                .items_center()
                                                .gap_1()
                                                .when(i > 0, |el| el.child(sep))
                                                .child(SharedString::from(part))
                                        }),
                                    ),
                            )
                            // Editor
                            .child(div().flex_1().min_h(px(0.0)).child(self.editor.clone())),
                    ),
            )

            // === Terminal panel ===
            .when(self.terminal_open, |parent| {
                let terminal_h = if self.terminal_zoom {
                    TERMINAL_ZOOM_H
                } else {
                    TERMINAL_H
                };
                parent.child(
                    div()
                        .flex()
                        .flex_col()
                        .h(px(terminal_h))
                        .w_full()
                        .flex_none()
                        .bg(theme::color(theme::BG_DARKER))
                        .border_t_1()
                        .border_color(theme::color(theme::BG))
                        // Header: session tab + pane controls
                        .child(
                            div()
                                .flex()
                                .flex_row()
                                .items_center()
                                .justify_between()
                                .h(px(28.0))
                                .px_3()
                                .child(
                                    div()
                                        .flex()
                                        .flex_row()
                                        .items_center()
                                        .gap_2()
                                        .child(
                                            div()
                                                .px_2()
                                                .py_1()
                                                .rounded_sm()
                                                .bg(theme::color(theme::BG))
                                                .text_xs()
                                                .text_color(theme::color(theme::TEXT))
                                                .child({
                                                    let shell_label = if shell.is_empty() {
                                                        "shell".to_string()
                                                    } else {
                                                        shell.clone()
                                                    };
                                                    SharedString::from(format!(
                                                        "{} \u{2014} {}",
                                                        root_name, shell_label
                                                    ))
                                                }),
                                        )
                                        .child(
                                            div()
                                                .text_xs()
                                                .text_color(theme::color(theme::GREEN))
                                                .child("\u{25cf}"),
                                        ),
                                )
                                .child(
                                    div()
                                        .flex()
                                        .flex_row()
                                        .items_center()
                                        .gap_1()
                                        .child(icon_button("term-new", "+", true, accent))
                                        .child(icon_button("term-split", "\u{2731}", true, accent))
                                        .child(
                                            icon_button(
                                                "term-zoom",
                                                "\u{25eb}",
                                                self.terminal_zoom,
                                                accent,
                                            )
                                            .on_click(cx.listener(|this, _, _, cx| {
                                                this.toggle_terminal_zoom(cx);
                                            })),
                                        )
                                        .child(
                                            icon_button("term-close", "\u{2304}", true, accent)
                                                .on_click(cx.listener(|this, _, _, cx| {
                                                    this.toggle_terminal(cx);
                                                })),
                                        ),
                                ),
                        )
                        .child(div().flex_1().min_h(px(0.0)).child(self.terminal.clone())),
                )
            })

            // === Status bar ===
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .justify_between()
                    .h(px(STATUS_H))
                    .px_2()
                    .bg(theme::color(theme::BG_DARKER))
                    .border_t_1()
                    .border_color(theme::color(theme::BG))
                    .child(status_left)
                    .child(status_right),
            )

            // === Settings panel overlay (painted last, on top) ===
            .when(self.settings_open, |parent| {
                parent.child(
                    div()
                        .id("settings-overlay")
                        .absolute()
                        .top_0()
                        .left_0()
                        .right_0()
                        .bottom_0()
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.close_settings(cx);
                        }))
                        .child(
                            div()
                                .id("settings-panel")
                                .absolute()
                                .top(px(TITLEBAR_H + 12.0))
                                .right(px(16.0))
                                .w(px(340.0))
                                .rounded_lg()
                                .p_4()
                                .bg(theme::color(theme::BG_DARK))
                                .border_1()
                                .border_color(theme::color(theme::SURFACE))
                                .shadow_lg()
                                .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
                                .on_click(|_, _, cx| cx.stop_propagation())

                                // Appearance
                                .child(
                                    div()
                                        .mb_3()
                                        .child(
                                            div()
                                                .text_xs()
                                                .font_weight(FontWeight::BOLD)
                                                .text_color(theme::color(theme::SUBTEXT))
                                                .child("APPEARANCE"),
                                        )
                                        .child(
                                            div()
                                                .flex()
                                                .flex_row()
                                                .gap_2()
                                                .mt_2()
                                                .children(theme::ThemeMode::ALL.iter().map(|&mode| {
                                                    let is_active = mode == current_mode;
                                                    let label = mode.label();
                                                    div()
                                                        .px_3()
                                                        .py_1()
                                                        .rounded_md()
                                                        .border_1()
                                                        .border_color(if is_active {
                                                            accent
                                                        } else {
                                                            theme::color(theme::SURFACE_HI)
                                                        })
                                                        .bg(if is_active {
                                                            theme::color(theme::SURFACE)
                                                        } else {
                                                            theme::color(theme::BG)
                                                        })
                                                        .text_xs()
                                                        .text_color(if is_active {
                                                            theme::color(theme::TEXT)
                                                        } else {
                                                            theme::color(theme::FAINT)
                                                        })
                                                        .cursor_pointer()
                                                        .id(("theme-mode", mode as usize))
                                                        .on_click(cx.listener(
                                                            move |this, _, _, cx| {
                                                                theme::update(mode, current_accent);
                                                                this.close_settings(cx);
                                                            },
                                                        ))
                                                        .child(label)
                                                })),
                                        ),
                                )

                                // Accent colours
                                .child(
                                    div()
                                        .w_full()
                                        .child(
                                            div()
                                                .text_xs()
                                                .font_weight(FontWeight::BOLD)
                                                .text_color(theme::color(theme::SUBTEXT))
                                                .child("ACCENT"),
                                        )
                                        .child(
                                            div()
                                                .flex()
                                                .flex_row()
                                                .flex_wrap()
                                                .gap_2()
                                                .mt_2()
                                                .children(theme::Accent::ALL.iter().map(|&acc| {
                                                    let is_active = acc == current_accent;
                                                    div()
                                                        .w(px(36.0))
                                                        .h(px(24.0))
                                                        .rounded_md()
                                                        .border_2()
                                                        .border_color(if is_active {
                                                            accent
                                                        } else {
                                                            theme::color(theme::SURFACE_HI)
                                                        })
                                                        .bg(acc.swatch())
                                                        .cursor_pointer()
                                                        .id(("accent", acc as usize))
                                                        .on_click(cx.listener(
                                                            move |this, _, _, cx| {
                                                                theme::update(current_mode, acc);
                                                                this.close_settings(cx);
                                                            },
                                                        ))
                                                })),
                                        )
                                        .child(
                                            div()
                                                .mt_2()
                                                .text_xs()
                                                .text_color(theme::color(theme::FAINT))
                                                .child(SharedString::from(format!(
                                                    "{} \u{2022} {}",
                                                    current_mode.label(),
                                                    current_accent.label()
                                                ))),
                                        ),
                                ),
                        ),
                )
            })

            // === Extensions panel overlay (store) ===
            .when(self.extensions_open, |parent| {
                let installed = crate::extensions::installed();
                let this = cx.entity();
                parent.child(
                    div()
                        .id("extensions-overlay")
                        .absolute()
                        .top_0()
                        .left_0()
                        .right_0()
                        .bottom_0()
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.close_extensions(cx);
                        }))
                        .child(
                            div()
                                .id("extensions-panel")
                                .absolute()
                                .top(px(TITLEBAR_H + 12.0))
                                .right(px(16.0))
                                .w(px(460.0))
                                .max_h(px(560.0))
                                .overflow_y_scroll()
                                .rounded_lg()
                                .p_4()
                                .bg(theme::color(theme::BG_DARK))
                                .border_1()
                                .border_color(theme::color(theme::SURFACE))
                                .shadow_lg()
                                .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
                                .on_click(|_, _, cx| cx.stop_propagation())

                                .child(
                                    div()
                                        .mb_3()
                                        .child(
                                            div()
                                                .text_xs()
                                                .font_weight(FontWeight::BOLD)
                                                .text_color(theme::color(theme::SUBTEXT))
                                                .child("EXTENSIONS"),
                                        )
                                        .child(
                                            div()
                                                .text_xs()
                                                .text_color(theme::color(theme::FAINT))
                                                .child(
                                                    "Language extensions from the Codify store. \
                                                     Installed languages take effect on reopened files.",
                                                ),
                                        ),
                                )

                                .children(crate::extensions::catalog().iter().enumerate().map(|(i, ext)| {
                                    let is_installed =
                                        installed.iter().any(|e| e.id == ext.id);
                                    extension_row(ext, is_installed, accent, &this, i)
                                }))

                                .when(
                                    installed.iter().any(|e| {
                                        !crate::extensions::catalog()
                                            .iter()
                                            .any(|c| c.id == e.id)
                                    }),
                                    |panel| {
                                        panel.child(
                                            div()
                                                .mt_4()
                                                .mb_2()
                                                .child(
                                                    div()
                                                        .text_xs()
                                                        .font_weight(FontWeight::BOLD)
                                                        .text_color(theme::color(theme::SUBTEXT))
                                                        .child("INSTALLED"),
                                                ),
                                        )
                                    },
                                )

                                .children(
                                    installed.iter().enumerate().filter(|(_, e)| {
                                        !crate::extensions::catalog()
                                            .iter()
                                            .any(|c| c.id == e.id)
                                    }).map(|(i, ext)| extension_row(ext, true, accent, &this, i + crate::extensions::catalog().len())),
                                ),
                        ),
                )
            })
    }
}

/// A single store/catalog row with an Install/Uninstall action.
fn extension_row(
    ext: &Extension,
    is_installed: bool,
    accent: gpui::Hsla,
    this: &Entity<RootView>,
    index: usize,
) -> impl gpui::IntoElement {
    let ext_id = ext.id.clone();
    let this_entity = this.clone();
    let ext_name = ext.name.clone();
    let ext_version = ext.version.clone();
    let ext_desc = ext.description.clone();
    let ext_author = ext.author.clone();
    let action_label = if is_installed { "Uninstall" } else { "Install" };

    div()
        .flex()
        .flex_row()
        .items_center()
        .justify_between()
        .gap_2()
        .p_2()
        .rounded_md()
        .hover(|s| s.bg(theme::color(theme::SURFACE)))
        .child(
            div()
                .flex()
                .flex_col()
                .gap_0p5()
                .flex_1()
                .min_w(px(0.0))
                .child(
                    div()
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap_2()
                        .child(
                            div()
                                .text_xs()
                                .font_weight(FontWeight::BOLD)
                                .text_color(theme::color(theme::TEXT))
                                .child(SharedString::from(ext_name)),
                        )
                        .child(
                            div()
                                .text_xs()
                                .text_color(theme::color(theme::FAINT))
                                .child(SharedString::from(ext_version)),
                        ),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(theme::color(theme::FAINT))
                        .child(SharedString::from(ext_desc)),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(accent)
                        .child(SharedString::from(ext_author)),
                ),
        )
        .child(
            div()
                .flex_none()
                .px_2()
                .py_1()
                .rounded_md()
                .border_1()
                .border_color(if is_installed {
                    accent
                } else {
                    theme::color(theme::SURFACE_HI)
                })
                .text_xs()
                .text_color(if is_installed {
                    accent
                } else {
                    theme::color(theme::TEXT)
                })
                .cursor_pointer()
                .id(("ext-action", index))
                .on_click(move |_, _, app: &mut App| {
                    let id = ext_id.clone();
                    app.update_entity(&this_entity, |this, cx| {
                        if is_installed {
                            crate::extensions::uninstall(&id);
                        } else {
                            crate::extensions::install(&id);
                        }
                        this.extensions_open = true;
                        cx.notify();
                    });
                })
                .child(SharedString::from(action_label)),
        )
}
