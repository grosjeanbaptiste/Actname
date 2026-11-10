use actname::scoring::{ScoringError, ScoringModel, Weights};
use actname::{InMemoryLexicon, NamingCriteria, NamingService, Strategy};

fn criteria(strategy: Strategy) -> NamingCriteria {
    NamingCriteria {
        prefixes: vec!["act".into(), "acte".into()],
        context: "architecture bâtiments plans conception".into(),
        max_length: 12,
        limit: 50,
        strategy,
        exclusions: vec![],
    }
}

fn generate(model: ScoringModel, strategy: Strategy) -> Vec<actname::Candidate> {
    NamingService::new(&InMemoryLexicon)
        .with_model(model)
        .generate(&criteria(strategy))
        .unwrap_or_else(|error| panic!("{error}"))
}

#[test]
fn default_model_reproduces_the_default_service() {
    let plain = NamingService::new(&InMemoryLexicon)
        .generate(&criteria(Strategy::Hybrid))
        .unwrap_or_else(|error| panic!("{error}"));
    assert_eq!(generate(ScoringModel::default(), Strategy::Hybrid), plain);
}

#[test]
fn weights_decide_the_total() {
    let only_semantic = ScoringModel {
        weights: Weights {
            semantic: 1,
            readability: 0,
            originality: 0,
        },
        ..ScoringModel::default()
    };
    let names = generate(only_semantic, Strategy::Composed);
    assert!(!names.is_empty());
    assert!(names.iter().all(|c| c.score.total == c.score.semantic));
}

#[test]
fn originality_per_form_is_a_setting() {
    let model = ScoringModel {
        weights: Weights {
            semantic: 0,
            readability: 0,
            originality: 1,
        },
        originality_composed: 33,
        ..ScoringModel::default()
    };
    let names = generate(model, Strategy::Composed);
    assert!(!names.is_empty());
    assert!(
        names
            .iter()
            .all(|c| c.score.total == 33 && c.score.originality == 33)
    );
}

#[test]
fn readability_penalties_are_settings() {
    let strict = ScoringModel {
        length_free: 0,
        length_penalty: 5,
        ..ScoringModel::default()
    };
    let readability = |m| {
        generate(m, Strategy::Composed)
            .iter()
            .map(|c| c.score.readability as u32)
            .sum::<u32>()
    };
    assert!(readability(strict) < readability(ScoringModel::default()));
}

#[test]
fn semantic_gain_per_match_is_a_setting() {
    let model = ScoringModel {
        semantic_base: 10,
        semantic_per_match: 1,
        ..ScoringModel::default()
    };
    let names = generate(model, Strategy::Composed);
    assert!(names.iter().all(|c| c.score.semantic <= 10 + 8));
}

#[test]
fn weights_summing_to_zero_are_refused_by_generate() {
    let model = ScoringModel {
        weights: Weights {
            semantic: 0,
            readability: 0,
            originality: 0,
        },
        ..ScoringModel::default()
    };
    let result = NamingService::new(&InMemoryLexicon)
        .with_model(model)
        .generate(&criteria(Strategy::Hybrid));
    assert!(result.is_err());
}

#[test]
fn model_is_read_from_key_value_text_over_the_defaults() {
    let text = "# my model\nweight_semantic = 60\nweight_originality=10\n\nlength_free=8\n";
    let model = ScoringModel::from_text(text).unwrap_or_else(|error| panic!("{error}"));
    assert_eq!(
        model.weights,
        Weights {
            semantic: 60,
            readability: 30,
            originality: 10
        }
    );
    assert_eq!(model.length_free, 8);
    assert_eq!(
        model.originality_coined,
        ScoringModel::default().originality_coined
    );
}

#[test]
fn text_errors_name_the_line() {
    assert_eq!(
        ScoringModel::from_text("nope"),
        Err(ScoringError::Line(1, "expected key=value".into()))
    );
    assert_eq!(
        ScoringModel::from_text("length_free=1\nweight_semantic=x"),
        Err(ScoringError::Line(
            2,
            "weight_semantic: not a number: \"x\"".into()
        ))
    );
    assert_eq!(
        ScoringModel::from_text("colour=1"),
        Err(ScoringError::Line(1, "unknown key colour".into()))
    );
}
