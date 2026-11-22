//! `actname generate ...` as a library entry point, so another tool (`repo`) can run it
//! in-process; the `actname` binary only wraps it.
use std::io::Write;

use crate::context::from_readme;
use crate::lexicon::TextLexicon;
use crate::scoring::ScoringModel;
use crate::semantics::SemanticKnowledgeBase;
use crate::{Candidate, InMemoryLexicon, Lexicon, NamingCriteria, NamingService, Strategy};

#[derive(Clone, Copy)]
enum OutputFormat {
    Text,
    Json,
}

#[derive(Default)]
struct DataFiles {
    lexicon: Option<String>,
    scoring: Option<String>,
    semantics: Option<String>,
}

struct CliOptions {
    criteria: NamingCriteria,
    /// Where the context comes from when `--domain` gives none.
    readme: Option<String>,
    format: OutputFormat,
    files: DataFiles,
}

/// The one-line usage shown for `--help` and after an error.
pub fn usage() -> &'static str {
    "actname generate --prefix act --prefix acte --domain \"architecture plans\" [--max-length 10] [--limit 20] [--strategy hybrid|composed|coined] [--exclude NAME] [--format text|json] [--lexicon FILE] [--scoring FILE] [--semantics FILE] [--readme FILE]"
}

fn parse_arguments(arguments: Vec<String>) -> Result<Option<CliOptions>, String> {
    if arguments.is_empty() || arguments[0] == "--help" || arguments[0] == "-h" {
        return Ok(None);
    }

    if arguments[0] != "generate" {
        return Err(format!("unknown command: {}", arguments[0]));
    }

    let mut prefixes = Vec::new();
    let mut context = String::new();
    let mut max_length = 10;
    let mut limit = 20;
    let mut strategy = Strategy::Hybrid;
    let mut exclusions = Vec::new();
    let mut format = OutputFormat::Text;
    let mut files = DataFiles::default();
    let mut readme = None;

    let mut arguments = arguments.into_iter().skip(1);

    while let Some(option) = arguments.next() {
        let value = arguments
            .next()
            .ok_or_else(|| format!("missing value for {option}"))?;

        match option.as_str() {
            "--prefix" => prefixes.push(value.to_ascii_lowercase()),
            "--domain" => context = value,
            "--max-length" => {
                max_length = value
                    .parse()
                    .map_err(|_| "invalid maximum length".to_string())?;
            }
            "--limit" => {
                limit = value
                    .parse()
                    .map_err(|_| "invalid result limit".to_string())?;
            }
            "--strategy" => {
                strategy = match value.as_str() {
                    "hybrid" => Strategy::Hybrid,
                    "composed" => Strategy::Composed,
                    "coined" => Strategy::Coined,
                    _ => return Err("invalid generation strategy".into()),
                };
            }
            "--exclude" => exclusions.push(value),
            "--lexicon" => files.lexicon = Some(value),
            "--scoring" => files.scoring = Some(value),
            "--semantics" => files.semantics = Some(value),
            "--readme" => readme = Some(value),
            "--format" => {
                format = match value.as_str() {
                    "text" => OutputFormat::Text,
                    "json" => OutputFormat::Json,
                    _ => return Err("invalid output format".into()),
                };
            }
            _ => return Err(format!("unknown option: {option}")),
        }
    }

    Ok(Some(CliOptions {
        criteria: NamingCriteria {
            prefixes,
            context,
            max_length,
            limit,
            strategy,
            exclusions,
        },
        readme,
        format,
        files,
    }))
}

fn render_text(candidates: &[Candidate]) -> String {
    let mut text = format!(
        "{:<16} {:>9} {:>11} {:>11} {:>8}\n",
        "NAME", "SEMANTIC", "READABILITY", "ORIGINALITY", "TOTAL"
    );

    for candidate in candidates {
        text.push_str(&format!(
            "{:<16} {:>9} {:>11} {:>11} {:>8}\n",
            candidate.name,
            candidate.score.semantic,
            candidate.score.readability,
            candidate.score.originality,
            candidate.score.total
        ));
    }
    text
}

fn render_json(candidates: &[Candidate]) -> String {
    let mut text = String::from("[\n");

    for (index, candidate) in candidates.iter().enumerate() {
        let separator = if index + 1 == candidates.len() {
            ""
        } else {
            ","
        };

        text.push_str(&format!(
            "  {{\"name\":\"{}\",\"form\":\"{}\",\"score\":{{\"semantic\":{},\"readability\":{},\"originality\":{},\"total\":{}}},\"provenance\":{{\"concepts\":[{}],\"transformations\":[{}]}}}}{}\n",
            candidate.name,
            candidate.form.as_str(),
            candidate.score.semantic,
            candidate.score.readability,
            candidate.score.originality,
            candidate.score.total,
            json_array(&candidate.provenance.concepts),
            json_array(&candidate.provenance.transformations),
            separator
        ));
    }

    text.push_str("]\n");
    text
}

fn json_array(values: &[String]) -> String {
    values
        .iter()
        .map(|value| json_string(value))
        .collect::<Vec<_>>()
        .join(",")
}

fn json_string(value: &str) -> String {
    let mut escaped = String::from("\"");

    for character in value.chars() {
        match character {
            '"' => escaped.push_str("\\\""),
            '\\' => escaped.push_str("\\\\"),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            character if character <= '\u{001f}' => {
                escaped.push_str(&format!("\\u{:04x}", character as u32));
            }
            character => escaped.push(character),
        }
    }

    escaped.push('"');
    escaped
}

fn read(path: &str) -> Result<String, String> {
    std::fs::read_to_string(path).map_err(|error| format!("{path}: {error}"))
}

const DEFAULT_README: &str = "README.md";

/// Without `--domain`, the context is the repository's README when it holds text. A README that
/// is absent by default is no context; an explicit `--readme` that is absent, or any README that
/// cannot be read, is an error.
fn resolve_context(options: &mut CliOptions) -> Result<(), String> {
    if !options.criteria.context.trim().is_empty() {
        return Ok(());
    }
    let path = options.readme.as_deref().unwrap_or(DEFAULT_README);
    match std::fs::read_to_string(path) {
        Ok(text) => {
            if let Some(context) = from_readme(&text) {
                options.criteria.context = context;
            }
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound && options.readme.is_none() => {}
        Err(error) => return Err(format!("{path}: {error}")),
    }
    if options.criteria.context.trim().is_empty() {
        return Err(format!(
            "semantic context is required: pass --domain, or run in a repository with a non-empty {path}"
        ));
    }
    Ok(())
}

fn generate<L: Lexicon>(lexicon: &L, options: &CliOptions) -> Result<Vec<Candidate>, String> {
    let mut service = NamingService::new(lexicon);

    if let Some(path) = &options.files.scoring {
        let model =
            ScoringModel::from_text(&read(path)?).map_err(|error| format!("{path}: {error}"))?;
        service = service.with_model(model);
    }

    if let Some(path) = &options.files.semantics {
        let base = SemanticKnowledgeBase::from_tsv(&read(path)?)
            .map_err(|error| format!("{path}: {error}"))?;
        service = service.with_knowledge(base);
    }

    let found = service
        .generate(&options.criteria)
        .map_err(|error| error.to_string())?;
    if found.is_empty() {
        return Err(format!(
            "no name matches the context {:?}: its words may be unknown concepts (add some with --semantics and --lexicon)",
            options.criteria.context
        ));
    }
    Ok(found)
}

/// Run `actname <args>`: the answer goes to `out`; an error is its message.
pub fn dispatch(args: &[String], out: &mut dyn Write) -> Result<(), String> {
    let Some(mut options) = parse_arguments(args.to_vec())? else {
        return emit(out, &format!("{}\n", usage()));
    };
    resolve_context(&mut options)?;

    let candidates = match &options.files.lexicon {
        Some(path) => {
            let lexicon =
                TextLexicon::from_text(&read(path)?).map_err(|error| format!("{path}: {error}"))?;
            generate(&lexicon, &options)?
        }
        None => generate(&InMemoryLexicon, &options)?,
    };

    let text = match options.format {
        OutputFormat::Text => render_text(&candidates),
        OutputFormat::Json => render_json(&candidates),
    };
    emit(out, &text)
}

fn emit(out: &mut dyn Write, text: &str) -> Result<(), String> {
    out.write_all(text.as_bytes())
        .map_err(|error| format!("cannot write the answer: {error}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(args: &[&str]) -> (Result<(), String>, String) {
        let args: Vec<String> = args.iter().map(ToString::to_string).collect();
        let mut out = Vec::new();
        let result = dispatch(&args, &mut out);
        (result, String::from_utf8_lossy(&out).into_owned())
    }

    #[test]
    fn no_arguments_and_help_print_the_usage() {
        for args in [&[][..], &["--help"], &["-h"]] {
            let (result, out) = run(args);
            assert_eq!(result, Ok(()));
            assert_eq!(out.trim(), usage());
        }
    }

    #[test]
    fn an_unknown_command_or_option_is_an_error_and_prints_nothing() {
        let (result, out) = run(&["frobnicate"]);
        assert_eq!(result, Err("unknown command: frobnicate".into()));
        let (result, _) = run(&["generate", "--zap", "1"]);
        assert_eq!(result, Err("unknown option: --zap".into()));
        assert!(out.is_empty());
    }

    #[test]
    fn a_bad_limit_and_a_missing_context_are_refused() {
        let (result, _) = run(&["generate", "--prefix", "act", "--limit", "x"]);
        assert_eq!(result, Err("invalid result limit".into()));
        let (result, _) = run(&[
            "generate",
            "--prefix",
            "act",
            "--readme",
            "/no/such/readme.md",
        ]);
        assert!(matches!(result, Err(m) if m.contains("/no/such/readme.md")));
    }

    #[test]
    fn generate_prints_a_table_with_one_row_per_name() {
        let (result, out) = run(&[
            "generate",
            "--prefix",
            "act",
            "--domain",
            "architecture plans",
            "--limit",
            "3",
        ]);
        assert_eq!(result, Ok(()));
        let lines: Vec<&str> = out.lines().collect();
        assert!(lines[0].starts_with("NAME"), "{out}");
        assert_eq!(lines.len(), 4, "{out}");
        assert!(lines[1..].iter().all(|l| l.starts_with("act")), "{out}");
    }

    #[test]
    fn generate_prints_json_when_asked() {
        let (result, out) = run(&[
            "generate",
            "--prefix",
            "act",
            "--domain",
            "architecture plans",
            "--limit",
            "2",
            "--format",
            "json",
        ]);
        assert_eq!(result, Ok(()));
        assert!(
            out.starts_with("[\n") && out.trim_end().ends_with(']'),
            "{out}"
        );
        assert_eq!(out.matches("\"name\":\"act").count(), 2, "{out}");
    }
}
