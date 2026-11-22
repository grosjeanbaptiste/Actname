//! A repository name tool must find names for every kind of utility, in English and in French.
use actname::{InMemoryLexicon, NamingCriteria, NamingService, Strategy};

/// category, an English context, a French context.
const CATEGORIES: &[(&str, &str, &str)] = &[
    (
        "terminal",
        "command line terminal",
        "ligne de commande terminal",
    ),
    (
        "build",
        "build system compiler",
        "compilation et assemblage",
    ),
    ("testing", "unit testing framework", "tests unitaires"),
    (
        "lint",
        "linter and code formatter",
        "analyse statique et formatage",
    ),
    ("debug", "debugger and profiler", "débogage et profilage"),
    (
        "deploy",
        "continuous integration deployment",
        "intégration continue et déploiement",
    ),
    ("container", "docker container runtime", "conteneurs"),
    ("cloud", "cloud infrastructure", "infrastructure cloud"),
    (
        "database",
        "database and sql queries",
        "base de données requêtes",
    ),
    ("web", "web server and api", "serveur web et api"),
    ("network", "network http client", "réseau client http"),
    (
        "security",
        "security and encryption",
        "sécurité et chiffrement",
    ),
    (
        "auth",
        "authentication and passwords",
        "authentification et mots de passe",
    ),
    ("logs", "logging and monitoring", "journaux et supervision"),
    (
        "analytics",
        "data analysis and charts",
        "analyse de données et graphiques",
    ),
    (
        "ai",
        "machine learning and neural networks",
        "apprentissage automatique et réseaux de neurones",
    ),
    (
        "search",
        "search engine and indexing",
        "moteur de recherche et indexation",
    ),
    (
        "parsing",
        "parser and serialization",
        "analyseur syntaxique et sérialisation",
    ),
    (
        "files",
        "file system and directories",
        "fichiers et répertoires",
    ),
    ("text", "text editor", "éditeur de texte"),
    (
        "docs",
        "documentation generator",
        "génération de documentation",
    ),
    (
        "packages",
        "package manager and dependencies",
        "gestionnaire de paquets et dépendances",
    ),
    (
        "config",
        "configuration and environment variables",
        "configuration et variables d'environnement",
    ),
    (
        "schedule",
        "task scheduler and cron jobs",
        "planificateur de tâches",
    ),
    (
        "messaging",
        "message queue and events",
        "file de messages et événements",
    ),
    ("mobile", "mobile app", "application mobile"),
    ("game", "game engine", "moteur de jeu"),
    ("media", "audio and video player", "lecteur audio et vidéo"),
    (
        "image",
        "image processing and graphics",
        "traitement d'images et graphisme",
    ),
    (
        "ui",
        "user interface components",
        "interface utilisateur et composants",
    ),
    (
        "finance",
        "payments and accounting",
        "paiements et comptabilité",
    ),
    (
        "email",
        "email and chat messaging",
        "courriel et messagerie",
    ),
    (
        "calendar",
        "calendar and time zones",
        "calendrier et fuseaux horaires",
    ),
    ("backup", "backup and storage", "sauvegarde et stockage"),
    (
        "benchmark",
        "benchmark and performance",
        "mesure de performances",
    ),
    (
        "scraping",
        "web scraping and crawler",
        "extraction de pages web et robot d'indexation",
    ),
    (
        "i18n",
        "translation and localization",
        "traduction et localisation",
    ),
    ("learning", "education and courses", "éducation et cours"),
    ("health", "health and fitness", "santé et forme"),
    (
        "embedded",
        "embedded iot sensors",
        "objets connectés et capteurs",
    ),
    ("maps", "maps and geolocation", "cartes et géolocalisation"),
    ("notes", "notes and productivity", "notes et productivité"),
    (
        "template",
        "project template scaffolding",
        "modèle de projet et génération de squelette",
    ),
    (
        "convert",
        "file converter and migration",
        "convertisseur et migration",
    ),
    (
        "plugin",
        "plugin and extension system",
        "système de plugins et extensions",
    ),
    (
        "workflow",
        "workflow automation",
        "automatisation de flux de travail",
    ),
    ("git", "git repositories", "dépôts git"),
    (
        "alert",
        "alerting and notifications",
        "alertes et notifications",
    ),
    ("a11y", "accessibility", "accessibilité"),
    (
        "blockchain",
        "blockchain and crypto wallet",
        "chaîne de blocs et portefeuille",
    ),
];

fn names(context: &str) -> Vec<String> {
    let criteria = NamingCriteria {
        prefixes: vec!["act".into()],
        context: context.into(),
        max_length: 10,
        limit: 20,
        strategy: Strategy::Hybrid,
        exclusions: vec![],
    };
    NamingService::new(&InMemoryLexicon)
        .generate(&criteria)
        .unwrap_or_else(|error| panic!("{context}: {error}"))
        .into_iter()
        .map(|candidate| candidate.name)
        .collect()
}

#[test]
fn every_category_gets_names_in_english_and_french() {
    let missing: Vec<String> = CATEGORIES
        .iter()
        .flat_map(|(category, en, fr)| [("en", category, en), ("fr", category, fr)])
        .filter(|(_, _, context)| names(context).is_empty())
        .map(|(lang, category, context)| format!("{category}/{lang}: {context}"))
        .collect();
    assert!(missing.is_empty(), "no name for:\n{}", missing.join("\n"));
}
