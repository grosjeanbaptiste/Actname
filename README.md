# Actname

Actname is a deterministic, offline-first semantic naming engine written in Rust.

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
- Readability: 30%
- Originality: 25%

These scores are deterministic estimates, not validated linguistic measurements.

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
