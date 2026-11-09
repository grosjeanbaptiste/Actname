# Actname

Actname is a deterministic, offline-first semantic naming engine written in Rust.

## API

### Command line

```text
actname generate --prefix <p>... --domain "<words>" [--max-length n] [--limit n]
                 [--strategy hybrid|composed|coined] [--exclude <name>]... [--format text|json]
actname --help
```

| Option | Required | Default | Meaning |
|---|---|---|---|
| `--prefix` | yes, repeatable | – | Every name starts with one of these. Lowercase ASCII letters only. |
| `--domain` | yes | – | Words describing the subject, any of French, English, Dutch, Spanish, German. |
| `--max-length` | no | `10` | Longest name, must exceed at least one prefix length. |
| `--limit` | no | `20` | Number of names returned, must be positive. |
| `--strategy` | no | `hybrid` | `composed`, `coined` or both (`hybrid`). |
| `--exclude` | no, repeatable | – | Names never returned. |
| `--format` | no | `text` | `text` (table) or `json`. |

Names come back ranked best first, deterministically: same input, same output.

JSON output is an array, one object per name (`score` fields are integers 0–100):

```json
[{"name":"actarch","form":"coined",
  "score":{"semantic":52,"readability":93,"originality":89,"total":74},
  "provenance":{"concepts":["architecture"],"transformations":["root_truncation:4"]}}]
```

Exit code `0` on success. Exit code `1` with the reason on stderr (for example
`semantic context is required`) when an option is missing, unknown or invalid.

### Library

```rust
use actname::{InMemoryLexicon, NamingCriteria, NamingService, Strategy};

let criteria = NamingCriteria {
    prefixes: vec!["act".into()],
    context: "architecture".into(),
    max_length: 10,
    limit: 5,
    strategy: Strategy::Hybrid,
    exclusions: vec![],
};
let names = NamingService::new(&InMemoryLexicon).generate(&criteria)?; // Vec<Candidate>
assert!(names.iter().all(|c| c.name.starts_with("act")));
# Ok::<(), actname::NamingError>(())
```

`Candidate` carries `name`, `form`, `score` and `provenance`. Invalid criteria
return `NamingError`. The `Lexicon` trait lets a caller supply its own vocabulary.

### Not in scope

Whether a name is free on GitHub, as a domain or as a trademark is not decided
here. A caller that needs it checks the candidates itself.

## Features

- Required prefixes
- Semantic context filtering
- Maximum name length
- Composed names and coined names
- Excluded names
- Deterministic ranking
- Explainable scoring
- JSON and text output
- No external runtime dependencies

## Requirements

Rust 1.85 or newer.

## Build

```bash
cargo build --release
```

## Tests

```bash
cargo test --all-targets
```

## Generate names

```bash
cargo run -- generate \
  --prefix act \
  --prefix acte \
  --domain "architecture bâtiments plans conception" \
  --max-length 10 \
  --limit 20 \
  --strategy hybrid
```

## JSON

```bash
cargo run -- generate \
  --prefix act \
  --domain "architecture" \
  --format json
```

## Strategies

- `hybrid`: composed and coined names
- `composed`: lexical combinations
- `coined`: constructed neologisms

## Architecture

- `src/domain.rs`: domain models, constraints, semantic scoring
- `src/application.rs`: naming use case
- `src/infrastructure.rs`: built-in semantic lexicon
- `src/main.rs`: command-line interface
- `tests/`: acceptance and integration tests

## Scoring

The ranking uses three weighted heuristic criteria:

- Semantic relevance: 45%
- Phonotactic readability: 30%
- Morphological originality: 25%

These scores are deterministic estimates, not validated linguistic measurements.
Readability considers difficult consonant clusters and repeated letters; valid
names are not penalized merely for being longer than seven characters.
Coined-name originality is adjusted using the transformations used to form it,
so meaningful recompositions can score above simple truncations.

## Limitations

The initial implementation uses a finite semantic lexicon.

It does not perform embedding similarity, trademark clearance, domain availability checks, or GitHub name availability checks.

The coined-name classification describes the generation strategy and does not certify that a name is novel or unused.

## Multilingual semantic knowledge base

Actname supports deterministic lexical expansion across French,
English, Dutch, Spanish and German.

The embedded vocabulary is stored in `resources/semantic.tsv`.

Each row contains a canonical concept and pipe-separated aliases.

The semantic engine supports Unicode accent normalization,
exact token matching, multilingual lexical equivalence and
deterministic semantic expansion.

Unknown concepts do not receive inferred semantic relationships.

Future extensions may introduce external dictionaries,
morphological analysis and contextual semantic similarity.

## Morphological generation

Actname provides a deterministic morphological generation API.

The module `actname::morphology` supports:

- Arbitrary lexical roots
- Prefix-based composition
- Boundary-character deduplication
- Prefix and suffix removal
- Root truncation
- Morphological contractions
- Linking vowels and root recomposition
- Candidate provenance
- Duplicate elimination
- Maximum-length constraints
- Exclusion lists
- Composed, coined and hybrid strategies

The morphological generator is independent from the existing
semantic ranking pipeline.

The next iteration will integrate generated candidates into
semantic scoring and the CLI.

The implementation is rule-based and does not claim linguistic
validity or trademark availability.

## Integrated semantic and morphological ranking

The naming application combines lexical and morphological candidates.

Morphological roots are selected from recognized semantic concepts.
Candidates without semantic evidence are rejected.

All candidates share the same semantic, readability, originality
and total scoring model.

The integrated pipeline preserves:

- Deterministic output
- Unique candidate names
- Semantic relevance constraints
- Maximum-length constraints
- Generation strategies
- Exclusion lists
- Result limits
- Existing CLI text and JSON formats

Morphological candidates are integrated before global ranking,
deduplication and result truncation.
Each returned candidate exposes its source concepts and transformations through
`Candidate::provenance`; JSON output includes the same information under
`provenance.concepts` and `provenance.transformations`.

Coined names are intentionally allowed to be artificial. For example,
`actetecture` can be traced to `architecture` through prefix removal and the
addition of a linking vowel, and is evaluated for sound and transformation
quality rather than penalized for not being a conventional word.
`acteble` links the `capacite` concept to the coined suffix `ble`, while
`actrack` records the shared-boundary fusion of `act` and `track`.
