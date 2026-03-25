mod config_store;
mod parsers;
mod theme;

use std::path::PathBuf;

use config_store::HighlightConfigStore;
use theme::Theme;

struct Highlighter {
    config_store: HighlightConfigStore,
    theme: Theme,
}

impl Highlighter {
    pub fn new(theme: String, theme_dir: PathBuf) -> Self {
        let config_store = HighlightConfigStore::new();

        let theme_path = theme_dir.join(theme + ".toml");
        let theme = Theme::new(theme_path);

        Self {
            config_store,
            theme,
        }
    }

    pub fn highlight_code() {
        todo!()
    }
}
