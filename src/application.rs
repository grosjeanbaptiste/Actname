use std::collections::BTreeSet;

use crate::domain::{Candidate, Lexicon, NamingCriteria, NamingError, evaluate, semantic_tokens};

pub struct NamingService<'a, L: Lexicon> {
    lexicon: &'a L,
}

impl<'a, L: Lexicon> NamingService<'a, L> {
    pub fn new(lexicon: &'a L) -> Self {
        Self { lexicon }
    }

    pub fn generate(&self, criteria: &NamingCriteria) -> Result<Vec<Candidate>, NamingError> {
        criteria.validate()?;

        let context = semantic_tokens(&criteria.context);

        let exclusions: BTreeSet<String> = criteria
            .exclusions
            .iter()
            .map(|name| name.to_ascii_lowercase())
            .collect();

        let mut candidates = Vec::new();

        for lexeme in self.lexicon.entries() {
            if !criteria.strategy.accepts(lexeme.form) {
                continue;
            }

            for prefix in &criteria.prefixes {
                if prefix.ends_with('e') && lexeme.suffix.starts_with('e') {
                    continue;
                }

                let name = format!("{prefix}{}", lexeme.suffix);

                if name.len() > criteria.max_length
                    || !name.chars().all(|character| character.is_ascii_lowercase())
                    || exclusions.contains(&name)
                {
                    continue;
                }

                if let Some(score) = evaluate(&name, lexeme, &context) {
                    candidates.push(Candidate {
                        name,
                        form: lexeme.form,
                        score,
                    });
                }
            }
        }

        candidates.sort_by(|left, right| {
            right
                .score
                .total
                .cmp(&left.score.total)
                .then_with(|| right.score.semantic.cmp(&left.score.semantic))
                .then_with(|| left.name.cmp(&right.name))
        });

        let mut seen = BTreeSet::new();

        candidates.retain(|candidate| seen.insert(candidate.name.clone()));
        candidates.truncate(criteria.limit);

        Ok(candidates)
    }
}
