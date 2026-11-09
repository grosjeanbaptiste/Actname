use actname::{Form, InMemoryLexicon, NamingCriteria, NamingService, Strategy};

fn criteria(context: &str) -> NamingCriteria {
    NamingCriteria {
        prefixes: vec!["act".into(), "acte".into()],
        context: context.into(),
        max_length: 12,
        limit: 100,
        strategy: Strategy::Hybrid,
        exclusions: vec![],
    }
}

fn generate(context: &str) -> Vec<actname::Candidate> {
    let lexicon = InMemoryLexicon;
    let service = NamingService::new(&lexicon);

    service
        .generate(&criteria(context))
        .unwrap_or_else(|error| panic!("{error}"))
}

#[test]
fn integrates_morphological_candidates() {
    let results = generate("architecture");

    assert!(
        results
            .iter()
            .any(|candidate| { candidate.name == "actarchi" && candidate.form == Form::Coined })
    );
}

#[test]
fn preserves_neologism_trace_and_rewards_recomposition() {
    let results = generate("architecture");
    let candidate = results
        .iter()
        .find(|candidate| candidate.name == "actetecture");
    let Some(candidate) = candidate else {
        panic!("the linked neologism should be generated");
    };

    assert_eq!(candidate.provenance.concepts, ["architecture"]);
    assert_eq!(
        candidate.provenance.transformations,
        ["prefix_removal:archi", "linking_vowel:e"]
    );
    assert!(candidate.score.originality > 90);

    let shorter = results
        .iter()
        .find(|candidate| candidate.name == "actecture");
    let Some(shorter) = shorter else {
        panic!("the existing contraction should remain available");
    };
    assert_eq!(candidate.score.readability, shorter.score.readability);
    assert!(candidate.score.total >= shorter.score.total);
}

#[test]
fn enforces_length_boundary_for_recomposed_neologisms() {
    let lexicon = InMemoryLexicon;
    let service = NamingService::new(&lexicon);
    let mut input = criteria("architecture");

    input.max_length = "actetecture".len();
    let results = service
        .generate(&input)
        .unwrap_or_else(|error| panic!("{error}"));
    assert!(
        results
            .iter()
            .any(|candidate| candidate.name == "actetecture")
    );

    input.max_length -= 1;
    let results = service
        .generate(&input)
        .unwrap_or_else(|error| panic!("{error}"));
    assert!(
        !results
            .iter()
            .any(|candidate| candidate.name == "actetecture")
    );
}

#[test]
fn excludes_recomposed_neologisms_case_insensitively() {
    let lexicon = InMemoryLexicon;
    let service = NamingService::new(&lexicon);
    let mut input = criteria("architecture");
    input.exclusions = vec!["Actetecture".into()];

    let results = service
        .generate(&input)
        .unwrap_or_else(|error| panic!("{error}"));

    assert!(
        !results
            .iter()
            .any(|candidate| candidate.name == "actetecture")
    );
}

#[test]
fn retains_existing_lexical_candidates() {
    let results = generate("architecture bâtiments plans conception");

    assert!(
        results
            .iter()
            .any(|candidate| { candidate.name == "actecture" })
    );

    assert!(
        results
            .iter()
            .any(|candidate| { candidate.name == "acteplan" })
    );
}

#[test]
fn rejects_unrelated_context() {
    assert!(generate("xyzunrelatedword").is_empty());
}

#[test]
fn supports_multilingual_semantics() {
    let results = generate("building blueprints");

    assert!(results.iter().any(|candidate| {
        candidate.name == "actbatim" || candidate.name == "actplans" || candidate.name == "actplan"
    }));
}

#[test]
fn preserves_canonical_neologism_provenance_for_multilingual_context() {
    let lexicon = InMemoryLexicon;
    let service = NamingService::new(&lexicon);
    let results = service
        .generate(&criteria("arquitectura"))
        .unwrap_or_else(|error| panic!("{error}"));
    let candidate = results
        .iter()
        .find(|candidate| candidate.name == "actetecture");
    let Some(candidate) = candidate else {
        panic!("the architectural neologism should match English semantic aliases");
    };

    assert_eq!(candidate.provenance.concepts, ["architecture"]);
    assert_eq!(
        candidate.provenance.transformations,
        ["prefix_removal:archi", "linking_vowel:e"]
    );
}

#[test]
fn fuses_shared_boundary_in_actrack_and_tracks_its_origin() {
    let results = generate("sports performance tracking");
    let candidate = results.iter().find(|candidate| candidate.name == "actrack");
    let Some(candidate) = candidate else {
        panic!("the shared boundary should be fused in actrack");
    };

    assert_eq!(candidate.form, Form::Composed);
    assert!(candidate.provenance.concepts.contains(&"suivi".into()));
    assert_eq!(
        candidate.provenance.transformations,
        ["boundary_deduplication"]
    );
}

#[test]
fn generates_acteble_from_capability_with_traceable_suffix() {
    let results = generate("capability");
    let candidate = results.iter().find(|candidate| candidate.name == "acteble");
    let Some(candidate) = candidate else {
        panic!("capability context should generate acteble");
    };

    assert_eq!(candidate.form, Form::Coined);
    assert_eq!(candidate.provenance.concepts, ["capacite"]);
    assert_eq!(
        candidate.provenance.transformations,
        ["prefix_suffix_composition", "lexical_suffix:ble"]
    );
}

#[test]
fn generates_acteble_from_french_capacite() {
    let results = generate("capacité");

    assert!(results.iter().any(|candidate| {
        candidate.name == "acteble" && candidate.provenance.concepts == ["capacite"]
    }));
}

#[test]
fn respects_coined_strategy() {
    let lexicon = InMemoryLexicon;
    let service = NamingService::new(&lexicon);

    let mut input = criteria("architecture");
    input.strategy = Strategy::Coined;

    let results = service
        .generate(&input)
        .unwrap_or_else(|error| panic!("{error}"));

    assert!(
        results
            .iter()
            .any(|candidate| { candidate.name == "actarchi" })
    );

    let neologism = results
        .iter()
        .find(|candidate| candidate.name == "actetecture");
    let Some(neologism) = neologism else {
        panic!("the recomposed neologism should be available in coined strategy");
    };
    assert_eq!(
        neologism.provenance.transformations,
        ["prefix_removal:archi", "linking_vowel:e"]
    );

    assert!(
        results
            .iter()
            .all(|candidate| { candidate.form == Form::Coined })
    );
}

#[test]
fn respects_composed_strategy() {
    let lexicon = InMemoryLexicon;
    let service = NamingService::new(&lexicon);

    let mut input = criteria("architecture");
    input.strategy = Strategy::Composed;

    let results = service
        .generate(&input)
        .unwrap_or_else(|error| panic!("{error}"));

    assert!(
        results
            .iter()
            .all(|candidate| { candidate.form == Form::Composed })
    );

    assert!(
        !results
            .iter()
            .any(|candidate| { candidate.name == "actarchi" })
    );

    assert!(
        !results
            .iter()
            .any(|candidate| { candidate.name == "actetecture" })
    );
}

#[test]
fn applies_exclusions_to_all_sources() {
    let lexicon = InMemoryLexicon;
    let service = NamingService::new(&lexicon);

    let mut input = criteria("architecture");
    input.exclusions = vec!["actarchi".into(), "actecture".into()];

    let results = service
        .generate(&input)
        .unwrap_or_else(|error| panic!("{error}"));

    assert!(
        results
            .iter()
            .all(|candidate| { candidate.name != "actarchi" && candidate.name != "actecture" })
    );
}

#[test]
fn preserves_deterministic_ranking() {
    let first = generate("architecture bâtiments conception");
    let second = generate("architecture bâtiments conception");

    assert_eq!(first, second);

    assert!(
        first
            .windows(2)
            .all(|pair| { pair[0].score.total >= pair[1].score.total })
    );

    let names: std::collections::BTreeSet<_> =
        first.iter().map(|candidate| &candidate.name).collect();

    assert_eq!(names.len(), first.len());
}

#[test]
fn enforces_maximum_length() {
    let results = generate("architecture bâtiments conception");

    assert!(
        results
            .iter()
            .all(|candidate| { candidate.name.len() <= 12 })
    );
}

#[test]
fn preserves_semantic_score_consistency() {
    let results = generate("architecture");

    assert!(
        results
            .iter()
            .all(|candidate| { candidate.score.semantic > 0 && candidate.score.total <= 100 })
    );
}
