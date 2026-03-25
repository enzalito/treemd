use std::{collections::HashMap, fs, path::PathBuf};

use ratatui::style::Style;

pub struct Theme {
    higlights: Vec<String>,
    colors: Vec<Style>,
}

impl Theme {
    pub fn new(theme_path: PathBuf) -> Self {
        let file_content = match fs::read_to_string(theme_path) {
            Ok(content) => content,
            Err(e) => {
                eprintln!("failed to read highlighter theme: {e}");
                return Self::default();
            }
        };

        let color_map = match toml::from_str::<HashMap<String, Style>>(&file_content) {
            Ok(map) => map,
            Err(e) => {
                eprintln!("failed to parse highlighter theme: {e}");
                return Self::default();
            }
        };

        let mapping_count = color_map.capacity();
        let mut theme = Self {
            higlights: Vec::with_capacity(mapping_count),
            colors: Vec::with_capacity(mapping_count),
        };

        for (highlight, color) in color_map {
            theme.higlights.push(highlight);
            theme.colors.push(color);
        }

        theme
    }
}

impl Default for Theme {
    fn default() -> Self {
        return Theme {
            higlights: Vec::new(),
            colors: Vec::new(),
        };
    }
}
