# Actname

Actname is a deterministic, offline-first semantic naming engine written in Rust.

## Install

Needs Rust 1.85 or newer (`rustup`).

```text
cargo install --locked --git https://github.com/grosjeanbaptiste/Actname
export PATH="$HOME/.cargo/bin:$PATH"
actname generate --prefix act --domain architecture --limit 3
```

From a checkout: `cargo install --locked --path .`. The binary is
`~/.cargo/bin/actname`; update by re-running the first command with `--force`.

## API

### Command line

```text
actname generate --prefix <p>... [--domain "<words>"] [--readme <file>] [--max-length n] [--limit n]
                 [--strategy hybrid|composed|coined] [--exclude <name>]... [--format text|json]
actname --help
```

| Option | Required | Default | Meaning |
|---|---|---|---|
| `--prefix` | yes, repeatable | – | Every name starts with one of these. Lowercase ASCII letters only. |
| `--domain` | yes, unless the README gives it | the repository's README | Words describing the subject, any of French, English, Dutch, Spanish, German. Without it, the context is the content of `README.md` in the current directory, when it holds text. An explicit `--domain` wins over the README. |
| `--readme` | no | `README.md` | The file read as the context when `--domain` is absent. A file named here that does not exist, or any file that cannot be read, is an error; a missing default `README.md` is simply no context. |
| `--max-length` | no | `10` | Longest name, must exceed at least one prefix length. |
| `--limit` | no | `20` | Number of names returned, must be positive. |
| `--strategy` | no | `hybrid` | `composed`, `coined` or both (`hybrid`). |
| `--exclude` | no, repeatable | – | Names never returned. |
| `--format` | no | `text` | `text` (table) or `json`. |
| `--lexicon` | no | `resources/lexicon.tsv` | Lexicon file replacing the embedded one. |
| `--scoring` | no | built-in model | Scoring model file (weights, penalties). |
| `--semantics` | no | `resources/semantic.tsv` | Word-to-concept file replacing the embedded one. |

Names come back ranked best first, deterministically: same input, same output.

Inside a repository with a README, `--domain` can be left out: the README is the context.

```text
cd my-repo && actname generate --prefix act --limit 3
```


JSON output is an array, one object per name (`score` fields are integers 0–100):

```json
[{"name":"actarch","form":"coined",
  "score":{"semantic":52,"readability":93,"originality":89,"total":74},
  "provenance":{"concepts":["architecture"],"transformations":["root_truncation:4"]}}]
```

Exit code `0` on success. Exit code `1` with the reason on stderr (for example
`semantic context is required`) when an option is missing, unknown or invalid.

### Data files

Nothing the ranking depends on is fixed in the code; each file below replaces its embedded default.

- **Lexicon** (`--lexicon`): one `suffix<TAB>composed|coined<TAB>concept|concept` per line, `#` comments.
  Default: `resources/lexicon.tsv`.
- **Semantics** (`--semantics`): one `concept<TAB>alias|alias|…` per line, in any language; a word of
  `--domain` that matches an alias or concept selects that concept. Default: `resources/semantic.tsv`.
- **Scoring** (`--scoring`): `key=value` lines over the defaults, `#` comments. Keys, with defaults:
  `weight_semantic=45`, `weight_readability=30`, `weight_originality=25` (only the ratios matter),
  `originality_composed=70`, `originality_coined=90`, `coined_blend_form=55`,
  `coined_blend_quality=45`, `semantic_base=40`, `semantic_per_match=12`, `length_free=12`,
  `length_penalty=2`, `consonant_penalty=7`, `repetition_penalty=6`.

An unreadable or malformed file is an error naming the file and the line (exit code `1`).
The defaults are written once, in `ScoringModel::default()`.

```rust
use actname::lexicon::TextLexicon;
use actname::scoring::{ScoringModel, Weights};
use actname::semantics::SemanticKnowledgeBase;
use actname::{NamingCriteria, NamingService, Strategy};

let lexicon = TextLexicon::from_text("zork\tcomposed\tarchitecture\n").unwrap();
let knowledge = SemanticKnowledgeBase::from_tsv("architecture\tzorp\n").unwrap();
let model = ScoringModel { weights: Weights { semantic: 1, readability: 1, originality: 1 }, ..ScoringModel::default() };
let service = NamingService::new(&lexicon).with_knowledge(knowledge).with_model(model);
let criteria = NamingCriteria {
    prefixes: vec!["act".into()], context: "zorp".into(), max_length: 10,
    limit: 5, strategy: Strategy::Composed, exclusions: vec![],
};
assert!(service.generate(&criteria)?.iter().any(|c| c.name == "actzork"));
# Ok::<(), actname::NamingError>(())
```

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
