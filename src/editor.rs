use std::ops::Range;
use std::path::PathBuf;

use gpui::{
    App, Bounds, ClipboardItem, Context, CursorStyle, Element, ElementId, ElementInputHandler,
    Entity, EntityInputHandler, FocusHandle, Focusable, GlobalElementId, Hsla, LayoutId,
    MouseButton, MouseDownEvent, MouseMoveEvent, MouseUpEvent, PaintQuad, Pixels, Point,
    ShapedLine, SharedString, Style, TextRun, UTF16Selection, Window, actions,
    div, fill, point, prelude::*, px, relative, size,
};

use crate::language::{Language, TokenKind, HighlightState, language_for_path, tokenize_line};
use crate::theme;

const FONT_SIZE: f32 = 14.0;
const LINE_HEIGHT: f32 = 20.0;
const GUTTER_PAD: f32 = 12.0;
const CODE_PAD: f32 = 8.0;

actions!(
    editor,
    [
        Backspace,
        Delete,
        Left,
        Right,
        Up,
        Down,
        Home,
        End,
        Enter,
        Indent,
        SelectLeft,
        SelectRight,
        SelectUp,
        SelectDown,
        SelectAll,
        Paste,
        Cut,
        Copy,
        Save,
        NewLineBelow,
    ]
);

fn token_color(kind: TokenKind) -> Hsla {
    match kind {
        TokenKind::Plain => theme::color(theme::TEXT),
        TokenKind::Keyword => theme::color(theme::MAUVE),
        TokenKind::Type => theme::color(theme::YELLOW),
        TokenKind::Function => theme::color(theme::BLUE),
        TokenKind::Constant => theme::color(theme::PEACH),
        TokenKind::Str => theme::color(theme::GREEN),
        TokenKind::Comment => theme::color(theme::OVERLAY),
        TokenKind::Number => theme::color(theme::PEACH),
        TokenKind::Operator => theme::color(theme::SKY),
        TokenKind::Punctuation => theme::color(theme::SUBTEXT),
        TokenKind::Attribute => theme::color(theme::TEAL),
    }
}

fn line_starts(text: &str) -> Vec<usize> {
    let mut starts = vec![0usize];
    for (i, b) in text.bytes().enumerate() {
        if b == b'\n' {
            starts.push(i + 1);
        }
    }
    starts
}

fn line_index_for(starts: &[usize], offset: usize) -> usize {
    match starts.binary_search(&offset) {
        Ok(i) => i,
        Err(i) => i.saturating_sub(1),
    }
}

fn line_end(text: &str, starts: &[usize], idx: usize) -> usize {
    let end = starts.get(idx + 1).copied().unwrap_or(text.len());
    if end > starts[idx] && text.as_bytes().get(end - 1) == Some(&b'\n') {
        end - 1
    } else {
        end
    }
}

fn prev_boundary(text: &str, offset: usize) -> usize {
    if offset == 0 {
        return 0;
    }
    let mut i = offset - 1;
    while i > 0 && !text.is_char_boundary(i) {
        i -= 1;
    }
    i
}

fn next_boundary(text: &str, offset: usize) -> usize {
    if offset >= text.len() {
        return text.len();
    }
    let mut i = offset + 1;
    while i < text.len() && !text.is_char_boundary(i) {
        i += 1;
    }
    i
}

struct HitLine {
    line: usize,
    start: usize,
    shaped: ShapedLine,
}

struct HitTest {
    bounds: Bounds<Pixels>,
    scroll: usize,
    gutter: Pixels,
    lines: Vec<HitLine>,
}

pub struct Editor {
    focus_handle: FocusHandle,
    path: Option<PathBuf>,
    text: String,
    language: Language,
    dynamic_id: Option<String>,
    selected_range: Range<usize>,
    selection_reversed: bool,
    marked_range: Option<Range<usize>>,
    scroll: usize,
    dirty: bool,
    is_selecting: bool,
    hit: Option<HitTest>,
    status: SharedString,
}

impl Editor {
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self {
            focus_handle: cx.focus_handle(),
            path: None,
            text: String::new(),
            language: Language::Plain,
            dynamic_id: None,
            selected_range: 0..0,
            selection_reversed: false,
            marked_range: None,
            scroll: 0,
            dirty: false,
            is_selecting: false,
            hit: None,
            status: SharedString::from(""),
        }
    }

    pub fn title(&self) -> SharedString {
        let name = self
            .path
            .as_ref()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            .unwrap_or("untitled");
        if self.dirty {
            SharedString::from(format!("{} \u{2022}", name))
        } else {
            SharedString::from(name.to_string())
        }
    }

    pub fn open_file(&mut self, path: PathBuf, cx: &mut Context<Self>) {
        match std::fs::read_to_string(&path) {
            Ok(contents) => {
                if let Some(dyn_lang) = crate::extensions::dynamic_language_for_path(&path) {
                    self.language = Language::Custom;
                    self.dynamic_id = Some(dyn_lang.id);
                } else {
                    self.language = language_for_path(&path);
                    self.dynamic_id = None;
                }
                self.text = contents;
                self.path = Some(path);
                self.selected_range = 0..0;
                self.selection_reversed = false;
                self.scroll = 0;
                self.dirty = false;
                self.status = SharedString::from("");
            }
            Err(err) => {
                self.status = SharedString::from(format!("Could not open file: {}", err));
            }
        }
        cx.notify();
    }

    fn cursor(&self) -> usize {
        if self.selection_reversed {
            self.selected_range.start
        } else {
            self.selected_range.end
        }
    }

    fn selection_bounds(&self) -> Range<usize> {
        let start = self.selected_range.start.min(self.selected_range.end);
        let end = self.selected_range.start.max(self.selected_range.end);
        start..end
    }

    fn move_to(&mut self, offset: usize, cx: &mut Context<Self>) {
        let offset = offset.min(self.text.len());
        self.selected_range = offset..offset;
        self.selection_reversed = false;
        self.ensure_cursor_visible();
        cx.notify();
    }

    fn select_to(&mut self, offset: usize, cx: &mut Context<Self>) {
        let offset = offset.min(self.text.len());
        if self.selection_reversed {
            self.selected_range.start = offset;
        } else {
            self.selected_range.end = offset;
        }
        if self.selected_range.end < self.selected_range.start {
            self.selection_reversed = !self.selection_reversed;
            self.selected_range = self.selected_range.end..self.selected_range.start;
        }
        self.ensure_cursor_visible();
        cx.notify();
    }

    fn ensure_cursor_visible(&mut self) {
        let starts = line_starts(&self.text);
        let line = line_index_for(&starts, self.cursor());
        if line < self.scroll {
            self.scroll = line;
        }
    }

    fn left(&mut self, _: &Left, _: &mut Window, cx: &mut Context<Self>) {
        if self.selected_range.is_empty() {
            let target = prev_boundary(&self.text, self.cursor());
            self.move_to(target, cx);
        } else {
            let start = self.selection_bounds().start;
            self.move_to(start, cx);
        }
    }

    fn right(&mut self, _: &Right, _: &mut Window, cx: &mut Context<Self>) {
        if self.selected_range.is_empty() {
            let target = next_boundary(&self.text, self.cursor());
            self.move_to(target, cx);
        } else {
            let end = self.selection_bounds().end;
            self.move_to(end, cx);
        }
    }

    fn up(&mut self, _: &Up, _: &mut Window, cx: &mut Context<Self>) {
        self.move_vertical(-1, cx);
    }

    fn down(&mut self, _: &Down, _: &mut Window, cx: &mut Context<Self>) {
        self.move_vertical(1, cx);
    }

    fn select_up(&mut self, _: &SelectUp, _: &mut Window, cx: &mut Context<Self>) {
        self.select_vertical(-1, cx);
    }

    fn select_down(&mut self, _: &SelectDown, _: &mut Window, cx: &mut Context<Self>) {
        self.select_vertical(1, cx);
    }

    fn select_left(&mut self, _: &SelectLeft, _: &mut Window, cx: &mut Context<Self>) {
        let target = prev_boundary(&self.text, self.cursor());
        self.select_to(target, cx);
    }

    fn select_right(&mut self, _: &SelectRight, _: &mut Window, cx: &mut Context<Self>) {
        let target = next_boundary(&self.text, self.cursor());
        self.select_to(target, cx);
    }

    fn select_all(&mut self, _: &SelectAll, _: &mut Window, cx: &mut Context<Self>) {
        self.selected_range = 0..self.text.len();
        self.selection_reversed = false;
        cx.notify();
    }

    fn home(&mut self, _: &Home, _: &mut Window, cx: &mut Context<Self>) {
        let starts = line_starts(&self.text);
        let line = line_index_for(&starts, self.cursor());
        self.move_to(starts[line], cx);
    }

    fn end(&mut self, _: &End, _: &mut Window, cx: &mut Context<Self>) {
        let starts = line_starts(&self.text);
        let line = line_index_for(&starts, self.cursor());
        let end = line_end(&self.text, &starts, line);
        self.move_to(end, cx);
    }

    fn move_vertical(&mut self, delta: i64, cx: &mut Context<Self>) {
        let starts = line_starts(&self.text);
        let cur = self.cursor();
        let line = line_index_for(&starts, cur);
        let col = self.text[starts[line]..cur].chars().count();
        let target = (line as i64 + delta).clamp(0, starts.len() as i64 - 1) as usize;
        let target_start = starts[target];
        let target_end = line_end(&self.text, &starts, target);
        let line_str = &self.text[target_start..target_end];
        let new_start = target_start
            + line_str
                .char_indices()
                .nth(col)
                .map(|(i, _)| i)
                .unwrap_or(line_str.len());
        self.move_to(new_start, cx);
    }

    fn select_vertical(&mut self, delta: i64, cx: &mut Context<Self>) {
        let starts = line_starts(&self.text);
        let cur = self.cursor();
        let line = line_index_for(&starts, cur);
        let col = self.text[starts[line]..cur].chars().count();
        let target = (line as i64 + delta).clamp(0, starts.len() as i64 - 1) as usize;
        let target_start = starts[target];
        let target_end = line_end(&self.text, &starts, target);
        let line_str = &self.text[target_start..target_end];
        let new_start = target_start
            + line_str
                .char_indices()
                .nth(col)
                .map(|(i, _)| i)
                .unwrap_or(line_str.len());
        self.select_to(new_start, cx);
    }

    fn backspace(&mut self, _: &Backspace, window: &mut Window, cx: &mut Context<Self>) {
        if self.selected_range.is_empty() {
            let target = prev_boundary(&self.text, self.cursor());
            self.selected_range = target..self.cursor();
            self.selection_reversed = false;
        }
        self.replace_text_in_range(None, "", window, cx);
    }

    fn delete(&mut self, _: &Delete, window: &mut Window, cx: &mut Context<Self>) {
        if self.selected_range.is_empty() {
            let target = next_boundary(&self.text, self.cursor());
            self.selected_range = self.cursor()..target;
            self.selection_reversed = false;
        }
        self.replace_text_in_range(None, "", window, cx);
    }

    fn enter(&mut self, _: &Enter, window: &mut Window, cx: &mut Context<Self>) {
        let starts = line_starts(&self.text);
        let line = line_index_for(&starts, self.cursor());
        let line_start = starts[line];
        let indent: String = self.text[line_start..]
            .chars()
            .take_while(|c| *c == ' ' || *c == '\t')
            .collect();
        self.replace_text_in_range(None, &format!("\n{}", indent), window, cx);
    }

    fn new_line_below(&mut self, _: &NewLineBelow, _: &mut Window, cx: &mut Context<Self>) {
        let starts = line_starts(&self.text);
        let line = line_index_for(&starts, self.cursor());
        let end = line_end(&self.text, &starts, line);
        self.move_to(end, cx);
        self.text.insert(end, '\n');
        self.move_to(end + 1, cx);
        self.dirty = true;
    }

    fn indent(&mut self, _: &Indent, window: &mut Window, cx: &mut Context<Self>) {
        let selection = self.selection_bounds();
        if selection.is_empty() {
            self.replace_text_in_range(None, "    ", window, cx);
        } else {
            let starts = line_starts(&self.text);
            let first = line_index_for(&starts, selection.start);
            let last = line_index_for(&starts, selection.end);
            let mut insert_at: Vec<usize> = Vec::new();
            for line in first..=last {
                insert_at.push(starts[line]);
            }
            for (offset, pos) in insert_at.iter().enumerate() {
                self.text.insert_str(pos + offset * 4, "    ");
            }
            self.selected_range = selection.start..selection.end + 4 * (last - first + 1);
            self.selection_reversed = false;
            self.dirty = true;
            cx.notify();
        }
    }

    fn paste(&mut self, _: &Paste, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(text) = cx.read_from_clipboard().and_then(|item| item.text()) {
            self.replace_text_in_range(None, &text, window, cx);
        }
    }

    fn copy(&mut self, _: &Copy, _: &mut Window, cx: &mut Context<Self>) {
        let range = self.selection_bounds();
        if !range.is_empty() {
            cx.write_to_clipboard(ClipboardItem::new_string(self.text[range].to_string()));
        }
    }

    fn cut(&mut self, _: &Cut, window: &mut Window, cx: &mut Context<Self>) {
        let range = self.selection_bounds();
        if !range.is_empty() {
            cx.write_to_clipboard(ClipboardItem::new_string(self.text[range].to_string()));
            self.replace_text_in_range(None, "", window, cx);
        }
    }

    fn save(&mut self, _: &Save, _: &mut Window, cx: &mut Context<Self>) {
        if let Some(path) = self.path.clone() {
            match std::fs::write(&path, &self.text) {
                Ok(_) => {
                    self.dirty = false;
                    self.status = SharedString::from("saved");
                }
                Err(err) => self.status = SharedString::from(format!("save failed: {}", err)),
            }
            cx.notify();
        }
    }

    fn on_mouse_down(
        &mut self,
        event: &MouseDownEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.is_selecting = true;
        let offset = self.index_for_position(event.position);
        if event.modifiers.shift {
            self.select_to(offset, cx);
        } else {
            self.move_to(offset, cx);
        }
    }

    fn on_mouse_up(&mut self, _: &MouseUpEvent, _: &mut Window, _: &mut Context<Self>) {
        self.is_selecting = false;
    }

    fn on_mouse_move(&mut self, event: &MouseMoveEvent, _: &mut Window, cx: &mut Context<Self>) {
        if self.is_selecting {
            let offset = self.index_for_position(event.position);
            self.select_to(offset, cx);
        }
    }

    fn on_scroll_wheel(
        &mut self,
        event: &gpui::ScrollWheelEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let lines = line_starts(&self.text).len();
        let delta = event.delta.pixel_delta(px(LINE_HEIGHT)).y / px(1.0);
        let delta = if delta.abs() < 0.01 { 0.0 } else { delta };
        let step = if delta > 0.0 { 1 } else { -1 };
        let _ = delta.abs();
        let max_scroll = lines.saturating_sub(1);
        let next = (self.scroll as i64 + step).clamp(0, max_scroll as i64) as usize;
        if next != self.scroll {
            self.scroll = next;
            cx.notify();
        }
    }

    fn index_for_position(&self, position: Point<Pixels>) -> usize {
        let Some(hit) = self.hit.as_ref() else {
            return self.cursor();
        };
        if hit.lines.is_empty() {
            return 0;
        }
        let local_y = position.y - hit.bounds.origin.y;
        let row_val = (local_y / px(LINE_HEIGHT)).max(0.0);
        let row = row_val as usize;
        let idx = (hit.scroll + row).min(hit.lines.len().saturating_sub(1));
        let hit_line = &hit.lines[idx];
        let local_x = position.x - hit.bounds.origin.x - hit.gutter - px(CODE_PAD);
        let byte_in_line = hit_line
            .shaped
            .index_for_x(local_x)
            .unwrap_or_else(|| hit_line.shaped.text.len().min(usize::MAX));
        let offset = (hit_line.start + byte_in_line).min(self.text.len());
        let mut offset = offset;
        while offset > 0 && !self.text.is_char_boundary(offset) {
            offset -= 1;
        }
        offset
    }

    fn offset_to_utf16(&self, offset: usize) -> usize {
        self.text[..offset.min(self.text.len())]
            .encode_utf16()
            .count()
    }

    fn offset_from_utf16(&self, offset: usize) -> usize {
        let mut utf16_count = 0;
        let mut utf8_offset = 0;
        for ch in self.text.chars() {
            if utf16_count >= offset {
                break;
            }
            utf16_count += ch.len_utf16();
            utf8_offset += ch.len_utf8();
        }
        utf8_offset
    }

    fn range_to_utf16(&self, range: &Range<usize>) -> Range<usize> {
        self.offset_to_utf16(range.start)..self.offset_to_utf16(range.end)
    }

    fn range_from_utf16(&self, range: &Range<usize>) -> Range<usize> {
        self.offset_from_utf16(range.start)..self.offset_from_utf16(range.end)
    }
}

impl EntityInputHandler for Editor {
    fn text_for_range(
        &mut self,
        range_utf16: Range<usize>,
        actual_range: &mut Option<Range<usize>>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<String> {
        let range = self.range_from_utf16(&range_utf16);
        actual_range.replace(self.range_to_utf16(&range));
        Some(self.text[range].to_string())
    }

    fn selected_text_range(
        &mut self,
        _ignore_disabled_input: bool,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<UTF16Selection> {
        Some(UTF16Selection {
            range: self.range_to_utf16(&self.selected_range),
            reversed: self.selection_reversed,
        })
    }

    fn marked_text_range(
        &self,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<Range<usize>> {
        self.marked_range
            .as_ref()
            .map(|range| self.range_to_utf16(range))
    }

    fn unmark_text(&mut self, _window: &mut Window, _cx: &mut Context<Self>) {
        self.marked_range = None;
    }

    fn replace_text_in_range(
        &mut self,
        range_utf16: Option<Range<usize>>,
        new_text: &str,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let range = range_utf16
            .as_ref()
            .map(|range_utf16| self.range_from_utf16(range_utf16))
            .or(self.marked_range.clone())
            .unwrap_or_else(|| self.selection_bounds());

        let start = range.start.min(self.text.len());
        let end = range.end.min(self.text.len());
        let mut new_content = String::with_capacity(self.text.len() + new_text.len());
        new_content.push_str(&self.text[..start]);
        new_content.push_str(new_text);
        new_content.push_str(&self.text[end..]);
        self.text = new_content;
        let cursor = start + new_text.len();
        self.selected_range = cursor..cursor;
        self.selection_reversed = false;
        if !new_text.is_empty() {
            self.marked_range = None;
        }
        self.dirty = true;
        self.ensure_cursor_visible();
        cx.notify();
    }

    fn replace_and_mark_text_in_range(
        &mut self,
        range_utf16: Option<Range<usize>>,
        new_text: &str,
        new_selected_range_utf16: Option<Range<usize>>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let range = range_utf16
            .as_ref()
            .map(|range_utf16| self.range_from_utf16(range_utf16))
            .or(self.marked_range.clone())
            .unwrap_or_else(|| self.selection_bounds());

        let start = range.start.min(self.text.len());
        let end = range.end.min(self.text.len());
        let mut new_content = String::with_capacity(self.text.len() + new_text.len());
        new_content.push_str(&self.text[..start]);
        new_content.push_str(new_text);
        new_content.push_str(&self.text[end..]);
        self.text = new_content;

        if !new_text.is_empty() {
            self.marked_range = Some(start..start + new_text.len());
        } else {
            self.marked_range = None;
        }

        let selected = new_selected_range_utf16
            .as_ref()
            .map(|range_utf16| self.range_from_utf16(range_utf16))
            .map(|new_range| new_range.start + start..new_range.end + start)
            .unwrap_or_else(|| start + new_text.len()..start + new_text.len());
        self.selected_range = selected;
        self.selection_reversed = false;
        self.dirty = true;
        self.ensure_cursor_visible();
        let _ = window;
        cx.notify();
    }

    fn bounds_for_range(
        &mut self,
        _range_utf16: Range<usize>,
        bounds: Bounds<Pixels>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<Bounds<Pixels>> {
        Some(bounds)
    }

    fn character_index_for_point(
        &mut self,
        _point: gpui::Point<Pixels>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<usize> {
        None
    }
}

impl Focusable for Editor {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

struct EditorElement {
    editor: Entity<Editor>,
}

struct PrepaintState {
    gutter: Pixels,
    scroll: usize,
    lines: Vec<HitLine>,
    selection: Option<PaintQuad>,
    cursor: Option<PaintQuad>,
}

impl IntoElement for EditorElement {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

impl Element for EditorElement {
    type RequestLayoutState = ();
    type PrepaintState = PrepaintState;

    fn id(&self) -> Option<ElementId> {
        None
    }

    fn source_location(&self) -> Option<&'static core::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&gpui::InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        let mut style = Style::default();
        style.size.width = relative(1.).into();
        style.size.height = relative(1.).into();
        (window.request_layout(style, [], cx), ())
    }

    fn prepaint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&gpui::InspectorElementId>,
        bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        window: &mut Window,
        cx: &mut App,
    ) -> Self::PrepaintState {
        let editor = self.editor.read(cx);
        let text = editor.text.clone();
        let language = editor.language;
        let cursor = editor.cursor();
        let selection = editor.selection_bounds();
        let scroll = editor.scroll;

        let style = window.text_style();
        let font = style.font();
        let font_size = style.font_size.to_pixels(window.rem_size());
        let line_height = window.line_height();

        let starts = line_starts(&text);
        let total_lines = starts.len();
        let digits = total_lines.to_string().len().max(2);

        let number_font = font.clone();
        let gutter_probe = window.text_system().shape_line(
            SharedString::from("0".repeat(digits)),
            font_size,
            &[TextRun {
                len: digits,
                font: number_font.clone(),
                color: theme::color(theme::FAINT),
                background_color: None,
                underline: None,
                strikethrough: None,
            }],
            None,
        );
        let gutter = gutter_probe.x_for_index(digits) + px(GUTTER_PAD * 2.0);

        let visible = ((bounds.size.height / line_height).ceil() as usize) + 1;
        let first = scroll.min(total_lines.saturating_sub(1));
        let last = (first + visible).min(total_lines);

        let mut lines: Vec<HitLine> = Vec::with_capacity(last - first);

        // Compute syntax-highlight state at the start of the visible region so
        // multi-line constructs (block comments, embedded <script>/<style>) are
        // rendered correctly even when they begin above the scroll offset.
        let mut hl_state = HighlightState::default();
        for idx in 0..first {
            let s = starts[idx];
            let e = line_end(&text, &starts, idx);
            let line_text = &text[s..e];
            hl_state = tokenize_line(language, editor.dynamic_id.as_deref(), line_text, hl_state).1;
        }

        for line in first..last {
            let start = starts[line];
            let end = line_end(&text, &starts, line);
            let line_text = &text[start..end];
            let (tokens, new_state) = tokenize_line(language, editor.dynamic_id.as_deref(), line_text, hl_state);
            hl_state = new_state;
            let runs: Vec<TextRun> = tokens
                .iter()
                .map(|t| TextRun {
                    len: t.end - t.start,
                    font: font.clone(),
                    color: token_color(t.kind),
                    background_color: None,
                    underline: None,
                    strikethrough: None,
                })
                .collect();
            let shaped = window.text_system().shape_line(
                SharedString::from(line_text.to_string()),
                font_size,
                &runs,
                None,
            );
            lines.push(HitLine { line, start, shaped });
        }

        // Selection quad (single line only, for simplicity/perf)
        let selection_quad = {
            let sel = selection.clone();
            if sel.is_empty() {
                None
            } else {
                let sel_start_line = line_index_for(&starts, sel.start);
                let sel_end_line = line_index_for(&starts, sel.end);
                if sel_start_line == sel_end_line {
                    let hit = lines.iter().find(|l| l.line == sel_start_line);
                    hit.map(|hit| {
                        let lstart = hit.start;
                        let a = sel.start.saturating_sub(lstart);
                        let b = sel.end.saturating_sub(lstart).min(hit.shaped.text.len());
                        let x1 = hit.shaped.x_for_index(a);
                        let x2 = hit.shaped.x_for_index(b);
                        let row = (hit.line - first) as f32;
                        fill(
                            Bounds::new(
                                point(
                                    bounds.origin.x + gutter + px(CODE_PAD) + x1,
                                    bounds.origin.y + line_height * row,
                                ),
                                size(x2 - x1, line_height),
                            ),
                            theme::color(theme::SURFACE_HI),
                        )
                    })
                } else {
                    // Multi-line: highlight from start of cursor line only (partial)
                    None
                }
            }
        };

        let cursor_line = line_index_for(&starts, cursor);
        let cursor_quad = lines
            .iter()
            .find(|l| l.line == cursor_line)
            .map(|hit| {
                let col = cursor.saturating_sub(hit.start);
                let x = hit.shaped.x_for_index(col);
                let row = (hit.line - first) as f32;
                fill(
                    Bounds::new(
                        point(
                            bounds.origin.x + gutter + px(CODE_PAD) + x,
                            bounds.origin.y + line_height * row,
                        ),
                        size(px(2.0), line_height),
                    ),
                    theme::color(theme::TEXT),
                )
            });

        PrepaintState {
            gutter,
            scroll: first,
            lines,
            selection: selection_quad,
            cursor: cursor_quad,
        }
    }

    fn paint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&gpui::InspectorElementId>,
        bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        prepaint: &mut Self::PrepaintState,
        window: &mut Window,
        cx: &mut App,
    ) {
        let focus_handle = self.editor.read(cx).focus_handle.clone();
        window.handle_input(
            &focus_handle,
            ElementInputHandler::new(bounds, self.editor.clone()),
            cx,
        );

        let line_height = window.line_height();
        let font_size = window.text_style().font_size.to_pixels(window.rem_size());
        let font = window.text_style().font();
        let digits = self.editor.read(cx).text.lines().count().max(1).to_string().len().max(2);

        let scroll = prepaint.scroll;
        let gutter = prepaint.gutter;

        // Backgrounds
        window.paint_quad(fill(bounds, theme::color(theme::BG)));
        window.paint_quad(fill(
            Bounds::new(bounds.origin, size(gutter, bounds.size.height)),
            theme::color(theme::BG_DARK),
        ));

        // Selection
        if let Some(quad) = prepaint.selection.take() {
            window.paint_quad(quad);
        }

        // Line numbers + code
        let line_number_runs_color = theme::color(theme::FAINT);
        for hit in &prepaint.lines {
            let row = (hit.line - scroll) as f32;
            let y = bounds.origin.y + line_height * row;

            let num = format!("{:>width$}", hit.line + 1, width = digits);
            let num_shaped = window.text_system().shape_line(
                SharedString::from(num.clone()),
                font_size,
                &[TextRun {
                    len: num.len(),
                    font: font.clone(),
                    color: line_number_runs_color,
                    background_color: None,
                    underline: None,
                    strikethrough: None,
                }],
                None,
            );
            let num_w = num_shaped.x_for_index(num.len());
            let _ = num_shaped.paint(
                point(bounds.origin.x + gutter - px(GUTTER_PAD) - num_w, y),
                line_height,
                window,
                cx,
            );

            let _ = hit.shaped.paint(
                point(bounds.origin.x + gutter + px(CODE_PAD), y),
                line_height,
                window,
                cx,
            );
        }

        // Cursor
        if focus_handle.is_focused(window)
            && let Some(cursor) = prepaint.cursor.take()
        {
            window.paint_quad(cursor);
        }

        // Store hit-testing data for mouse input
        let hit_lines: Vec<HitLine> = prepaint
            .lines
            .iter()
            .map(|l| HitLine {
                line: l.line,
                start: l.start,
                shaped: l.shaped.clone(),
            })
            .collect();
        let scroll = prepaint.scroll;
        let gutter = prepaint.gutter;
        self.editor.update(cx, |editor, _cx| {
            editor.hit = Some(HitTest {
                bounds,
                scroll,
                gutter,
                lines: hit_lines,
            });
        });
    }
}

impl Render for Editor {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .size_full()
            .key_context("Editor")
            .track_focus(&self.focus_handle(cx))
            .cursor(CursorStyle::IBeam)
            .font_family("DejaVu Sans Mono")
            .text_size(px(FONT_SIZE))
            .line_height(px(LINE_HEIGHT))
            .on_action(cx.listener(Self::backspace))
            .on_action(cx.listener(Self::delete))
            .on_action(cx.listener(Self::left))
            .on_action(cx.listener(Self::right))
            .on_action(cx.listener(Self::up))
            .on_action(cx.listener(Self::down))
            .on_action(cx.listener(Self::select_left))
            .on_action(cx.listener(Self::select_right))
            .on_action(cx.listener(Self::select_up))
            .on_action(cx.listener(Self::select_down))
            .on_action(cx.listener(Self::select_all))
            .on_action(cx.listener(Self::home))
            .on_action(cx.listener(Self::end))
            .on_action(cx.listener(Self::enter))
            .on_action(cx.listener(Self::indent))
            .on_action(cx.listener(Self::paste))
            .on_action(cx.listener(Self::cut))
            .on_action(cx.listener(Self::copy))
            .on_action(cx.listener(Self::save))
            .on_action(cx.listener(Self::new_line_below))
            .on_mouse_down(MouseButton::Left, cx.listener(Self::on_mouse_down))
            .on_mouse_up(MouseButton::Left, cx.listener(Self::on_mouse_up))
            .on_mouse_up_out(MouseButton::Left, cx.listener(Self::on_mouse_up))
            .on_mouse_move(cx.listener(Self::on_mouse_move))
            .on_scroll_wheel(cx.listener(Self::on_scroll_wheel))
            .child(EditorElement { editor: cx.entity() })
    }
}
