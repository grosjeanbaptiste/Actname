use std::env;
use std::process::ExitCode;

use actname::{Candidate, InMemoryLexicon, NamingCriteria, NamingService, Strategy};

#[derive(Clone, Copy)]
enum OutputFormat {
    Text,
    Json,
}

struct CliOptions {
    criteria: NamingCriteria,
    format: OutputFormat,
}

fn usage() -> &'static str {
    "actname generate --prefix act --prefix acte --domain \"architecture plans\" [--max-length 10] [--limit 20] [--strategy hybrid|composed|coined] [--exclude NAME] [--format text|json]"
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
        format,
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
            "  {{\"name\":\"{}\",\"form\":\"{}\",\"score\":{{\"semantic\":{},\"readability\":{},\"originality\":{},\"total\":{}}}}}{}",
            candidate.name,
            candidate.form.as_str(),
            candidate.score.semantic,
            candidate.score.readability,
            candidate.score.originality,
            candidate.score.total,
            separator
        );
    }

    println!("]");
}

fn run() -> Result<(), String> {
    let arguments: Vec<String> = env::args().skip(1).collect();

    let Some(options) = parse_arguments(arguments)? else {
        println!("{}", usage());
        return Ok(());
    };

    let lexicon = InMemoryLexicon;
    let service = NamingService::new(&lexicon);

    let candidates = service
        .generate(&options.criteria)
        .map_err(|error| error.to_string())?;

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
