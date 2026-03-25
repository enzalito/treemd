use std::collections::HashMap;
use std::sync::OnceLock;
use tree_sitter_highlight::HighlightConfiguration;

use crate::tui::highlight::parsers::{tree_sitter_ada, tree_sitter_go};

pub struct HighlightConfigStore {
    cache: HashMap<&'static str, OnceLock<HighlightConfiguration>>,
}

impl HighlightConfigStore {
    pub fn new() -> Self {
        let keys = ["rust", "python"];
        let mut cache = HashMap::with_capacity(keys.len());

        for key in keys {
            cache.insert(key, OnceLock::new());
        }

        Self { cache }
    }

    pub fn get(
        &self,
        lang: &str,
        highlight_names: &[impl AsRef<str>],
    ) -> Option<&HighlightConfiguration> {
        let lock = self.cache.get(lang)?;

        let config = lock.get_or_init(|| {
            let mut config = match lang {
                "rust" => HighlightConfiguration::new(
                    tree_sitter_rust::LANGUAGE.into(),
                    lang,
                    tree_sitter_rust::HIGHLIGHTS_QUERY,
                    tree_sitter_rust::INJECTIONS_QUERY,
                    "",
                )
                .unwrap(),
                "python" => HighlightConfiguration::new(
                    tree_sitter_python::LANGUAGE.into(),
                    lang,
                    tree_sitter_python::HIGHLIGHTS_QUERY,
                    "",
                    "",
                )
                .unwrap(),
                "go" => HighlightConfiguration::new(
                    tree_sitter_go::LANGUAGE.into(),
                    lang,
                    tree_sitter_go::HIGHLIGHTS_QUERY,
                    "",
                    "",
                )
                .unwrap(),
                _ => unreachable!(
                    "Key '{}' was inserted into the map but lacks an init branch",
                    lang
                ),
            };

            config.configure(highlight_names);

            config
        });

        Some(config)
    }
}

impl Default for HighlightConfigStore {
    fn default() -> Self {
        Self::new()
    }
}
