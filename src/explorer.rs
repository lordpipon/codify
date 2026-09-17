use std::path::PathBuf;

use gpui::{Context, Entity, FontWeight, Render, SharedString, Window, div, prelude::*, px};

use crate::editor::Editor;
use crate::theme;

struct FileNode {
    name: String,
    path: PathBuf,
    is_dir: bool,
    depth: usize,
    expanded: bool,
}

pub struct FileTree {
    root: Option<PathBuf>,
    nodes: Vec<FileNode>,
    selected: Option<PathBuf>,
    editor: Entity<Editor>,
}

fn read_children(path: &PathBuf, depth: usize) -> Vec<FileNode> {
    let mut entries: Vec<FileNode> = Vec::new();
    if let Ok(read_dir) = std::fs::read_dir(path) {
        for entry in read_dir.flatten() {
            let file_name = entry.file_name().to_string_lossy().to_string();
            if file_name.starts_with('.') || file_name == "node_modules" || file_name == "target" {
                continue;
            }
            let file_type = entry.file_type().ok();
            let is_dir = file_type.map(|t| t.is_dir()).unwrap_or(false);
            entries.push(FileNode {
                name: file_name,
                path: entry.path(),
                is_dir,
                depth,
                expanded: false,
            });
        }
    }
    entries.sort_by(|a, b| {
        if a.is_dir != b.is_dir {
            return if a.is_dir { std::cmp::Ordering::Less } else { std::cmp::Ordering::Greater };
        }
        a.name.to_lowercase().cmp(&b.name.to_lowercase())
    });
    entries
}

impl FileTree {
    pub fn new(editor: Entity<Editor>, cx: &mut Context<Self>) -> Self {
        let root = std::env::args()
            .nth(1)
            .map(PathBuf::from)
            .or_else(|| std::env::current_dir().ok());
        let mut this = Self {
            root: None,
            nodes: Vec::new(),
            selected: None,
            editor,
        };
        if let Some(root) = root {
            this.set_root(root, cx);
        }
        this
    }

    pub fn set_root(&mut self, root: PathBuf, cx: &mut Context<Self>) {
        self.nodes = read_children(&root, 0);
        self.root = Some(root);
        cx.notify();
    }

    pub fn refresh(&mut self, cx: &mut Context<Self>) {
        if let Some(root) = self.root.clone() {
            let selected = self.selected.clone();
            self.set_root(root, cx);
            self.selected = selected;
        }
    }

    fn activate(&mut self, index: usize, cx: &mut Context<Self>) {
        if index >= self.nodes.len() {
            return;
        }
        let is_dir = self.nodes[index].is_dir;
        let path = self.nodes[index].path.clone();
        if is_dir {
            if self.nodes[index].expanded {
                self.nodes[index].expanded = false;
                let depth = self.nodes[index].depth;
                let mut end = index + 1;
                while end < self.nodes.len() && self.nodes[end].depth > depth {
                    end += 1;
                }
                self.nodes.drain(index + 1..end);
            } else {
                let depth = self.nodes[index].depth;
                let children = read_children(&path, depth + 1);
                self.nodes[index].expanded = true;
                let mut insert_at = index + 1;
                for child in children {
                    self.nodes.insert(insert_at, child);
                    insert_at += 1;
                }
            }
            cx.notify();
        } else {
            self.selected = Some(path.clone());
            self.editor
                .update(cx, |editor, cx| editor.open_file(path, cx));
        }
    }
}

impl Render for FileTree {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let root_label = self
            .root
            .as_ref()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            .unwrap_or("No folder open")
            .to_uppercase();

        let rows: Vec<_> = self
            .nodes
            .iter()
            .enumerate()
            .map(|(index, node)| {
                let is_selected = self.selected.as_ref() == Some(&node.path);
                let indent = node.depth as f32 * 12.0;
                let glyph = if node.is_dir {
                    if node.expanded {
                        "\u{25be}"
                    } else {
                        "\u{25b8}"
                    }
                } else {
                    "\u{2022}"
                };
                let name_color = if node.is_dir { theme::TEXT } else { theme::SUBTEXT };
                let glyph_color = if node.is_dir { theme::BLUE } else { theme::OVERLAY };

                div()
                    .id(("file-row", index))
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap_1p5()
                    .h(px(22.0))
                    .w_full()
                    .pl(px(8.0 + indent))
                    .pr(px(8.0))
                    .rounded_sm()
                    .text_sm()
                    .cursor_pointer()
                    .when(is_selected, |el| el.bg(theme::color(theme::SURFACE)))
                    .hover(|style| style.bg(theme::color(theme::BG_DARKER)))
                    .child(
                        div()
                            .w(px(12.0))
                            .text_color(theme::color(glyph_color))
                            .child(glyph),
                    )
                    .child(
                        div()
                            .text_color(theme::color(name_color))
                            .child(SharedString::from(node.name.clone())),
                    )
                    .on_click(cx.listener(move |this, _event, _window, cx| {
                        this.activate(index, cx);
                    }))
            })
            .collect();

        div()
            .flex()
            .flex_col()
            .size_full()
            .bg(theme::color(theme::BG_DARK))
            .border_r_1()
            .border_color(theme::color(theme::BG_DARKER))
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .justify_between()
                    .px_3()
                    .py_2()
                    .border_b_1()
                    .border_color(theme::color(theme::BG_DARKER))
                    .text_xs()
                    .font_weight(FontWeight::BOLD)
                    .text_color(theme::color(theme::FAINT))
                    .child(SharedString::from(root_label))
                    .child(
                        div()
                            .id("refresh-btn")
                            .px_1()
                            .rounded_sm()
                            .cursor_pointer()
                            .hover(|style| style.bg(theme::color(theme::SURFACE)))
                            .child("\u{21bb}")
                            .on_click(cx.listener(|this, _event, _window, cx| {
                                this.refresh(cx);
                            })),
                    ),
            )
            .child(
                div()
                    .id("file-list")
                    .flex()
                    .flex_col()
                    .flex_1()
                    .p_1()
                    .overflow_y_scroll()
                    .children(rows),
            )
    }
}
