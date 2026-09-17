use gpui::{
    App, Context, CursorStyle, Entity, FocusHandle, Focusable, FontWeight, MouseButton, Render,
    ResizeEdge, SharedString, Window, div, prelude::*, px,
};

use crate::editor::Editor;
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
}

const TITLEBAR_H: f32 = 38.0;
const RESIZE_EDGE: f32 = 6.0;

impl RootView {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let editor = cx.new(Editor::new);
        let explorer = cx.new(|cx| FileTree::new(editor.clone(), cx));
        let terminal = cx.new(Terminal::new);
        terminal.update(cx, |terminal, cx| terminal.start(cx));

        cx.observe(&editor, |_this, _editor, cx| cx.notify()).detach();

        Self {
            explorer,
            editor,
            terminal,
            focus_handle: cx.focus_handle(),
            settings_open: false,
            extensions_open: false,
        }
    }

    pub fn focus_editor(&self, window: &mut Window, cx: &App) {
        let handle = self.editor.read(cx).focus_handle(cx);
        window.focus(&handle);
    }

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

impl Render for RootView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let title = self.editor.read(cx).title();
        let accent = theme::accent();
        let current_mode = theme::current_mode();
        let current_accent = theme::current_accent();

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

                            // Settings button
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

                            // Extensions button
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

                            // Minimize
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
                                    .id("win-min")
                                    .on_click(|_, window, _cx| window.minimize_window())
                                    .child(
                                        div()
                                            .text_sm()
                                            .text_color(theme::color(theme::SUBTEXT))
                                            .child("\u{2013}"),
                                    ),
                            )

                            // Maximize
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

                            // Close
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
                            ),
                    ),
            )

            // === Main body ===
            .child(
                div()
                    .flex()
                    .flex_row()
                    .flex_1()
                    .min_h(px(0.0))
                    .child(
                        div()
                            .w(px(240.0))
                            .h_full()
                            .flex_none()
                            .child(self.explorer.clone()),
                    )
                    .child(div().flex_1().h_full().child(self.editor.clone())),
            )

            // === Terminal panel ===
            .child(div().h(px(220.0)).w_full().child(self.terminal.clone()))

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
