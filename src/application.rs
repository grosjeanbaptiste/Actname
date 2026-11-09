use std::collections::BTreeSet;

use crate::domain::{
    Candidate, Form, Lexicon, NamingCriteria, NamingError, Strategy, evaluate, evaluate_matches,
    semantic_tokens,
};
use crate::morphology::{MorphologicalGenerator, MorphologicalRequest, MorphologicalStrategy};

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

        let mut candidates = self.lexical_candidates(criteria, &context, &exclusions);

        candidates.extend(self.morphological_candidates(criteria, &context));

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

    fn lexical_candidates(
        &self,
        criteria: &NamingCriteria,
        context: &BTreeSet<String>,
        exclusions: &BTreeSet<String>,
    ) -> Vec<Candidate> {
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

                if let Some(score) = evaluate(&name, lexeme, context) {
                    candidates.push(Candidate {
                        name,
                        form: lexeme.form,
                        score,
                    });
                }
            }
        }

        candidates
    }

    fn morphological_candidates(
        &self,
        criteria: &NamingCriteria,
        context: &BTreeSet<String>,
    ) -> Vec<Candidate> {
        let roots: Vec<String> = context
            .iter()
            .filter(|concept| {
                self.lexicon.entries().iter().any(|lexeme| {
                    lexeme
                        .concepts
                        .iter()
                        .any(|known| crate::domain::normalize(known) == **concept)
                })
            })
            .cloned()
            .collect();

        if roots.is_empty() {
            return Vec::new();
        }

        let strategy = match criteria.strategy {
            Strategy::Hybrid => MorphologicalStrategy::Hybrid,
            Strategy::Composed => MorphologicalStrategy::Composed,
            Strategy::Coined => MorphologicalStrategy::Coined,
        };

        let request = MorphologicalRequest {
            roots,
            prefixes: criteria.prefixes.clone(),
            max_length: criteria.max_length,
            limit: usize::MAX,
            strategy,
            exclusions: criteria.exclusions.clone(),
        };

        let generator = MorphologicalGenerator;

        generator
            .generate(&request)
            .into_iter()
            .filter_map(|candidate| {
                let form = match candidate.strategy {
                    MorphologicalStrategy::Composed => Form::Composed,
                    MorphologicalStrategy::Coined => Form::Coined,
                    MorphologicalStrategy::Hybrid => return None,
                };

                let matches = self
                    .lexicon
                    .entries()
                    .iter()
                    .filter(|lexeme| {
                        lexeme
                            .concepts
                            .iter()
                            .any(|concept| crate::domain::normalize(concept) == candidate.root)
                    })
                    .flat_map(|lexeme| lexeme.concepts.iter())
                    .filter(|concept| context.contains(&crate::domain::normalize(concept)))
                    .map(|concept| crate::domain::normalize(concept))
                    .collect::<BTreeSet<_>>()
                    .len();

                evaluate_matches(&candidate.name, form, matches).map(|score| Candidate {
                    name: candidate.name,
                    form,
                    score,
                })
            })
            .collect()
    }
}
