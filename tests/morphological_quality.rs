use actname::morphology::{MorphologicalCandidate, MorphologicalStrategy, morphological_quality};

fn candidate(name: &str, root: &str, transformation: &str) -> MorphologicalCandidate {
    MorphologicalCandidate {
        name: name.into(),
        root: root.into(),
        strategy: MorphologicalStrategy::Coined,
        transformations: vec![transformation.into()],
    }
}

#[test]
fn prefers_linguistic_contractions_to_arbitrary_truncations() {
    let contraction = candidate("actecture", "architecture", "architectural_contraction");

    let truncation = candidate("actarch", "architecture", "root_truncation:4");

    assert!(morphological_quality(&contraction) > morphological_quality(&truncation));
}

#[test]
fn rewards_preservation_of_longer_roots() {
    let short = candidate("actarch", "architecture", "root_truncation:4");

    let longer = candidate("actarchit", "architecture", "root_truncation:7");

    assert!(morphological_quality(&longer) > morphological_quality(&short));
}

#[test]
fn preserves_quality_bounds() {
    let candidates = [
        candidate("actecture", "architecture", "architectural_contraction"),
        candidate("actarch", "architecture", "root_truncation:4"),
        candidate("actarchit", "architecture", "root_truncation:7"),
    ];

    assert!(
        candidates
            .iter()
            .all(|candidate| { morphological_quality(candidate) <= 100 })
    );
}

#[test]
fn boundary_deduplication_is_not_a_truncation() {
    let candidate = MorphologicalCandidate {
        name: "actelectronic".into(),
        root: "electronic".into(),
        strategy: MorphologicalStrategy::Composed,
        transformations: vec!["boundary_deduplication".into()],
    };

    assert!(morphological_quality(&candidate) >= 90);
}
