#![allow(dead_code)]

use std::path::PathBuf;
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::RwLock;

use gpui::{Hsla, rgb};

// Canonical dark (Catppuccin Mocha) values. These double as stable lookup keys
// so that call sites can keep passing `theme::BG` etc. to `color()`.
pub const BG: u32 = 0x1e1e2e;
pub const BG_DARK: u32 = 0x181825;
pub const BG_DARKER: u32 = 0x11111b;
pub const SURFACE: u32 = 0x313244;
pub const SURFACE_HI: u32 = 0x45475a;
pub const OVERLAY: u32 = 0x6c7086;
pub const TEXT: u32 = 0xcdd6f4;
pub const SUBTEXT: u32 = 0xa6adc8;
pub const FAINT: u32 = 0x7f849c;
pub const BLUE: u32 = 0x89b4fa;
pub const GREEN: u32 = 0xa6e3a1;
pub const YELLOW: u32 = 0xf9e2af;
pub const RED: u32 = 0xf38ba8;
pub const MAUVE: u32 = 0xcba6f7;
pub const PEACH: u32 = 0xfab387;
pub const TEAL: u32 = 0x94e2d5;
pub const SKY: u32 = 0x89dceb;
pub const PINK: u32 = 0xf5c2e7;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ThemeMode {
    System,
    Light,
    Dark,
}

impl ThemeMode {
    pub const ALL: [ThemeMode; 3] = [ThemeMode::System, ThemeMode::Light, ThemeMode::Dark];

    pub fn label(self) -> &'static str {
        match self {
            ThemeMode::System => "System",
            ThemeMode::Light => "Light",
            ThemeMode::Dark => "Dark",
        }
    }

    fn as_u8(self) -> u8 {
        match self {
            ThemeMode::System => 0,
            ThemeMode::Light => 1,
            ThemeMode::Dark => 2,
        }
    }

    fn from_str(s: &str) -> Option<Self> {
        match s.trim().to_lowercase().as_str() {
            "system" => Some(ThemeMode::System),
            "light" => Some(ThemeMode::Light),
            "dark" => Some(ThemeMode::Dark),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Accent {
    Mauve,
    Blue,
    Green,
    Red,
    Orange,
    Pink,
    Teal,
    Yellow,
    White,
    Black,
}

impl Accent {
    pub const ALL: [Accent; 10] = [
        Accent::Mauve,
        Accent::Blue,
        Accent::Green,
        Accent::Red,
        Accent::Orange,
        Accent::Pink,
        Accent::Teal,
        Accent::Yellow,
        Accent::White,
        Accent::Black,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Accent::Mauve => "Mauve",
            Accent::Blue => "Dark Blue",
            Accent::Green => "Green",
            Accent::Red => "Red",
            Accent::Orange => "Orange",
            Accent::Pink => "Pink",
            Accent::Teal => "Light Blue",
            Accent::Yellow => "Yellow",
            Accent::White => "White",
            Accent::Black => "Black",
        }
    }

    /// A representative colour for previewing the accent in the settings UI.
    pub fn swatch(self) -> Hsla {
        rgb(self.rgb(false)).into()
    }

    fn as_u8(self) -> u8 {
        match self {
            Accent::Mauve => 0,
            Accent::Blue => 1,
            Accent::Green => 2,
            Accent::Red => 3,
            Accent::Orange => 4,
            Accent::Pink => 5,
            Accent::Teal => 6,
            Accent::Yellow => 7,
            Accent::White => 8,
            Accent::Black => 9,
        }
    }

    fn from_u8(v: u8) -> Self {
        match v {
            1 => Accent::Blue,
            2 => Accent::Green,
            3 => Accent::Red,
            4 => Accent::Orange,
            5 => Accent::Pink,
            6 => Accent::Teal,
            7 => Accent::Yellow,
            8 => Accent::White,
            9 => Accent::Black,
            _ => Accent::Mauve,
        }
    }

    fn from_str(s: &str) -> Option<Self> {
        let norm = s.trim().to_lowercase().replace([' ', '-', '_'], "");
        Accent::ALL
            .into_iter()
            .find(|a| a.label().to_lowercase().replace([' ', '-', '_'], "") == norm)
    }

    fn rgb(self, light: bool) -> u32 {
        match (self, light) {
            (Accent::Mauve, false) => 0xcba6f7,
            (Accent::Mauve, true) => 0x8839ef,
            (Accent::Blue, false) => 0x3b5bdb,
            (Accent::Blue, true) => 0x1e66f5,
            (Accent::Green, false) => 0xa6e3a1,
            (Accent::Green, true) => 0x40a02b,
            (Accent::Red, false) => 0xf38ba8,
            (Accent::Red, true) => 0xd20f39,
            (Accent::Orange, false) => 0xfab387,
            (Accent::Orange, true) => 0xfe640b,
            (Accent::Pink, false) => 0xf5c2e7,
            (Accent::Pink, true) => 0xea76cb,
            (Accent::Teal, false) => 0x89dceb,
            (Accent::Teal, true) => 0x04a5e5,
            (Accent::Yellow, false) => 0xf9e2af,
            (Accent::Yellow, true) => 0xdf8e1d,
            (Accent::White, false) => 0xffffff,
            (Accent::White, true) => 0x11111b,
            (Accent::Black, false) => 0x45475a,
            (Accent::Black, true) => 0x11111b,
        }
    }
}

/// Preference (what the user picked) and the resolved mode (after System is
/// resolved). Stored in atomics so `color()` stays cheap during rendering.
static PREF: AtomicU8 = AtomicU8::new(2); // default: Dark
static MODE: AtomicU8 = AtomicU8::new(2); // resolved: 1 = Light, 2 = Dark
static ACCENT: AtomicU8 = AtomicU8::new(0); // default: Mauve

#[derive(Clone, Copy)]
pub struct Settings {
    pub mode: ThemeMode,
    pub accent: Accent,
}

static LOADED: RwLock<Option<Settings>> = RwLock::new(None);

pub fn is_light() -> bool {
    MODE.load(Ordering::Relaxed) == 1
}

pub fn current_mode() -> ThemeMode {
    match PREF.load(Ordering::Relaxed) {
        0 => ThemeMode::System,
        1 => ThemeMode::Light,
        _ => ThemeMode::Dark,
    }
}

pub fn current_accent() -> Accent {
    Accent::from_u8(ACCENT.load(Ordering::Relaxed))
}

/// Load persisted settings. Call once at startup.
pub fn init() {
    let mut loaded = LOADED.write().unwrap();
    if loaded.is_some() {
        return;
    }
    let settings = load().unwrap_or(Settings {
        mode: ThemeMode::Dark,
        accent: Accent::Mauve,
    });
    apply_in_memory(settings);
    *loaded = Some(settings);
}

/// Change the active theme and persist it.
pub fn update(mode: ThemeMode, accent: Accent) {
    let settings = Settings { mode, accent };
    // Re-resolve System each time the user touches settings.
    apply_in_memory(settings);
    *LOADED.write().unwrap() = Some(settings);
    save(settings);
}

fn apply_in_memory(settings: Settings) {
    PREF.store(settings.mode.as_u8(), Ordering::Relaxed);
    ACCENT.store(settings.accent.as_u8(), Ordering::Relaxed);
    let resolved = match settings.mode {
        ThemeMode::System => detect_system(),
        other => other,
    };
    MODE.store(resolved.as_u8(), Ordering::Relaxed);
}

/// The public colour accessor. Values are the dark palette constants above; in
/// light mode they are remapped and accent colours are substituted.
pub fn color(v: u32) -> Hsla {
    let light = is_light();
    if v == BLUE || v == MAUVE {
        return rgb(current_accent().rgb(light)).into();
    }
    let rgb_value = if light { light_value(v) } else { v };
    rgb(rgb_value).into()
}

/// The current accent as a drawable colour.
pub fn accent() -> Hsla {
    rgb(current_accent().rgb(is_light())).into()
}

fn light_value(v: u32) -> u32 {
    match v {
        BG => 0xeff1f5,
        BG_DARK => 0xe6e9ef,
        BG_DARKER => 0xdce0e8,
        SURFACE => 0xccd0da,
        SURFACE_HI => 0xbcc0cc,
        OVERLAY => 0x9ca0b0,
        TEXT => 0x4c4f69,
        SUBTEXT => 0x5c5f77,
        FAINT => 0x8c8fa1,
        BLUE => 0x1e66f5,
        GREEN => 0x40a02b,
        YELLOW => 0xdf8e1d,
        RED => 0xd20f39,
        MAUVE => 0x8839ef,
        PEACH => 0xfe640b,
        TEAL => 0x179299,
        SKY => 0x04a5e5,
        PINK => 0xea76cb,
        other => other,
    }
}

fn detect_system() -> ThemeMode {
    #[cfg(target_os = "macos")]
    {
        if let Ok(out) = std::process::Command::new("defaults")
            .args(["read", "-g", "AppleInterfaceStyle"])
            .output()
        {
            return if String::from_utf8_lossy(&out.stdout)
                .trim()
                .eq_ignore_ascii_case("dark")
            {
                ThemeMode::Dark
            } else {
                ThemeMode::Light
            };
        }
        return ThemeMode::Light;
    }
    #[cfg(target_os = "windows")]
    {
        if let Ok(out) = std::process::Command::new("reg")
            .args([
                "query",
                r"HKCU\Software\Microsoft\Windows\CurrentVersion\Themes\Personalize",
                "/v",
                "AppsUseLightTheme",
            ])
            .output()
        {
            let text = String::from_utf8_lossy(&out.stdout);
            return if text.contains("0x0") {
                ThemeMode::Dark
            } else {
                ThemeMode::Light
            };
        }
        return ThemeMode::Dark;
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        if let Ok(out) = std::process::Command::new("gsettings")
            .args(["get", "org.gnome.desktop.interface", "color-scheme"])
            .output()
        {
            let text = String::from_utf8_lossy(&out.stdout).to_lowercase();
            if text.contains("dark") {
                return ThemeMode::Dark;
            }
            if text.contains("light") {
                return ThemeMode::Light;
            }
        }
        if let Ok(out) = std::process::Command::new("gsettings")
            .args(["get", "org.gnome.desktop.interface", "gtk-theme"])
            .output()
        {
            if String::from_utf8_lossy(&out.stdout)
                .to_lowercase()
                .contains("dark")
            {
                return ThemeMode::Dark;
            }
        }
        ThemeMode::Dark
    }
}

pub fn config_dir() -> PathBuf {
    #[cfg(target_os = "windows")]
    {
        if let Some(appdata) = std::env::var_os("APPDATA") {
            return PathBuf::from(appdata).join("Codify");
        }
    }
    #[cfg(target_os = "macos")]
    {
        if let Some(home) = std::env::var_os("HOME") {
            return PathBuf::from(home)
                .join("Library")
                .join("Application Support")
                .join("Codify");
        }
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        if let Some(xdg) = std::env::var_os("XDG_CONFIG_HOME")
            && !xdg.is_empty()
        {
            return PathBuf::from(xdg).join("codify");
        }
        if let Some(home) = std::env::var_os("HOME") {
            return PathBuf::from(home).join(".config").join("codify");
        }
    }
    PathBuf::from(".codify")
}

fn config_path() -> PathBuf {
    config_dir().join("settings.toml")
}

fn load() -> Option<Settings> {
    let text = std::fs::read_to_string(config_path()).ok()?;
    let mut mode = ThemeMode::Dark;
    let mut accent = Accent::Mauve;
    for line in text.lines() {
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        let value = value.trim().trim_matches('"');
        match key.trim() {
            "mode" => mode = ThemeMode::from_str(value).unwrap_or(mode),
            "accent" => accent = Accent::from_str(value).unwrap_or(accent),
            _ => {}
        }
    }
    Some(Settings { mode, accent })
}

fn save(settings: Settings) {
    let dir = config_dir();
    if std::fs::create_dir_all(&dir).is_err() {
        return;
    }
    let mode = match settings.mode {
        ThemeMode::System => "system",
        ThemeMode::Light => "light",
        ThemeMode::Dark => "dark",
    };
    let accent = settings.accent.label().to_lowercase().replace(' ', "-");
    let contents = format!("mode = \"{mode}\"\naccent = \"{accent}\"\n");
    let _ = std::fs::write(config_path(), contents);
}
