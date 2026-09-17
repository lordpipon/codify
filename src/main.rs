mod app;
mod editor;
mod explorer;
mod extensions;
mod language;
mod terminal;
mod theme;

use gpui::{
    App, Application, Bounds, KeyBinding, SharedString, WindowBounds, WindowDecorations,
    WindowOptions, actions, prelude::*, px, size,
};

use app::RootView;

actions!(codify, [Quit]);

fn main() {
    Application::new().run(|cx: &mut App| {
        theme::init();
        extensions::init();

        cx.bind_keys([
            KeyBinding::new("backspace", editor::Backspace, Some("Editor")),
            KeyBinding::new("delete", editor::Delete, Some("Editor")),
            KeyBinding::new("left", editor::Left, Some("Editor")),
            KeyBinding::new("right", editor::Right, Some("Editor")),
            KeyBinding::new("up", editor::Up, Some("Editor")),
            KeyBinding::new("down", editor::Down, Some("Editor")),
            KeyBinding::new("shift-left", editor::SelectLeft, Some("Editor")),
            KeyBinding::new("shift-right", editor::SelectRight, Some("Editor")),
            KeyBinding::new("shift-up", editor::SelectUp, Some("Editor")),
            KeyBinding::new("shift-down", editor::SelectDown, Some("Editor")),
            KeyBinding::new("home", editor::Home, Some("Editor")),
            KeyBinding::new("end", editor::End, Some("Editor")),
            KeyBinding::new("enter", editor::Enter, Some("Editor")),
            KeyBinding::new("tab", editor::Indent, Some("Editor")),
            KeyBinding::new("ctrl-a", editor::SelectAll, Some("Editor")),
            KeyBinding::new("ctrl-c", editor::Copy, Some("Editor")),
            KeyBinding::new("ctrl-v", editor::Paste, Some("Editor")),
            KeyBinding::new("ctrl-x", editor::Cut, Some("Editor")),
            KeyBinding::new("ctrl-s", editor::Save, Some("Editor")),
            KeyBinding::new("ctrl-enter", editor::NewLineBelow, Some("Editor")),
            KeyBinding::new("ctrl-q", Quit, None),
        ]);

        cx.on_action(|_: &Quit, cx| cx.quit());

        let bounds = Bounds::centered(None, size(px(1400.0), px(900.0)), cx);
        let window = cx
            .open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(bounds)),
                    titlebar: Some(gpui::TitlebarOptions {
                        title: Some(SharedString::from("Codify")),
                        appears_transparent: true,
                        ..Default::default()
                    }),
                    window_decorations: Some(WindowDecorations::Client),
                    app_id: Some("org.codify.Codify".into()),
                    is_movable: true,
                    is_resizable: true,
                    is_minimizable: true,
                    window_min_size: Some(size(px(600.0), px(400.0))),
                    ..Default::default()
                },
                |window, cx| {
                    window.set_client_inset(px(TITLEBAR_H));
                    cx.new(RootView::new)
                },
            )
            .unwrap();

        window
            .update(cx, |root, window, cx| {
                root.focus_editor(window, cx);
            })
            .ok();

        cx.activate(true);
    });
}

const TITLEBAR_H: f32 = 38.0;
