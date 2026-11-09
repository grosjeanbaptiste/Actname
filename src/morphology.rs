use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MorphologicalStrategy {
    Composed,
    Coined,
    Hybrid,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MorphologicalRequest {
    pub roots: Vec<String>,
    pub prefixes: Vec<String>,
    pub max_length: usize,
    pub limit: usize,
    pub strategy: MorphologicalStrategy,
    pub exclusions: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MorphologicalCandidate {
    pub name: String,
    pub root: String,
    pub strategy: MorphologicalStrategy,
    pub transformations: Vec<String>,
}

#[derive(Debug, Default)]
pub struct MorphologicalGenerator;

impl MorphologicalGenerator {
    pub fn generate(&self, request: &MorphologicalRequest) -> Vec<MorphologicalCandidate> {
        if request.max_length == 0 || request.limit == 0 {
            return Vec::new();
        }

        let mut candidates = BTreeMap::new();

        for prefix in &request.prefixes {
            if !valid_word(prefix) {
                continue;
            }

            for root in &request.roots {
                let normalized = crate::semantics::normalize(root);

                if !valid_word(&normalized) {
                    continue;
                }

                if request.strategy != MorphologicalStrategy::Coined {
                    Self::insert(
                        &mut candidates,
                        request,
                        prefix,
                        &normalized,
                        &normalized,
                        MorphologicalStrategy::Composed,
                        Vec::new(),
                    );
                }

                if request.strategy != MorphologicalStrategy::Composed {
                    for (stem, transformation) in stems(&normalized) {
                        Self::insert(
                            &mut candidates,
                            request,
                            prefix,
                            &normalized,
                            &stem,
                            MorphologicalStrategy::Coined,
                            vec![transformation],
                        );
                    }
                }
            }
        }

        candidates.into_values().take(request.limit).collect()
    }

    fn insert(
        candidates: &mut BTreeMap<String, MorphologicalCandidate>,
        request: &MorphologicalRequest,
        prefix: &str,
        root: &str,
        stem: &str,
        strategy: MorphologicalStrategy,
        mut transformations: Vec<String>,
    ) {
        let (name, fused) = fuse(prefix, stem);

        if fused {
            transformations.push("boundary_deduplication".into());
        }

        if name.len() > request.max_length
            || name.len() <= prefix.len()
            || request
                .exclusions
                .iter()
                .any(|excluded| excluded.eq_ignore_ascii_case(&name))
        {
            return;
        }

        let candidate = MorphologicalCandidate {
            name: name.clone(),
            root: root.to_owned(),
            strategy,
            transformations,
        };

        candidates.entry(name).or_insert(candidate);
    }
}

fn valid_word(value: &str) -> bool {
    !value.is_empty() && value.bytes().all(|byte| byte.is_ascii_lowercase())
}

fn fuse(prefix: &str, stem: &str) -> (String, bool) {
    let overlap = prefix
        .as_bytes()
        .last()
        .zip(stem.as_bytes().first())
        .is_some_and(|(left, right)| left == right);

    if overlap {
        (format!("{prefix}{}", &stem[1..]), true)
    } else {
        (format!("{prefix}{stem}"), false)
    }
}

fn stems(root: &str) -> Vec<(String, String)> {
    let mut variants = BTreeMap::new();

    for suffix in [
        "ation", "ition", "ology", "ologie", "logie", "ement", "ment", "ique", "ical", "ical",
        "ing", "tion", "ure", "ics", "ic",
    ] {
        if let Some(stem) = root.strip_suffix(suffix) {
            if stem.len() >= 3 {
                variants.insert(stem.to_owned(), format!("suffix_removal:{suffix}"));
            }
        }
    }

    for prefix in [
        "archi", "electro", "elect", "techno", "micro", "macro", "hyper", "inter",
    ] {
        if let Some(stem) = root.strip_prefix(prefix) {
            if stem.len() >= 3 {
                variants.insert(stem.to_owned(), format!("prefix_removal:{prefix}"));
            }
        }
    }

    if root == "architecture" {
        variants.insert("ecture".into(), "architectural_contraction".into());
    }

    if root.len() >= 7 {
        for length in [4, 5, 6, 7] {
            if let Some(stem) = root.get(..length) {
                variants.insert(stem.to_owned(), format!("root_truncation:{length}"));
            }
        }
    }

    variants.into_iter().collect()
}

pub fn morphological_quality(candidate: &MorphologicalCandidate) -> u8 {
    let mut quality: i32 = 100;

    for transformation in &candidate.transformations {
        if transformation.starts_with("root_truncation:") {
            let preserved = transformation
                .split_once(':')
                .and_then(|(_, value)| value.parse::<usize>().ok())
                .unwrap_or(0);

            let root_length = candidate.root.len();

            let preservation = preserved
                .saturating_mul(100)
                .checked_div(root_length)
                .unwrap_or(0);

            let penalty = match preservation {
                0..=39 => 35,
                40..=59 => 25,
                60..=79 => 15,
                _ => 8,
            };

            quality -= penalty;
        } else if transformation.starts_with("suffix_removal:") {
            quality -= 8;
        } else if transformation.starts_with("prefix_removal:") {
            quality -= 10;
        } else if transformation == "architectural_contraction" {
            quality -= 3;
        } else if transformation == "boundary_deduplication" {
            quality -= 0;
        } else {
            quality -= 15;
        }
    }

    quality.clamp(0, 100) as u8
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_invalid_words() {
        assert!(!valid_word(""));
        assert!(!valid_word("Act"));
        assert!(!valid_word("act-"));
        assert!(valid_word("act"));
    }

    #[test]
    fn fuses_equal_boundaries() {
        assert_eq!(fuse("acte", "electronic"), ("actelectronic".into(), true));
    }

    #[test]
    fn preserves_distinct_boundaries() {
        assert_eq!(fuse("act", "geometry"), ("actgeometry".into(), false));
    }

    #[test]
    fn derives_architectural_contraction() {
        assert!(
            stems("architecture")
                .iter()
                .any(|(stem, _)| stem == "ecture")
        );
    }

    #[test]
    fn does_not_truncate_short_roots() {
        assert!(stems("law").is_empty());
    }
}
