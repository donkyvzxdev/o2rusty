use macroquad::prelude::*;
use serde::{Serialize, Deserialize};
use std::fs;

pub const BASE_SCROLL_TIME: f64 = 2.0;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ScrollLayout {
    Center,
    Left,
}

impl ScrollLayout {
    pub fn next(&self) -> Self {
        match self {
            ScrollLayout::Center => ScrollLayout::Left,
            ScrollLayout::Left => ScrollLayout::Center,
        }
    }
    pub fn name(&self) -> &'static str {
        match self {
            ScrollLayout::Center => "Center",
            ScrollLayout::Left => "Left",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ScrollDirection {
    Down,
    Up,
}

impl ScrollDirection {
    pub fn next(&self) -> Self {
        match self {
            ScrollDirection::Down => ScrollDirection::Up,
            ScrollDirection::Up => ScrollDirection::Down,
        }
    }
    pub fn name(&self) -> &'static str {
        match self {
            ScrollDirection::Down => "Down",
            ScrollDirection::Up => "Up",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NoteStyle {
    Rectangle,
    Circle,
    Arrow,
}

impl NoteStyle {
    pub fn next(&self) -> Self {
        match self {
            NoteStyle::Rectangle => NoteStyle::Circle,
            NoteStyle::Circle => NoteStyle::Arrow,
            NoteStyle::Arrow => NoteStyle::Rectangle,
        }
    }
    pub fn name(&self) -> &'static str {
        match self {
            NoteStyle::Rectangle => "Rectangle",
            NoteStyle::Circle => "Circle",
            NoteStyle::Arrow => "Arrow",
        }
    }
}

#[derive(Clone)]
pub struct GameConfig {
    pub key_bindings: [KeyCode; 7],
    pub scroll_speed: f64,
    pub column_start: f64,
    pub scroll_layout: ScrollLayout,
    pub scroll_direction: ScrollDirection,
    pub volume: f64,
    pub hit_position: f64,
    pub note_colors: [String; 7],
    pub note_height: f64,
    pub column_width: f64,
    pub audio_offset_ms: f64,
    pub ln_shortening_ms: f64,
    pub note_style: NoteStyle,
    pub column_spacing: f64,
    pub column_line_color: String,
    pub column_bg_color: String,
    pub column_line_enabled: bool,
    pub column_bg_enabled: bool,
}

impl Default for GameConfig {
    fn default() -> Self {
        use KeyCode::*;
        GameConfig {
            key_bindings: [S, D, F, Space, J, K, L],
            scroll_speed: 3.0,
            column_start: 0.0,
            scroll_layout: ScrollLayout::Center,
            scroll_direction: ScrollDirection::Down,
            volume: 1.0,
            hit_position: 555.0,
            note_colors: [
                "#9C59B5".into(), "#3399DB".into(), "#2ECC70".into(),
                "#F2C40F".into(), "#E67D21".into(), "#E84D3D".into(),
                "#EDEDF2".into(),
            ],
            note_height: 22.0,
            column_width: 62.0,
            audio_offset_ms: 0.0,
            ln_shortening_ms: 0.0,
            note_style: NoteStyle::Rectangle,
            column_spacing: 4.0,
            column_line_color: "#26263AFF".into(),
            column_bg_color: "#14141EFF".into(),
            column_line_enabled: true,
            column_bg_enabled: true,
        }
    }
}

pub fn keycode_name(kc: KeyCode) -> &'static str {
    use KeyCode::*;
    match kc {
        A => "A", B => "B", C => "C", D => "D", E => "E", F => "F", G => "G",
        H => "H", I => "I", J => "J", K => "K", L => "L", M => "M",
        N => "N", O => "O", P => "P", Q => "Q", R => "R", S => "S", T => "T",
        U => "U", V => "V", W => "W", X => "X", Y => "Y", Z => "Z",
        Key0 => "0", Key1 => "1", Key2 => "2", Key3 => "3", Key4 => "4",
        Key5 => "5", Key6 => "6", Key7 => "7", Key8 => "8", Key9 => "9",
        F1 => "F1", F2 => "F2", F3 => "F3", F4 => "F4", F5 => "F5", F6 => "F6",
        F7 => "F7", F8 => "F8", F9 => "F9", F10 => "F10", F11 => "F11", F12 => "F12",
        Space => "Space", Enter => "Enter", Escape => "Esc",
        LeftShift => "LShift", RightShift => "RShift",
        LeftControl => "LCtrl", RightControl => "RCtrl",
        LeftAlt => "LAlt", RightAlt => "RAlt",
        Tab => "Tab", Backspace => "Bksp",
        CapsLock => "CapsLk", ScrollLock => "ScrlLk", NumLock => "NumLk",
        PrintScreen => "PrtSc", Pause => "Pause", Menu => "Menu",
        Insert => "Ins", Delete => "Del",
        Home => "Home", End => "End", PageUp => "PgUp", PageDown => "PgDn",
        Up => "Up", Down => "Down", Left => "Left", Right => "Right",
        Minus => "-", Equal => "=",
        LeftBracket => "[", RightBracket => "]",
        Semicolon => ";", Apostrophe => "'", GraveAccent => "`",
        Comma => ",", Period => ".", Slash => "/", Backslash => "\\",
        Kp0 => "Kp0", Kp1 => "Kp1", Kp2 => "Kp2", Kp3 => "Kp3",
        Kp4 => "Kp4", Kp5 => "Kp5", Kp6 => "Kp6", Kp7 => "Kp7",
        Kp8 => "Kp8", Kp9 => "Kp9",
        KpEnter => "KpEnt", KpAdd => "Kp+", KpSubtract => "Kp-",
        KpMultiply => "Kp*", KpDivide => "Kp/", KpDecimal => "Kp.",
        _ => "?",
    }
}

pub fn keycode_from_name(name: &str) -> Option<KeyCode> {
    use KeyCode::*;
    match name {
        "A" => Some(A), "B" => Some(B), "C" => Some(C), "D" => Some(D),
        "E" => Some(E), "F" => Some(F), "G" => Some(G), "H" => Some(H),
        "I" => Some(I), "J" => Some(J), "K" => Some(K), "L" => Some(L),
        "M" => Some(M), "N" => Some(N), "O" => Some(O), "P" => Some(P),
        "Q" => Some(Q), "R" => Some(R), "S" => Some(S), "T" => Some(T),
        "U" => Some(U), "V" => Some(V), "W" => Some(W), "X" => Some(X),
        "Y" => Some(Y), "Z" => Some(Z),
        "0" => Some(Key0), "1" => Some(Key1), "2" => Some(Key2),
        "3" => Some(Key3), "4" => Some(Key4), "5" => Some(Key5),
        "6" => Some(Key6), "7" => Some(Key7), "8" => Some(Key8),
        "9" => Some(Key9),
        "F1" => Some(F1), "F2" => Some(F2), "F3" => Some(F3), "F4" => Some(F4),
        "F5" => Some(F5), "F6" => Some(F6), "F7" => Some(F7), "F8" => Some(F8),
        "F9" => Some(F9), "F10" => Some(F10), "F11" => Some(F11), "F12" => Some(F12),
        "Space" => Some(Space), "Enter" => Some(Enter), "Esc" => Some(Escape),
        "LShift" => Some(LeftShift), "RShift" => Some(RightShift),
        "LCtrl" => Some(LeftControl), "RCtrl" => Some(RightControl),
        "LAlt" => Some(LeftAlt), "RAlt" => Some(RightAlt),
        "Tab" => Some(Tab), "Bksp" => Some(Backspace),
        "CapsLk" => Some(CapsLock), "ScrlLk" => Some(ScrollLock), "NumLk" => Some(NumLock),
        "PrtSc" => Some(PrintScreen), "Pause" => Some(Pause), "Menu" => Some(Menu),
        "Ins" => Some(Insert), "Del" => Some(Delete),
        "Home" => Some(Home), "End" => Some(End), "PgUp" => Some(PageUp), "PgDn" => Some(PageDown),
        "Up" => Some(Up), "Down" => Some(Down), "Left" => Some(Left), "Right" => Some(Right),
        "-" => Some(Minus), "=" => Some(Equal),
        "[" => Some(LeftBracket), "]" => Some(RightBracket),
        ";" => Some(Semicolon), "'" => Some(Apostrophe), "`" => Some(GraveAccent),
        "," => Some(Comma), "." => Some(Period),
        "/" => Some(Slash), "\\" => Some(Backslash),
        "Kp0" => Some(Kp0), "Kp1" => Some(Kp1), "Kp2" => Some(Kp2), "Kp3" => Some(Kp3),
        "Kp4" => Some(Kp4), "Kp5" => Some(Kp5), "Kp6" => Some(Kp6), "Kp7" => Some(Kp7),
        "Kp8" => Some(Kp8), "Kp9" => Some(Kp9),
        "KpEnt" => Some(KpEnter), "Kp+" => Some(KpAdd), "Kp-" => Some(KpSubtract),
        "Kp*" => Some(KpMultiply), "Kp/" => Some(KpDivide), "Kp." => Some(KpDecimal),
        _ => None,
    }
}

pub const ALL_KEYS: &[(KeyCode, &str)] = &[
    (KeyCode::A, "A"), (KeyCode::B, "B"), (KeyCode::C, "C"), (KeyCode::D, "D"),
    (KeyCode::E, "E"), (KeyCode::F, "F"), (KeyCode::G, "G"), (KeyCode::H, "H"),
    (KeyCode::I, "I"), (KeyCode::J, "J"), (KeyCode::K, "K"), (KeyCode::L, "L"),
    (KeyCode::M, "M"), (KeyCode::N, "N"), (KeyCode::O, "O"), (KeyCode::P, "P"),
    (KeyCode::Q, "Q"), (KeyCode::R, "R"), (KeyCode::S, "S"), (KeyCode::T, "T"),
    (KeyCode::U, "U"), (KeyCode::V, "V"), (KeyCode::W, "W"), (KeyCode::X, "X"),
    (KeyCode::Y, "Y"), (KeyCode::Z, "Z"),
    (KeyCode::Key0, "0"), (KeyCode::Key1, "1"), (KeyCode::Key2, "2"),
    (KeyCode::Key3, "3"), (KeyCode::Key4, "4"), (KeyCode::Key5, "5"),
    (KeyCode::Key6, "6"), (KeyCode::Key7, "7"), (KeyCode::Key8, "8"),
    (KeyCode::Key9, "9"),
    (KeyCode::F1, "F1"), (KeyCode::F2, "F2"), (KeyCode::F3, "F3"),
    (KeyCode::F4, "F4"), (KeyCode::F5, "F5"), (KeyCode::F6, "F6"),
    (KeyCode::F7, "F7"), (KeyCode::F8, "F8"), (KeyCode::F9, "F9"),
    (KeyCode::F10, "F10"), (KeyCode::F11, "F11"), (KeyCode::F12, "F12"),
    (KeyCode::Space, "Space"), (KeyCode::Enter, "Enter"), (KeyCode::Escape, "Esc"),
    (KeyCode::LeftShift, "LShift"), (KeyCode::RightShift, "RShift"),
    (KeyCode::LeftControl, "LCtrl"), (KeyCode::RightControl, "RCtrl"),
    (KeyCode::LeftAlt, "LAlt"), (KeyCode::RightAlt, "RAlt"),
    (KeyCode::Tab, "Tab"), (KeyCode::Backspace, "Bksp"),
    (KeyCode::CapsLock, "CapsLk"), (KeyCode::ScrollLock, "ScrlLk"), (KeyCode::NumLock, "NumLk"),
    (KeyCode::PrintScreen, "PrtSc"), (KeyCode::Pause, "Pause"), (KeyCode::Menu, "Menu"),
    (KeyCode::Insert, "Ins"), (KeyCode::Delete, "Del"),
    (KeyCode::Home, "Home"), (KeyCode::End, "End"),
    (KeyCode::PageUp, "PgUp"), (KeyCode::PageDown, "PgDn"),
    (KeyCode::Up, "Up"), (KeyCode::Down, "Down"), (KeyCode::Left, "Left"), (KeyCode::Right, "Right"),
    (KeyCode::Minus, "-"), (KeyCode::Equal, "="),
    (KeyCode::LeftBracket, "["), (KeyCode::RightBracket, "]"),
    (KeyCode::Semicolon, ";"), (KeyCode::Apostrophe, "'"), (KeyCode::GraveAccent, "`"),
    (KeyCode::Comma, ","), (KeyCode::Period, "."),
    (KeyCode::Slash, "/"), (KeyCode::Backslash, "\\"),
    (KeyCode::Kp0, "Kp0"), (KeyCode::Kp1, "Kp1"), (KeyCode::Kp2, "Kp2"),
    (KeyCode::Kp3, "Kp3"), (KeyCode::Kp4, "Kp4"), (KeyCode::Kp5, "Kp5"),
    (KeyCode::Kp6, "Kp6"), (KeyCode::Kp7, "Kp7"), (KeyCode::Kp8, "Kp8"),
    (KeyCode::Kp9, "Kp9"),
    (KeyCode::KpEnter, "KpEnt"), (KeyCode::KpAdd, "Kp+"), (KeyCode::KpSubtract, "Kp-"),
    (KeyCode::KpMultiply, "Kp*"), (KeyCode::KpDivide, "Kp/"), (KeyCode::KpDecimal, "Kp."),
];

pub fn hex_to_color(hex: &str) -> Color {
    let h = hex.trim_start_matches('#');
    if h.len() < 6 { return Color::new(1.0, 1.0, 1.0, 1.0); }
    let r = u8::from_str_radix(&h[0..2], 16).unwrap_or(255) as f32 / 255.0;
    let g = u8::from_str_radix(&h[2..4], 16).unwrap_or(255) as f32 / 255.0;
    let b = u8::from_str_radix(&h[4..6], 16).unwrap_or(255) as f32 / 255.0;
    let a = if h.len() >= 8 {
        u8::from_str_radix(&h[6..8], 16).unwrap_or(255) as f32 / 255.0
    } else { 1.0 };
    Color::new(r, g, b, a)
}

#[allow(dead_code)]
fn color_to_hex(c: Color) -> String {
    let ri = (c.r * 255.0).round() as u8;
    let gi = (c.g * 255.0).round() as u8;
    let bi = (c.b * 255.0).round() as u8;
    let ai = (c.a * 255.0).round() as u8;
    format!("#{:02X}{:02X}{:02X}{:02X}", ri, gi, bi, ai)
}

#[derive(Serialize, Deserialize)]
struct ConfigData {
    key_bindings: Vec<String>,
    scroll_speed: f64,
    column_start: f64,
    scroll_layout: String,
    scroll_direction: String,
    volume: f64,
    hit_position: f64,
    #[serde(default)]
    note_colors: Vec<String>,
    #[serde(default)]
    note_height: Option<f64>,
    #[serde(default)]
    column_width: Option<f64>,
    #[serde(default)]
    audio_offset_ms: f64,
    #[serde(default)]
    ln_shortening_ms: f64,
    #[serde(default)]
    note_style: String,
    #[serde(default)]
    column_spacing: f64,
    #[serde(default)]
    column_line_color: String,
    #[serde(default)]
    column_bg_color: String,
    #[serde(default)]
    column_line_enabled: bool,
    #[serde(default)]
    column_bg_enabled: bool,
}

impl From<&GameConfig> for ConfigData {
    fn from(c: &GameConfig) -> Self {
        ConfigData {
            key_bindings: c.key_bindings.iter().map(|k| keycode_name(*k).to_string()).collect(),
            scroll_speed: c.scroll_speed,
            column_start: c.column_start,
            scroll_layout: c.scroll_layout.name().to_string(),
            scroll_direction: c.scroll_direction.name().to_string(),
            volume: c.volume,
            hit_position: c.hit_position,
            note_colors: c.note_colors.to_vec(),
            note_height: Some(c.note_height),
            column_width: Some(c.column_width),
            audio_offset_ms: c.audio_offset_ms,
            ln_shortening_ms: c.ln_shortening_ms,
            note_style: c.note_style.name().to_string(),
            column_spacing: c.column_spacing,
            column_line_color: c.column_line_color.clone(),
            column_bg_color: c.column_bg_color.clone(),
            column_line_enabled: c.column_line_enabled,
            column_bg_enabled: c.column_bg_enabled,
        }
    }
}

impl ConfigData {
    fn to_config(&self) -> GameConfig {
        let def = GameConfig::default();
        let mut bindings = def.key_bindings;
        for (i, name) in self.key_bindings.iter().enumerate() {
            if i < 7 {
                if let Some(kc) = keycode_from_name(name) {
                    bindings[i] = kc;
                }
            }
        }
        let mut note_colors = def.note_colors;
        for (i, c) in self.note_colors.iter().enumerate() {
            if i < 7 { note_colors[i] = c.clone(); }
        }

        GameConfig {
            key_bindings: bindings,
            scroll_speed: self.scroll_speed,
            column_start: self.column_start,
            scroll_layout: match self.scroll_layout.as_str() {
                "Left" => ScrollLayout::Left,
                _ => ScrollLayout::Center,
            },
            scroll_direction: match self.scroll_direction.as_str() {
                "Up" => ScrollDirection::Up,
                _ => ScrollDirection::Down,
            },
            volume: self.volume.clamp(0.0, 1.0),
            hit_position: self.hit_position.clamp(50.0, 700.0),
            note_colors,
            note_height: self.note_height.unwrap_or(def.note_height).clamp(5.0, 80.0),
            column_width: self.column_width.unwrap_or(def.column_width).clamp(10.0, 300.0),
            audio_offset_ms: self.audio_offset_ms.clamp(-500.0, 500.0),
            ln_shortening_ms: self.ln_shortening_ms.clamp(0.0, 500.0),
            note_style: match self.note_style.as_str() {
                "Circle" => NoteStyle::Circle,
                "Arrow" => NoteStyle::Arrow,
                _ => NoteStyle::Rectangle,
            },
            column_spacing: self.column_spacing.clamp(-100.0, 100.0),
            column_line_color: if self.column_line_color.len() >= 6 { self.column_line_color.clone() } else { def.column_line_color },
            column_bg_color: if self.column_bg_color.len() >= 6 { self.column_bg_color.clone() } else { def.column_bg_color },
            column_line_enabled: self.column_line_enabled,
            column_bg_enabled: self.column_bg_enabled,
        }
    }
}

pub fn save_config(config: &GameConfig) {
    let data = ConfigData::from(config);
    if let Ok(json) = serde_json::to_string_pretty(&data) {
        let _ = fs::write("config.json", json);
    }
}

pub fn load_config() -> GameConfig {
    let content = match fs::read_to_string("config.json") {
        Ok(c) => c,
        Err(_) => return GameConfig::default(),
    };
    let data: ConfigData = match serde_json::from_str(&content) {
        Ok(d) => d,
        Err(_) => return GameConfig::default(),
    };
    data.to_config()
}
