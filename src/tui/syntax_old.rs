use std::path::PathBuf;

use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use syntect::easy::HighlightLines;
use syntect::highlighting::{Theme, ThemeSet};
use syntect::parsing::SyntaxSet;
use syntect::util::LinesWithEndings;

const DEFAULT_CODE_THEME: &str = "base16-ocean.dark";

pub struct SyntaxHighlighter {
    syntax_set: SyntaxSet,
    theme: Theme,
}

impl SyntaxHighlighter {
    pub fn new(theme: &str, theme_dir: Option<PathBuf>) -> Self {
        let syntax_set = SyntaxSet::load_defaults_newlines();
        let mut theme_set = ThemeSet::load_defaults();
        if let Some(dir) = theme_dir
            && let Ok(paths) = ThemeSet::discover_theme_paths(dir)
        {
            for path in paths {
                match ThemeSet::get_theme(&path) {
                    Ok(theme) => {
                        if let Some(name) = path.file_stem().and_then(|s| s.to_str()) {
                            theme_set.themes.insert(name.to_owned(), theme);
                        }
                    }
                    Err(e) => {
                        eprintln!("warning: skipping theme {}: {}", path.display(), e);
                    }
                }
            }
        }

        let theme = theme_set
            .themes
            .get(theme)
            .or_else(|| {
                if theme != DEFAULT_CODE_THEME {
                    eprintln!(
                        "warning: code theme '{}' not found, using '{}'",
                        theme, DEFAULT_CODE_THEME
                    );
                }
                theme_set.themes.get(DEFAULT_CODE_THEME)
            })
            .cloned()
            .expect("syntect default themes must contain base16-ocean.dark");

        Self { syntax_set, theme }
    }

    pub fn highlight_code(&self, code: &str, language: &str) -> Vec<Line<'static>> {
        // Replace tabs with spaces to avoid terminal rendering artifacts during scrolling
        // Tabs can cause inconsistent display widths across different terminals
        let code = code.replace('\t', "    ");

        let syntax = self
            .syntax_set
            .find_syntax_by_token(language)
            .unwrap_or_else(|| self.syntax_set.find_syntax_plain_text());

        let mut highlighter = HighlightLines::new(syntax, &self.theme);
        let mut lines = Vec::new();

        for line in LinesWithEndings::from(&code) {
            let ranges = highlighter
                .highlight_line(line, &self.syntax_set)
                .unwrap_or_default();

            let spans: Vec<Span> = ranges
                .into_iter()
                .map(|(style, text)| {
                    let fg = style.foreground;
                    let color = Color::Rgb(fg.r, fg.g, fg.b);
                    let mut ratatui_style = Style::default().fg(color);

                    if style
                        .font_style
                        .contains(syntect::highlighting::FontStyle::BOLD)
                    {
                        ratatui_style = ratatui_style.add_modifier(Modifier::BOLD);
                    }
                    if style
                        .font_style
                        .contains(syntect::highlighting::FontStyle::ITALIC)
                    {
                        ratatui_style = ratatui_style.add_modifier(Modifier::ITALIC);
                    }
                    if style
                        .font_style
                        .contains(syntect::highlighting::FontStyle::UNDERLINE)
                    {
                        ratatui_style = ratatui_style.add_modifier(Modifier::UNDERLINED);
                    }

                    Span::styled(text.to_string(), ratatui_style)
                })
                .collect();

            lines.push(Line::from(spans));
        }

        lines
    }

    pub fn detect_language(info_string: &str) -> String {
        // Extract language from info string (e.g., "rust" from "```rust")
        info_string
            .split_whitespace()
            .next()
            .unwrap_or("text")
            .to_lowercase()
    }
}
