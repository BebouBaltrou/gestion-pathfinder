//! Économie d'actions, santé en combat, jets de stabilisation, soins par round.

use crate::*;
use aventure::{ContextePj, EtatAventure, EvenementAventure, IdPj};
use fiches::journal::EvenementFiche;
use temps::{Duree, Instant};

fn ctx(id: IdPj) -> Option<ContextePj> {
    let (nom, init) = match id {
        1 => ("Mialee", 2),
        2 => ("Valéros", 1),
        _ => return None,
    };
    Some(ContextePj {
        nom: nom.into(),
        niveau: 5,
        pv_max: 31,
        con_score: 12,
        con_mod: 1,
        vigueur: 4,
        initiative: init,
        incantation: None,
        dommages: vec![],
    })
}

/// Mialee (12) joue avant Valéros (6).
fn combat() -> Combat {
    let mut e = EtatAventure::nouveau(Instant::depuis(1, 8, 0).unwrap(), false);
    for pj in [1, 2] {
        e.appliquer(&EvenementAventure::Entree { pj }, &ctx)
            .unwrap();
    }
    let mut c = Combat::nouveau(&e, vec![1, 2], &ctx).unwrap();
    c.appliquer(ActionCombat::Initiative { pj: 1, d20: 10 }, &ctx)
        .unwrap();
    c.appliquer(ActionCombat::Initiative { pj: 2, d20: 5 }, &ctx)
        .unwrap();
    c.appliquer(ActionCombat::Demarrer { surprise: None }, &ctx)
        .unwrap();
    c
}

fn agir(pj: IdPj, t: TypeAction, libelle: &str, deplacement: bool) -> ActionCombat {
    ActionCombat::Agir {
        pj,
        type_action: t,
        libelle: libelle.into(),
        deplacement,
    }
}

fn degats(pj: IdPj, nombre: i32) -> ActionCombat {
    ActionCombat::Aventure(EvenementAventure::Degats {
        pj,
        nombre,
        letaux: true,
    })
}

#[test]
fn economie_du_tour_et_remise_a_zero() {
    let mut c = combat();
    c.appliquer(agir(1, TypeAction::Simple, "Attaque", false), &ctx)
        .unwrap();
    c.appliquer(agir(1, TypeAction::Mouvement, "Se déplacer", true), &ctx)
        .unwrap();
    let e = c.appliquer(agir(1, TypeAction::Simple, "Autre attaque", false), &ctx);
    assert!(e.is_err());
    assert!(c
        .etat()
        .journal
        .iter()
        .any(|j| j.texte.contains("Attaque (action simple)")));
    // ce n'est pas le tour de Valéros
    assert!(c
        .appliquer(agir(2, TypeAction::Simple, "Attaque", false), &ctx)
        .is_err());
    // tour suivant : Valéros joue, puis nouveau round : Mialee retrouve ses actions
    c.appliquer(ActionCombat::TourSuivant, &ctx).unwrap();
    c.appliquer(
        agir(2, TypeAction::Complexe, "Attaque à outrance", false),
        &ctx,
    )
    .unwrap();
    c.appliquer(ActionCombat::TourSuivant, &ctx).unwrap();
    assert_eq!(c.etat().actif, Some(1));
    c.appliquer(agir(1, TypeAction::Complexe, "Charge", true), &ctx)
        .unwrap();
}

#[test]
fn annuler_une_action_libere_l_economie() {
    let mut c = combat();
    c.appliquer(agir(1, TypeAction::Simple, "Attaque", false), &ctx)
        .unwrap();
    c.annuler(&ctx).unwrap();
    c.appliquer(
        agir(1, TypeAction::Complexe, "Attaque à outrance", false),
        &ctx,
    )
    .unwrap();
}

#[test]
fn action_immediate_hors_tour_prend_l_action_rapide_suivante() {
    let mut c = combat();
    c.appliquer(ActionCombat::TourSuivant, &ctx).unwrap(); // tour de Valéros
    c.appliquer(
        agir(1, TypeAction::Immediate, "Chute de plume", false),
        &ctx,
    )
    .unwrap();
    assert!(c
        .appliquer(agir(1, TypeAction::Immediate, "Encore", false), &ctx)
        .is_err());
    c.appliquer(ActionCombat::TourSuivant, &ctx).unwrap(); // retour à Mialee
    assert!(c
        .appliquer(agir(1, TypeAction::Rapide, "Sort accéléré", false), &ctx)
        .is_err());
    c.appliquer(agir(1, TypeAction::Simple, "Attaque", false), &ctx)
        .unwrap();
}

#[test]
fn round_de_surprise_une_seule_action() {
    let mut e = EtatAventure::nouveau(Instant::depuis(1, 8, 0).unwrap(), false);
    e.appliquer(&EvenementAventure::Entree { pj: 1 }, &ctx)
        .unwrap();
    let mut c = Combat::nouveau(&e, vec![1], &ctx).unwrap();
    c.appliquer(ActionCombat::Initiative { pj: 1, d20: 10 }, &ctx)
        .unwrap();
    c.appliquer(
        ActionCombat::Demarrer {
            surprise: Some(vec![1]),
        },
        &ctx,
    )
    .unwrap();
    c.appliquer(agir(1, TypeAction::Mouvement, "Se déplacer", true), &ctx)
        .unwrap();
    assert!(c
        .appliquer(agir(1, TypeAction::Simple, "Attaque", false), &ctx)
        .is_err());
    assert!(c
        .appliquer(agir(1, TypeAction::Complexe, "Charge", true), &ctx)
        .is_err());
    c.appliquer(agir(1, TypeAction::Libre, "Parler", false), &ctx)
        .unwrap();
}

#[test]
fn hors_de_combat_une_action_et_un_degat_d_effort() {
    let mut c = combat();
    c.appliquer(degats(1, 31), &ctx).unwrap(); // 0 PV
    assert_eq!(c.etat().aventure.pjs[&1].vital.degats, 31);
    c.appliquer(agir(1, TypeAction::Simple, "Attaque", false), &ctx)
        .unwrap();
    assert_eq!(c.etat().aventure.pjs[&1].vital.degats, 32); // -1 : mourant
    assert!(c.etat().aventure.pjs[&1].etats_fiche.contains("dying"));
}

#[test]
fn hors_de_combat_se_deplacer_ne_coute_rien() {
    let mut c = combat();
    c.appliquer(degats(1, 31), &ctx).unwrap();
    c.appliquer(agir(1, TypeAction::Mouvement, "Se déplacer", true), &ctx)
        .unwrap();
    assert_eq!(c.etat().aventure.pjs[&1].vital.degats, 31);
    assert!(c
        .appliquer(agir(1, TypeAction::Simple, "Attaque", false), &ctx)
        .is_err()); // une seule action
}

#[test]
fn mourant_ne_peut_pas_agir_et_doit_se_stabiliser() {
    let mut c = combat();
    c.appliquer(degats(1, 35), &ctx).unwrap(); // -4
    assert!(c
        .appliquer(agir(1, TypeAction::Simple, "Attaque", false), &ctx)
        .is_err());
    assert_eq!(
        c.etat().demandes(&ctx),
        vec![Demande::Stabilisation { pj: 1, malus: -4 }]
    );
    c.appliquer(
        ActionCombat::Aventure(EvenementAventure::Stabilisation { pj: 1, d20: 18 }),
        &ctx,
    )
    .unwrap();
    assert!(c.etat().demandes(&ctx).is_empty());
    assert!(c.etat().aventure.pjs[&1].etats_fiche.contains("stable"));
    // l'état de la fiche suit : « dying » retiré, « stable » ajouté
    let fiche = &c.etat().fiche;
    assert!(fiche.contains(&(1, EvenementFiche::EtatAjoute("stable".into()))));
    assert!(fiche.contains(&(1, EvenementFiche::EtatRetire("dying".into()))));
}

#[test]
fn guerison_acceleree_a_chaque_debut_de_tour() {
    let mut c = combat();
    c.appliquer(degats(1, 10), &ctx).unwrap();
    c.appliquer(
        ActionCombat::Aventure(EvenementAventure::EffetAjoute {
            pj: 1,
            nom: "Guérison accélérée".into(),
            source: "capacité".into(),
            duree: Duree::Heures(1),
            effets: vec![],
            etats: vec![],
            soin_par_round: 4,
            protections: vec![],
        }),
        &ctx,
    )
    .unwrap();
    c.appliquer(ActionCombat::TourSuivant, &ctx).unwrap(); // Valéros
    c.appliquer(ActionCombat::TourSuivant, &ctx).unwrap(); // nouveau round : début du tour de Mialee
    assert_eq!(c.etat().aventure.pjs[&1].vital.degats, 6);
    c.annuler(&ctx).unwrap();
    assert_eq!(c.etat().aventure.pjs[&1].vital.degats, 10);
}

#[test]
fn etat_prejudiciable_applique_pendant_le_combat() {
    let mut c = combat();
    c.appliquer(
        ActionCombat::Aventure(EvenementAventure::EffetAjoute {
            pj: 2,
            nom: "Terreur".into(),
            source: "sort".into(),
            duree: Duree::Rounds(2),
            effets: vec![],
            etats: vec!["shaken".into()],
            soin_par_round: 0,
            protections: vec![],
        }),
        &ctx,
    )
    .unwrap();
    assert!(c
        .etat()
        .fiche
        .contains(&(2, EvenementFiche::EtatAjoute("shaken".into()))));
    for _ in 0..4 {
        c.appliquer(ActionCombat::TourSuivant, &ctx).unwrap(); // deux rounds complets
    }
    assert!(c
        .etat()
        .fiche
        .contains(&(2, EvenementFiche::EtatRetire("shaken".into()))));
}
