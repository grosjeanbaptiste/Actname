use crate::domain::{Form, Lexeme, Lexicon};

#[derive(Default)]
pub struct InMemoryLexicon;

impl Lexicon for InMemoryLexicon {
    fn entries(&self) -> &[Lexeme] {
        ENTRIES
    }
}

static ENTRIES: &[Lexeme] = &[
    Lexeme {
        suffix: "ecture",
        concepts: &[
            "architecture",
            "batiments",
            "batiment",
            "plans",
            "plan",
            "conception",
            "structures",
        ],
        form: Form::Coined,
    },
    Lexeme {
        suffix: "form",
        concepts: &[
            "formes",
            "forme",
            "conception",
            "objets",
            "modelisation",
            "geometrie",
            "design",
        ],
        form: Form::Composed,
    },
    Lexeme {
        suffix: "plan",
        concepts: &[
            "plans",
            "plan",
            "dessin",
            "architecture",
            "batiments",
            "construction",
        ],
        form: Form::Composed,
    },
    Lexeme {
        suffix: "cad",
        concepts: &[
            "cao",
            "cad",
            "conception",
            "technique",
            "dessin",
            "plans",
            "modelisation",
        ],
        form: Form::Composed,
    },
    Lexeme {
        suffix: "morph",
        concepts: &[
            "formes",
            "forme",
            "morphologie",
            "geometrie",
            "modelisation",
            "transformation",
        ],
        form: Form::Coined,
    },
    Lexeme {
        suffix: "geom",
        concepts: &[
            "geometrie",
            "geometrique",
            "formes",
            "modelisation",
            "plans",
            "spatial",
        ],
        form: Form::Composed,
    },
    Lexeme {
        suffix: "struct",
        concepts: &[
            "structures",
            "structure",
            "batiments",
            "architecture",
            "ingenierie",
            "construction",
        ],
        form: Form::Composed,
    },
    Lexeme {
        suffix: "model",
        concepts: &[
            "modele",
            "modeles",
            "modelisation",
            "simulation",
            "conception",
            "objets",
        ],
        form: Form::Composed,
    },
    Lexeme {
        suffix: "draft",
        concepts: &[
            "plans",
            "plan",
            "dessin",
            "technique",
            "architecture",
            "conception",
        ],
        form: Form::Composed,
    },
    Lexeme {
        suffix: "forge",
        concepts: &[
            "fabrication",
            "prototypes",
            "prototype",
            "conception",
            "ingenierie",
            "objets",
        ],
        form: Form::Coined,
    },
    Lexeme {
        suffix: "mesh",
        concepts: &[
            "maillage",
            "geometrie",
            "modelisation",
            "simulation",
            "objets",
            "physiques",
        ],
        form: Form::Composed,
    },
    Lexeme {
        suffix: "frame",
        concepts: &[
            "structure",
            "structures",
            "cadre",
            "architecture",
            "batiments",
            "construction",
        ],
        form: Form::Composed,
    },
    Lexeme {
        suffix: "craft",
        concepts: &[
            "creation",
            "fabrication",
            "conception",
            "prototypes",
            "objets",
            "design",
        ],
        form: Form::Coined,
    },
    Lexeme {
        suffix: "graph",
        concepts: &[
            "graphes",
            "graphe",
            "modelisation",
            "structures",
            "relations",
            "plans",
        ],
        form: Form::Composed,
    },
    Lexeme {
        suffix: "circuit",
        concepts: &[
            "circuits",
            "circuit",
            "electronique",
            "electrique",
            "technologiques",
            "hardware",
        ],
        form: Form::Composed,
    },
    Lexeme {
        suffix: "shape",
        concepts: &[
            "forme",
            "formes",
            "objets",
            "geometrie",
            "design",
            "conception",
        ],
        form: Form::Composed,
    },
    Lexeme {
        suffix: "build",
        concepts: &[
            "construction",
            "batiments",
            "fabrication",
            "assemblage",
            "structures",
            "objets",
        ],
        form: Form::Composed,
    },
    Lexeme {
        suffix: "tech",
        concepts: &[
            "technologie",
            "technologies",
            "technologiques",
            "objets",
            "electronique",
            "ingenierie",
        ],
        form: Form::Composed,
    },
    Lexeme {
        suffix: "kinet",
        concepts: &[
            "mecanique",
            "mouvement",
            "physique",
            "physiques",
            "simulation",
            "machines",
        ],
        form: Form::Coined,
    },
    Lexeme {
        suffix: "logic",
        concepts: &[
            "logique",
            "algorithmes",
            "informatique",
            "programmation",
            "logiciel",
            "raisonnement",
        ],
        form: Form::Composed,
    },
    Lexeme {
        suffix: "lex",
        concepts: &[
            "droit",
            "juridique",
            "juridiques",
            "loi",
            "lois",
            "legislation",
            "articles",
        ],
        form: Form::Composed,
    },
    Lexeme {
        suffix: "juris",
        concepts: &[
            "droit",
            "juridique",
            "juridiques",
            "justice",
            "jurisprudence",
            "legislation",
        ],
        form: Form::Composed,
    },
    Lexeme {
        suffix: "score",
        concepts: &[
            "sport",
            "sports",
            "scores",
            "resultats",
            "competition",
            "pronostics",
        ],
        form: Form::Composed,
    },
    Lexeme {
        suffix: "track",
        concepts: &[
            "sport",
            "sports",
            "suivi",
            "performance",
            "performances",
            "activites",
            "donnees",
        ],
        form: Form::Composed,
    },
    Lexeme {
        suffix: "ble",
        concepts: &["capacite"],
        form: Form::Coined,
    },
    Lexeme {
        suffix: "predict",
        concepts: &[
            "prediction",
            "predictions",
            "pronostics",
            "statistiques",
            "prevision",
            "previsions",
        ],
        form: Form::Composed,
    },
];
