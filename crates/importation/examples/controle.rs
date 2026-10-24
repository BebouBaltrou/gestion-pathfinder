//! Contrôle une fiche PDF et affiche le rapport :
//! `cargo run -p importation --example controle -- chemin\vers\fiche.pdf`

use importation::{controler, lire_champs, lire_feuille, Gravite};
use std::path::Path;

fn main() {
    let Some(chemin) = std::env::args().nth(1) else {
        eprintln!("usage : controle <fiche.pdf>");
        std::process::exit(2);
    };
    let champs = match lire_champs(Path::new(&chemin)) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(1);
        }
    };
    let feuille = lire_feuille(&champs);
    let rapport = controler(&feuille, fiches::Referentiel::partage());
    println!(
        "{} — {} ({} champs lus)\n",
        feuille.nom, feuille.classes_et_niveaux, feuille.champs_lus
    );
    for g in [Gravite::Ecart, Gravite::AValider, Gravite::Accord] {
        let titre = match g {
            Gravite::Ecart => "ÉCARTS (les règles donnent une autre valeur)",
            Gravite::AValider => "À VALIDER",
            Gravite::Accord => "D'ACCORD",
        };
        println!("== {titre}");
        for l in rapport.lignes.iter().filter(|l| l.gravite == g) {
            println!(
                "- [{}] {} : feuille {} / règles {}   ({})\n    {}",
                l.rubrique, l.libelle, l.feuille, l.regles, l.champ, l.explication
            );
        }
        println!();
    }
    if let Ok(p) = importation::proposer(&feuille, fiches::Referentiel::partage()) {
        if let Ok(v) = p.fiche.calculer(fiches::Referentiel::partage()) {
            println!(
                "Fiche proposée : niveau {}, PV {}, dons {:?}
",
                p.niveau,
                v.pv_max.total,
                p.fiche.dons_choisis()
            );
        }
    }
    println!("== NON VÉRIFIÉ");
    for n in &rapport.non_verifie {
        println!("- {n}");
    }
}
