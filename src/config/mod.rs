// src/config/mod.rs
use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use crossterm::style::Color;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditorConfig {
    pub tab_size: usize,
    pub theme: Theme,
    pub line_numbers: bool,
    pub auto_indent: bool,
    pub highlight_current_line: bool,
    pub show_whitespace: bool,
    pub word_wrap: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theme {
    pub name: String,
    pub background: ColorDef,
    pub foreground: ColorDef,
    pub cursor: ColorDef,
    pub selection: ColorDef,
    pub search_highlight: ColorDef,
    pub line_numbers: ColorDef,
    pub line_numbers_highlight: ColorDef,
    pub status_line: StatusLineTheme,
    pub ui: UiTheme,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusLineTheme {
    pub normal: ColorDef,
    pub insert: ColorDef,
    pub visual: ColorDef,
    pub command: ColorDef,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiTheme {
    pub background: ColorDef,
    pub foreground: ColorDef,
    pub selected: ColorDef,
    pub active: ColorDef,
    pub inactive: ColorDef,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ColorDef {
    Named(NamedColor),
    Rgb { r: u8, g: u8, b: u8 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NamedColor {
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
    BrightBlack,
    BrightRed,
    BrightGreen,
    BrightYellow,
    BrightBlue,
    BrightMagenta,
    BrightCyan,
    BrightWhite,
}

impl Default for EditorConfig {
    fn default() -> Self {
        Self {
            tab_size: 4,
            theme: Theme::default(),
            line_numbers: true,
            auto_indent: true,
            highlight_current_line: true,
            show_whitespace: false,
            word_wrap: false,
        }
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            name: "default".to_string(),
            background: ColorDef::Named(NamedColor::Black),
            foreground: ColorDef::Named(NamedColor::White),
            cursor: ColorDef::Named(NamedColor::White),
            selection: ColorDef::Named(NamedColor::BrightBlue),
            search_highlight: ColorDef::Named(NamedColor::Yellow),
            line_numbers: ColorDef::Named(NamedColor::BrightBlack),
            line_numbers_highlight: ColorDef::Named(NamedColor::White),
            status_line: StatusLineTheme::default(),
            ui: UiTheme::default(),
        }
    }
}

impl Default for StatusLineTheme {
    fn default() -> Self {
        Self {
            normal: ColorDef::Named(NamedColor::BrightBlack),
            insert: ColorDef::Named(NamedColor::BrightGreen),
            visual: ColorDef::Named(NamedColor::BrightBlue),
            command: ColorDef::Named(NamedColor::BrightYellow),
        }
    }
}

impl Default for UiTheme {
    fn default() -> Self {
        Self {
            background: ColorDef::Named(NamedColor::Black),
            foreground: ColorDef::Named(NamedColor::White),
            selected: ColorDef::Named(NamedColor::BrightBlue),
            active: ColorDef::Named(NamedColor::BrightWhite),
            inactive: ColorDef::Named(NamedColor::BrightBlack),
        }
    }
}

impl ColorDef {
    pub fn to_crossterm_color(&self) -> Color {
        match self {
            ColorDef::Named(named) => named.to_crossterm_color(),
            ColorDef::Rgb { r, g, b } => Color::Rgb { r: *r, g: *g, b: *b },
        }
    }
}

impl NamedColor {
    pub fn to_crossterm_color(&self) -> Color {
        match self {
            NamedColor::Black => Color::Black,
            NamedColor::Red => Color::Red,
            NamedColor::Green => Color::Green,
            NamedColor::Yellow => Color::Yellow,
            NamedColor::Blue => Color::Blue,
            NamedColor::Magenta => Color::Magenta,
            NamedColor::Cyan => Color::Cyan,
            NamedColor::White => Color::White,
            NamedColor::BrightBlack => Color::DarkGrey,
            NamedColor::BrightRed => Color::Red, // Crossterm doesn't have bright variants
            NamedColor::BrightGreen => Color::Green,
            NamedColor::BrightYellow => Color::Yellow,
            NamedColor::BrightBlue => Color::Blue,
            NamedColor::BrightMagenta => Color::Magenta,
            NamedColor::BrightCyan => Color::Cyan,
            NamedColor::BrightWhite => Color::White,
        }
    }
}

impl EditorConfig {
    pub fn load() -> Result<Self, ConfigError> {
        let config_path = Self::get_config_path()?;
        if config_path.exists() {
            let contents = std::fs::read_to_string(config_path)?;
            Ok(toml::from_str(&contents)?)
        } else {
            Ok(Self::default())
        }
    }

    pub fn save(&self) -> Result<(), ConfigError> {
        let config_path = Self::get_config_path()?;
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let contents = toml::to_string_pretty(self)?;
        std::fs::write(config_path, contents)?;
        Ok(())
    }

    fn get_config_path() -> Result<PathBuf, ConfigError> {
        dirs::config_dir()
            .map(|mut path| {
                path.push("zorforge");
                path.push("config.toml");
                path
            })
            .ok_or(ConfigError::NoConfigDir)
    }

    pub fn load_from_file(path: &PathBuf) -> Result<Self, ConfigError> {
        let contents = std::fs::read_to_string(path)?;
        Ok(toml::from_str(&contents)?)
    }

    pub fn load_default() -> Result<Self, ConfigError> {
        Ok(Self::default())
    }

    pub fn default() -> Self {
        Self {
            tab_size: 4,
            theme: Theme::default(),
            line_numbers: true,
            auto_indent: true,
            highlight_current_line: true,
            show_whitespace: false,
            word_wrap: true,
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Could not determine config directory")]
    NoConfigDir,
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("TOML parsing error: {0}")]
    Toml(#[from] toml::de::Error),
    
    #[error("TOML serialization error: {0}")]
    TomlSer(#[from] toml::ser::Error),
}