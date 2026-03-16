use std::path::PathBuf;

use ratatui::style::Color;
use serde::Deserialize;

#[derive(Deserialize)]
struct StyleToml {
    #[serde(default)]
    colors: ColorsToml,
}

#[derive(Deserialize, Default)]
struct ColorsToml {
    border: Option<String>,
    my_text: Option<String>,
    peer_text: Option<String>,
}

pub struct Colors {
    pub border: Color,
    pub my_text: Color,
    pub peer_text: Color,
}

impl Default for Colors {
    fn default() -> Self {
        Colors {
            border: Color::DarkGray,
            my_text: Color::Green,
            peer_text: Color::Blue,
        }
    }
}

#[derive(Default)]
pub struct Style {
    pub colors: Colors,
}

fn default_style_path() -> anyhow::Result<PathBuf> {
    Ok(dirs::home_dir()
        .ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?
        .join(".punchline")
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
        },
    }
}
