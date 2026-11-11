//! Names for the repositories that manage machines, volumes, servers and their configuration.
use actname::{InMemoryLexicon, NamingCriteria, NamingService, Strategy};

fn generate(context: &str) -> Vec<String> {
    let lexicon = InMemoryLexicon;
    let criteria = NamingCriteria {
        prefixes: vec!["act".into()],
        context: context.into(),
        max_length: 10,
        limit: 100,
        strategy: Strategy::Hybrid,
        exclusions: vec![],
    };
    NamingService::new(&lexicon)
        .generate(&criteria)
        .unwrap_or_else(|error| panic!("{error}"))
        .into_iter()
        .map(|candidate| candidate.name)
        .collect()
}

fn assert_names(context: &str, expected: &str) {
    let results = generate(context);
    assert!(
        results.contains(&expected.to_string()),
        "{context:?} should give {expected:?}, got {results:?}"
    );
}

#[test]
fn english_configuration_gives_conf() {
    assert_names("configuration settings", "actconf");
}

#[test]
fn french_settings_give_conf() {
    assert_names("reglages parametres", "actconf");
}

#[test]
fn german_settings_give_conf() {
    assert_names("einstellungen", "actconf");
}

#[test]
fn servers_give_host() {
    assert_names("server", "acthost");
}

#[test]
fn spanish_servers_give_host() {
    assert_names("servidores", "acthost");
}

#[test]
fn volumes_and_disks_give_vol() {
    assert_names("storage volumes disks", "actvol");
}

#[test]
fn fleet_of_machines_gives_fleet() {
    assert_names("fleet of machines", "actfleet");
}

#[test]
fn infrastructure_gives_infra() {
    assert_names("infrastructure inventory", "actinfra");
}

#[test]
fn provisioning_gives_ops() {
    assert_names("provisioning deployment", "actops");
}

#[test]
fn secrets_give_keep() {
    assert_names("secrets credentials", "actkeep");
}

#[test]
fn existing_architecture_names_do_not_move() {
    assert_names("building blueprints", "actecture");
}
