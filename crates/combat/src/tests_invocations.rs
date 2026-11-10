//! Convocation d'alliés naturels en combat : créatures invoquées, initiative, annulation, validation.

use crate::invocations::{est_invocation, BASE_INVOCATIONS};
use crate::tests_sorts::ctx_avec;
use crate::*;
use aventure::{EtatAventure, EvenementAventure, IdPj};
use temps::Instant;

/// Combat commencé : Mialee (pj 1, joue en premier) et Valéros (pj 2).
fn combat(ctx: &dyn Fn(IdPj) -> Option<aventure::ContextePj>) -> Combat {
    let mut e = EtatAventure::nouveau(Instant::depuis(1, 8, 0).unwrap(), false);
    for pj in [1, 2] {
        e.appliquer(&EvenementAventure::Entree { pj }, ctx).unwrap();
    }
    let mut c = Combat::nouveau(&e, vec![1, 2], ctx).unwrap();
    c.appliquer(ActionCombat::Initiative { pj: 1, d20: 10 }, ctx)
        .unwrap();
    c.appliquer(ActionCombat::Initiative { pj: 2, d20: 5 }, ctx)
        .unwrap();
    c.appliquer(ActionCombat::Demarrer { surprise: None }, ctx)
        .unwrap();
    c
}

fn convoquer(sort: &str, creature: &str, nombre: u32) -> ActionCombat {
    ActionCombat::Convoquer {
        lanceur: 1,
        sort: sort.into(),
        creature: creature.into(),
        nombre,
        d20: 15,
    }
}

#[test]
fn une_convocation_ajoute_une_creature_jouable() {
    let ctx = ctx_avec(11, 0);
    let mut c = combat(&ctx);
    c.appliquer(convoquer("Summon Nature's Ally 1", "Aigle", 1), &ctx)
        .unwrap();
    let e = c.etat();
    let id = BASE_INVOCATIONS;
    assert!(e.engages.contains(&id));
    assert!(e.ordre.contains(&id));
    let inv = &e.invocations[&id];
    assert_eq!(inv.creature.nom, "Aigle");
    assert_eq!(inv.lanceur, 1);
    // initiative : d20 15 + Init +2 de l'aigle
    assert_eq!(e.initiatives[&id].total, 17);
    // fiche de la créature : PV, CA, initiative lus par le mode aventure
    let ctx_inv = e.avec_invocations(&ctx);
    let c_inv = ctx_inv(id).unwrap();
    assert_eq!(c_inv.nom, "Aigle");
    assert_eq!(c_inv.pv_max, 5);
    assert_eq!(c_inv.initiative, 2);
}

#[test]
fn le_groupe_partage_un_seul_d20() {
    let ctx = ctx_avec(11, 0);
    let mut c = combat(&ctx);
    // Summon Nature's Ally 2 : 1d3 créatures de niveau 1 (Aigle)
    c.appliquer(convoquer("Summon Nature's Ally 2", "Aigle", 3), &ctx)
        .unwrap();
    let e = c.etat();
    let ids: Vec<IdPj> = e.invocations.keys().copied().collect();
    assert_eq!(ids.len(), 3);
    assert!(ids.iter().all(|i| e.initiatives[i].d20 == 15));
}

#[test]
fn nombre_et_niveau_sont_controles() {
    let ctx = ctx_avec(11, 0);
    let mut c = combat(&ctx);
    // SNA 1 : une seule créature de niveau 1
    assert!(c
        .appliquer(convoquer("Summon Nature's Ally 1", "Aigle", 2), &ctx)
        .is_err());
    // créature de niveau 3 hors de portée d'un SNA 1
    let niveau3 = referentiel_niveau(3);
    assert!(c
        .appliquer(convoquer("Summon Nature's Ally 1", &niveau3, 1), &ctx)
        .is_err());
    // nom inconnu
    assert!(c
        .appliquer(
            convoquer("Summon Nature's Ally 1", "Dragon inventé", 1),
            &ctx
        )
        .is_err());
    // pas une convocation
    assert!(c
        .appliquer(convoquer("Magic Missile", "Aigle", 1), &ctx)
        .is_err());
    // rien n'a changé
    assert!(c.etat().invocations.is_empty());
}

/// Premier nom de créature complète du niveau demandé (les données changent : on ne code pas un nom).
fn referentiel_niveau(niveau: u32) -> String {
    fiches::Referentiel::partage()
        .convocations
        .creatures
        .iter()
        .find(|c| c.niveau == niveau && c.complete)
        .unwrap()
        .nom
        .clone()
}

#[test]
fn annuler_retire_la_creature() {
    let ctx = ctx_avec(11, 0);
    let mut c = combat(&ctx);
    c.appliquer(convoquer("Summon Nature's Ally 1", "Aigle", 1), &ctx)
        .unwrap();
    assert_eq!(c.etat().invocations.len(), 1);
    assert!(c.annuler(&ctx).unwrap());
    assert!(c.etat().invocations.is_empty());
    assert!(!c.etat().engages.iter().any(|p| est_invocation(*p)));
}

#[test]
fn la_validation_ne_recopie_pas_les_creatures() {
    let ctx = ctx_avec(11, 0);
    let mut c = combat(&ctx);
    c.appliquer(convoquer("Summon Nature's Ally 1", "Aigle", 1), &ctx)
        .unwrap();
    let r = c.valider(&Validation::Tout, &[], &ctx).unwrap();
    assert!(!r.aventure.pjs.keys().any(|p| est_invocation(*p)));
    assert!(!r
        .aventure
        .effets
        .actifs
        .iter()
        .any(|t| t.proprietaire.parse::<u32>().is_ok_and(est_invocation)));
    assert!(r.sortie.fiche.iter().all(|(p, _)| !est_invocation(*p)));
    // le personnage lanceur reste, lui
    assert!(r.aventure.pjs.contains_key(&1));
}
