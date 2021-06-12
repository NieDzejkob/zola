use lazy_static::lazy_static;
use once_cell::sync::OnceCell;
use syntect::dumps::from_binary;
use syntect::easy::HighlightLines;
use syntect::highlighting::ThemeSet;
use syntect::parsing::SyntaxSet;

use crate::config::Config;

lazy_static! {
    pub static ref BUILTIN_SYNTAX_SET: SyntaxSet = {
        let ss: SyntaxSet =
            from_binary(include_bytes!("../../../sublime/syntaxes/newlines.packdump"));
        ss
    };
    pub static ref BUILTIN_HIGHLIGHT_THEME_SET: ThemeSet =
        from_binary(include_bytes!("../../../sublime/themes/all.themedump"));
}

pub static EXTRA_SYNTAX_SET: OnceCell<SyntaxSet> = OnceCell::new();
pub static EXTRA_HIGHLIGHT_THEME_SET: OnceCell<ThemeSet> = OnceCell::new();

pub enum SyntaxSource {
    BuiltIn,
    Extra,
    Plain,
    NotFound,
}

impl SyntaxSource {
    pub fn syntax_set(&self) -> &'static SyntaxSet {
        match self {
            SyntaxSource::Extra => EXTRA_SYNTAX_SET.get().unwrap(),
            _ => &BUILTIN_SYNTAX_SET,
        }
    }
}

/// Returns the highlighter and whether it was found in the extra or not
pub fn get_highlighter(
    language: Option<&str>,
    config: &Config,
) -> (HighlightLines<'static>, SyntaxSource) {
    let theme = config.markdown.get_highlight_theme();

    let mut source = SyntaxSource::Plain;
    if let Some(lang) = language {
        let syntax = EXTRA_SYNTAX_SET
            .get()
            .and_then(|extra| {
                source = SyntaxSource::Extra;
                extra.find_syntax_by_token(lang)
            })
            .or_else(|| {
                let hacked_lang = if lang == "js" || lang == "javascript" { "ts" } else { lang };
                source = SyntaxSource::BuiltIn;
                BUILTIN_SYNTAX_SET.find_syntax_by_token(hacked_lang)
            })
            .unwrap_or_else(|| {
                source = SyntaxSource::NotFound;
                BUILTIN_SYNTAX_SET.find_syntax_plain_text()
            });
        (HighlightLines::new(syntax, theme), source)
    } else {
        (HighlightLines::new(BUILTIN_SYNTAX_SET.find_syntax_plain_text(), theme), source)
    }
}
