//! Affiche le contenu du référentiel officiel et ses avertissements.
//! `cargo run -p regles --example verifier_referentiel`

use regles::controles::controler;

fn main() {
    let r = regles::referentiel_officiel();
    println!("Référentiel officiel");
    println!(
        "  salles : {}  extensions : {}  équipes : {}  gestionnaires : {}",
        r.salles.len(),
        r.extensions.len(),
        r.equipes.len(),
        r.gestionnaires.len()
    );
    println!(
        "  modèles de bâtiments : {}  modèles d'organisations : {}",
        r.modeles_batiments.len(),
        r.modeles_organisations.len()
    );
    println!(
        "\nExemple de fiche : {}",
        r.reference(&r.salle("bar").unwrap().source)
    );
    println!("\nAvertissements :");
    for a in controler(&r) {
        println!("  - {a}");
    }
}
