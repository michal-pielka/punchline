use std::path::PathBuf;

use ratatui::style::Color;
use serde::Deserialize;

#[derive(Deserialize)]
struct StyleToml {
    #[serde(default)]
    colors: ColorsToml,
    #[serde(default)]
    padding: PaddingToml,
}

#[derive(Deserialize, Default)]
struct ColorsToml {
    border: Option<String>,
    my_text: Option<String>,
    peer_text: Option<String>,
    input_text: Option<String>,
    sidebar_key: Option<String>,
    sidebar_value: Option<String>,
}

#[derive(Deserialize, Default)]
struct PaddingToml {
    #[serde(default)]
    chat_horizontal: u16,
    #[serde(default)]
    chat_vertical: u16,
}

pub struct Colors {
    pub border: Color,
    pub my_text: Color,
    pub peer_text: Color,
    pub input_text: Color,
    pub sidebar_key: Color,
    pub sidebar_value: Color,
}

#[derive(Default)]
pub struct Padding {
    pub chat_horizontal: u16,
    pub chat_vertical: u16,
}

impl Default for Colors {
    fn default() -> Self {
        Colors {
            border: Color::Reset,
            my_text: Color::Reset,
            peer_text: Color::Reset,
            input_text: Color::Reset,
            sidebar_key: Color::Reset,
            sidebar_value: Color::Reset,
        }
    }
}

#[derive(Default)]
pub struct Style {
    pub colors: Colors,
    pub padding: Padding,
}

fn default_style_path() -> anyhow::Result<PathBuf> {
    Ok(dirs::config_dir()
        .ok_or_else(|| anyhow::anyhow!("Could not find config directory"))?
        .join("punchline")
        .join("style.toml"))
}

fn parse_color(s: &str) -> Option<Color> {
    let s = s.strip_prefix('#').unwrap_or(s);
    if s.len() == 6 {
        let r = u8::from_str_radix(&s[0..2], 16).ok()?;
        let g = u8::from_str_radix(&s[2..4], 16).ok()?;
        let b = u8::from_str_radix(&s[4..6], 16).ok()?;
        return Some(Color::Rgb(r, g, b));
    }
    None
}

pub fn load_style() -> Style {
    let defaults = Style::default();

    let path = match default_style_path() {
        Ok(p) => p,
        Err(_) => return defaults,
    };

    let content = match std::fs::read_to_string(&path) {
        Ok(c) => c,
        Err(_) => return defaults,
    };

    let toml: StyleToml = match toml::from_str(&content) {
        Ok(t) => t,
        Err(_) => return defaults,
    };

    let colors = toml.colors;
    let default_colors = &defaults.colors;
    Style {
        colors: Colors {
            border: colors
                .border
                .as_deref()
                .and_then(parse_color)
                .unwrap_or(default_colors.border),
            my_text: colors
                .my_text
                .as_deref()
                .and_then(parse_color)
                .unwrap_or(default_colors.my_text),
            peer_text: colors
                .peer_text
                .as_deref()
                .and_then(parse_color)
                .unwrap_or(default_colors.peer_text),
            input_text: colors
                .input_text
                .as_deref()
                .and_then(parse_color)
                .unwrap_or(default_colors.input_text),
            sidebar_key: colors
                .sidebar_key
                .as_deref()
                .and_then(parse_color)
                .unwrap_or(default_colors.sidebar_key),
            sidebar_value: colors
                .sidebar_value
                .as_deref()
                .and_then(parse_color)
                .unwrap_or(default_colors.sidebar_value),
        },
        padding: Padding {
            chat_horizontal: toml.padding.chat_horizontal,
            chat_vertical: toml.padding.chat_vertical,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_color_hex_without_hash() {
        assert_eq!(parse_color("FF8000"), Some(Color::Rgb(255, 128, 0)));
    }

    #[test]
    fn parse_color_hex_with_hash() {
        assert_eq!(parse_color("#FF8000"), Some(Color::Rgb(255, 128, 0)));
    }

    #[test]
    fn parse_color_invalid_returns_none() {
        assert_eq!(parse_color("ZZZZZZ"), None);
    }

    #[test]
    fn parse_color_wrong_length_returns_none() {
        assert_eq!(parse_color("FFF"), None);
    }

    #[test]
    fn style_toml_with_overrides() {
        let toml_str = r#"
            [colors]
            border = "FF0000"
            my_text = "00FF00"

            [padding]
            chat_horizontal = 3
            chat_vertical = 1
        "#;
        let parsed: StyleToml = toml::from_str(toml_str).unwrap();
        assert_eq!(parsed.colors.border.as_deref(), Some("FF0000"));
        assert!(parsed.colors.peer_text.is_none()); // not overridden
        assert_eq!(parsed.padding.chat_horizontal, 3);
    }

    #[test]
    fn style_toml_empty() {
        let parsed: StyleToml = toml::from_str("").unwrap();
        assert!(parsed.colors.border.is_none());
        assert_eq!(parsed.padding.chat_horizontal, 0);
    }
}
