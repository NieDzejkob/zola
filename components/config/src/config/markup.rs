use std::path::Path;

use errors::{bail, Result};
use serde_derive::{Deserialize, Serialize};
use syntect::{
    highlighting::ThemeSet,
    parsing::{SyntaxSet, SyntaxSetBuilder},
};

use crate::highlighting::{
    BUILTIN_HIGHLIGHT_THEME_SET, EXTRA_HIGHLIGHT_THEME_SET, EXTRA_SYNTAX_SET,
};

pub const DEFAULT_HIGHLIGHT_THEME: &str = "base16-ocean-dark";

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Markdown {
    /// Whether to highlight all code blocks found in markdown files. Defaults to false
    pub highlight_code: bool,
    /// Which themes to use for code highlighting. See Readme for supported themes
    /// Defaults to "base16-ocean-dark"
    pub highlight_theme: String,
    /// Whether to render emoji aliases (e.g.: :smile: => 😄) in the markdown files
    pub render_emoji: bool,
    /// Whether external links are to be opened in a new tab
    /// If this is true, a `rel="noopener"` will always automatically be added for security reasons
    pub external_links_target_blank: bool,
    /// Whether to set rel="nofollow" for all external links
    pub external_links_no_follow: bool,
    /// Whether to set rel="noreferrer" for all external links
    pub external_links_no_referrer: bool,
    /// Whether smart punctuation is enabled (changing quotes, dashes, dots etc in their typographic form)
    pub smart_punctuation: bool,

    /// A list of directories to search for additional `.sublime-syntax` files in.
    pub extra_syntaxes: Vec<String>,
    /// A list of directories to search for additional `.tmTheme` files in.
    pub extra_highlight_themes: Vec<String>,
}

impl Markdown {
    /// Gets the configured highlight theme from the BUILTIN_HIGHLIGHT_THEME_SET or the EXTRA_HIGHLIGHT_THEME_SET
    pub fn get_highlight_theme(&self) -> &'static syntect::highlighting::Theme {
        if let Some(theme) = &BUILTIN_HIGHLIGHT_THEME_SET.themes.get(&self.highlight_theme) {
            theme
        } else {
            &EXTRA_HIGHLIGHT_THEME_SET.get().unwrap().themes[&self.highlight_theme]
        }
    }

    /// Attempt to load any theme sets found in the extra highlighting themes of the config
    /// TODO: move to markup.rs in 0.14
    pub fn load_extra_highlight_themes(&self, base_path: &Path) -> Result<Option<ThemeSet>> {
        let extra_highlight_themes = self.extra_highlight_themes.clone();
        if extra_highlight_themes.is_empty() {
            return Ok(None);
        }

        let mut ts = ThemeSet::new();
        for dir in &extra_highlight_themes {
            ts.add_from_folder(base_path.join(dir))?;
        }
        let extra_theme_set = Some(ts);

        Ok(extra_theme_set)
    }

    /// Attempt to load any extra syntax found in the extra syntaxes of the config
    pub fn load_extra_syntaxes(&self, base_path: &Path) -> Result<Option<SyntaxSet>> {
        if self.extra_syntaxes.is_empty() {
            return Ok(None);
        }

        let mut ss = SyntaxSetBuilder::new();
        for dir in &self.extra_syntaxes {
            ss.add_from_folder(base_path.join(dir), true)?;
        }

        Ok(Some(ss.build()))
    }

    // Initialise static once cells: EXTRA_SYNTAX_SET and EXTRA_HIGHLIGHT_THEME_SET
    // They can only be initialised once, when building a new site the existing values are reused
    pub(crate) fn init_extra_syntaxes_and_highlight_themes(&self, path: &Path) -> Result<()> {
        if let Some(extra_syntax_set) = self.load_extra_syntaxes(path)? {
            if EXTRA_SYNTAX_SET.get().is_none() {
                EXTRA_SYNTAX_SET.set(extra_syntax_set).unwrap();
            }
        }
        if let Some(extra_highlight_theme_set) = self.load_extra_highlight_themes(path)? {
            if EXTRA_HIGHLIGHT_THEME_SET.get().is_none() {
                EXTRA_HIGHLIGHT_THEME_SET.set(extra_highlight_theme_set).unwrap();
            }
        }

        // validate that the chosen highlight_theme exists in the loaded highlight theme sets
        if !BUILTIN_HIGHLIGHT_THEME_SET.themes.contains_key(&self.highlight_theme) {
            if let Some(extra) = EXTRA_HIGHLIGHT_THEME_SET.get() {
                if !extra.themes.contains_key(&self.highlight_theme) {
                    bail!(
                        "Highlight theme {} not found in the extra theme set",
                        &self.highlight_theme
                    )
                }
            } else {
                bail!("Highlight theme {} not available.\n\
                You can load custom themes by configuring `extra_highlight_themes` with a list of folders containing .tmTheme files", &self.highlight_theme)
            }
        }

        Ok(())
    }

    pub fn has_external_link_tweaks(&self) -> bool {
        self.external_links_target_blank
            || self.external_links_no_follow
            || self.external_links_no_referrer
    }

    pub fn construct_external_link_tag(&self, url: &str, title: &str) -> String {
        let mut rel_opts = Vec::new();
        let mut target = "".to_owned();
        let title = if title.is_empty() { "".to_owned() } else { format!("title=\"{}\" ", title) };

        if self.external_links_target_blank {
            // Security risk otherwise
            rel_opts.push("noopener");
            target = "target=\"_blank\" ".to_owned();
        }
        if self.external_links_no_follow {
            rel_opts.push("nofollow");
        }
        if self.external_links_no_referrer {
            rel_opts.push("noreferrer");
        }
        let rel = if rel_opts.is_empty() {
            "".to_owned()
        } else {
            format!("rel=\"{}\" ", rel_opts.join(" "))
        };

        format!("<a {}{}{}href=\"{}\">", rel, target, title, url)
    }
}

impl Default for Markdown {
    fn default() -> Markdown {
        Markdown {
            highlight_code: false,
            highlight_theme: DEFAULT_HIGHLIGHT_THEME.to_owned(),
            render_emoji: false,
            external_links_target_blank: false,
            external_links_no_follow: false,
            external_links_no_referrer: false,
            smart_punctuation: false,
            extra_syntaxes: vec![],
            extra_highlight_themes: vec![],
        }
    }
}
