use std::env;
use std::process::ExitCode;

use actname::context::from_readme;
use actname::lexicon::TextLexicon;
use actname::scoring::ScoringModel;
use actname::semantics::SemanticKnowledgeBase;
use actname::{Candidate, InMemoryLexicon, Lexicon, NamingCriteria, NamingService, Strategy};

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

fn usage() -> &'static str {
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

fn render_text(candidates: &[Candidate]) {
    println!(
        "{:<16} {:>9} {:>11} {:>11} {:>8}",
        "NAME", "SEMANTIC", "READABILITY", "ORIGINALITY", "TOTAL"
    );

    for candidate in candidates {
        println!(
            "{:<16} {:>9} {:>11} {:>11} {:>8}",
            candidate.name,
            candidate.score.semantic,
            candidate.score.readability,
            candidate.score.originality,
            candidate.score.total
        );
    }
}

fn render_json(candidates: &[Candidate]) {
    println!("[");

    for (index, candidate) in candidates.iter().enumerate() {
        let separator = if index + 1 == candidates.len() {
            ""
        } else {
            ","
        };

        println!(
            "  {{\"name\":\"{}\",\"form\":\"{}\",\"score\":{{\"semantic\":{},\"readability\":{},\"originality\":{},\"total\":{}}},\"provenance\":{{\"concepts\":[{}],\"transformations\":[{}]}}}}{}",
            candidate.name,
            candidate.form.as_str(),
            candidate.score.semantic,
            candidate.score.readability,
            candidate.score.originality,
            candidate.score.total,
            json_array(&candidate.provenance.concepts),
            json_array(&candidate.provenance.transformations),
            separator
        );
    }

    println!("]");
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

    service
        .generate(&options.criteria)
        .map_err(|error| error.to_string())
}

fn run() -> Result<(), String> {
    let arguments: Vec<String> = env::args().skip(1).collect();

    let Some(mut options) = parse_arguments(arguments)? else {
        println!("{}", usage());
        return Ok(());
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

    match options.format {
        OutputFormat::Text => render_text(&candidates),
        OutputFormat::Json => render_json(&candidates),
    }

    Ok(())
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            eprintln!("{}", usage());
            ExitCode::FAILURE
        }
    }
}
