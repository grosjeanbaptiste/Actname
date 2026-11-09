use actname::morphology::{MorphologicalGenerator, MorphologicalRequest, MorphologicalStrategy};

fn request(roots: &[&str], prefixes: &[&str], max_length: usize) -> MorphologicalRequest {
    MorphologicalRequest {
        roots: roots.iter().map(|value| (*value).to_owned()).collect(),
        prefixes: prefixes.iter().map(|value| (*value).to_owned()).collect(),
        max_length,
        limit: 100,
        strategy: MorphologicalStrategy::Hybrid,
        exclusions: vec![],
    }
}

#[test]
fn generates_names_from_arbitrary_roots() {
    let generator = MorphologicalGenerator;

    let results = generator.generate(&request(
        &["architecture", "geometry"],
        &["act", "acte"],
        12,
    ));

    assert!(
        results
            .iter()
            .any(|candidate| { candidate.name.starts_with("act") && candidate.name.len() <= 12 })
    );
}

#[test]
fn fuses_identical_boundary_characters() {
    let generator = MorphologicalGenerator;

    let results = generator.generate(&request(&["electronic"], &["acte"], 14));

    assert!(
        results
            .iter()
            .any(|candidate| { candidate.name == "actelectronic" })
    );

    assert!(
        !results
            .iter()
            .any(|candidate| { candidate.name == "acteelectronic" })
    );
}

#[test]
fn creates_shortened_variants() {
    let generator = MorphologicalGenerator;

    let results = generator.generate(&request(&["architecture"], &["act"], 10));

    assert!(
        results
            .iter()
            .any(|candidate| { candidate.name == "actecture" })
    );
}

#[test]
fn respects_maximum_length() {
    let generator = MorphologicalGenerator;

    let results = generator.generate(&request(
        &["architecture", "technology", "mathematics"],
        &["act", "acte"],
        8,
    ));

    assert!(
        results
            .iter()
            .all(|candidate| { candidate.name.len() <= 8 })
    );
}

#[test]
fn excludes_forbidden_names() {
    let generator = MorphologicalGenerator;

    let mut input = request(&["architecture"], &["act"], 12);
    input.exclusions.push("actecture".into());

    let results = generator.generate(&input);

    assert!(
        !results
            .iter()
            .any(|candidate| { candidate.name == "actecture" })
    );
}

#[test]
fn returns_unique_deterministic_results() {
    let generator = MorphologicalGenerator;

    let input = request(
        &["architecture", "architecture", "geometry"],
        &["act", "acte", "act"],
        12,
    );

    let first = generator.generate(&input);
    let second = generator.generate(&input);

    assert_eq!(first, second);

    let names: std::collections::BTreeSet<_> =
        first.iter().map(|candidate| &candidate.name).collect();

    assert_eq!(names.len(), first.len());
}

#[test]
fn supports_composed_strategy() {
    let generator = MorphologicalGenerator;

    let mut input = request(&["geometry"], &["act"], 12);
    input.strategy = MorphologicalStrategy::Composed;

    let results = generator.generate(&input);

    assert!(
        results
            .iter()
            .all(|candidate| { candidate.strategy == MorphologicalStrategy::Composed })
    );
}

#[test]
fn supports_coined_strategy() {
    let generator = MorphologicalGenerator;

    let mut input = request(&["architecture"], &["act"], 12);
    input.strategy = MorphologicalStrategy::Coined;

    let results = generator.generate(&input);

    assert!(!results.is_empty());

    assert!(
        results
            .iter()
            .all(|candidate| { candidate.strategy == MorphologicalStrategy::Coined })
    );
}

#[test]
fn limits_results() {
    let generator = MorphologicalGenerator;

    let mut input = request(
        &["architecture", "geometry", "technology"],
        &["act", "acte"],
        12,
    );

    input.limit = 2;

    assert_eq!(generator.generate(&input).len(), 2);
}

#[test]
fn rejects_invalid_inputs_without_panicking() {
    let generator = MorphologicalGenerator;

    let invalid = request(&["architecture"], &["ACT!"], 12);

    assert!(generator.generate(&invalid).is_empty());

    let empty = request(&[], &["act"], 12);

    assert!(generator.generate(&empty).is_empty());
}

#[test]
fn preserves_root_provenance() {
    let generator = MorphologicalGenerator;

    let results = generator.generate(&request(&["architecture"], &["act"], 12));

    assert!(
        results
            .iter()
            .all(|candidate| { candidate.root == "architecture" })
    );
}
