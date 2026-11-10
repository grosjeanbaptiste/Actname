use std::sync::OnceLock;

use crate::domain::{Lexeme, Lexicon};
use crate::lexicon::TextLexicon;

/// The lexicon embedded from `resources/lexicon.tsv`.
#[derive(Default)]
pub struct InMemoryLexicon;

impl Lexicon for InMemoryLexicon {
    fn entries(&self) -> &[Lexeme] {
        static EMBEDDED: OnceLock<TextLexicon> = OnceLock::new();
        EMBEDDED
            .get_or_init(|| {
                TextLexicon::from_text(include_str!("../resources/lexicon.tsv"))
                    .unwrap_or_else(|error| panic!("invalid embedded lexicon: {error}"))
            })
            .entries()
    }
}
