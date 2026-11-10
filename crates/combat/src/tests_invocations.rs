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

/// Première forme du sort donné (les données changent : on ne code pas un identifiant).
fn forme_de(sort: &str) -> String {
    fiches::Referentiel::partage()
        .formes
        .iter()
        .find(|f| f.sort_en == sort)
        .unwrap()
        .id
        .clone()
}

#[test]
fn une_forme_applique_ses_effets_au_personnage() {
    let ctx = ctx_avec(11, 0);
    let mut c = combat(&ctx);
    let forme = forme_de("Beast Shape II");
    c.appliquer(
        ActionCombat::Forme {
            pj: 1,
            sort: "Beast Shape II".into(),
            forme: forme.clone(),
        },
        &ctx,
    )
    .unwrap();
    let e = c.etat();
    assert_eq!(
        e.aventure
            .effets
            .actifs
            .iter()
            .filter(|t| t.proprietaire == "1")
            .count(),
        1
    );
    assert_eq!(e.emplacements_depenses[&(1, 4)], 1);
    // une seule forme à la fois
    let deux = c.appliquer(
        ActionCombat::Forme {
            pj: 1,
            sort: "Beast Shape II".into(),
            forme,
        },
        &ctx,
    );
    assert!(deux.is_err());
}

#[test]
fn une_forme_doit_appartenir_au_sort() {
    let ctx = ctx_avec(11, 0);
    let mut c = combat(&ctx);
    let mauvaise = forme_de("Elemental Body I");
    let r = c.appliquer(
        ActionCombat::Forme {
            pj: 1,
            sort: "Beast Shape II".into(),
            forme: mauvaise,
        },
        &ctx,
    );
    assert!(r.is_err());
    assert!(c.etat().aventure.effets.actifs.is_empty());
}

#[test]
fn le_contexte_du_combat_connait_les_invoques_et_les_personnages() {
    let ctx = ctx_avec(11, 0);
    let mut c = combat(&ctx);
    c.appliquer(convoquer("Summon Nature's Ally 1", "Aigle", 1), &ctx)
        .unwrap();
    let e = c.etat();
    let etendu = e.avec_invocations(&ctx);
    // la créature a sa fiche ; le personnage garde la sienne
    assert_eq!(etendu(BASE_INVOCATIONS).unwrap().nom, "Aigle");
    assert_eq!(etendu(1).unwrap().nom, "Mialee");
    assert!(etendu(42).is_none());
}

/// Combat avec le pouvoir de forme animale de Mialee (pj 1) : 2 utilisations par jour.
fn combat_avec_pouvoir(ctx: &dyn Fn(IdPj) -> Option<aventure::ContextePj>) -> Combat {
    let mut e = EtatAventure::nouveau(Instant::depuis(1, 8, 0).unwrap(), false);
    for pj in [1, 2] {
        e.appliquer(&EvenementAventure::Entree { pj }, ctx).unwrap();
    }
    let mut ressources = std::collections::BTreeMap::new();
    ressources.insert(
        "forme_animale".to_string(),
        aventure::Ressource {
            unite: aventure::UniteRessource::Utilisations,
            max: 2,
            utilisees: 0,
            source: "pouvoir de druide".into(),
        },
    );
    e.appliquer(
        &EvenementAventure::RessourcesDefinies {
            pj: 1,
            ressources: ressources.into_iter().collect(),
        },
        ctx,
    )
    .unwrap();
    let mut c = Combat::nouveau(&e, vec![1, 2], ctx).unwrap();
    c.appliquer(ActionCombat::Initiative { pj: 1, d20: 10 }, ctx)
        .unwrap();
    c.appliquer(ActionCombat::Initiative { pj: 2, d20: 5 }, ctx)
        .unwrap();
    c.appliquer(ActionCombat::Demarrer { surprise: None }, ctx)
        .unwrap();
    c
}

/// Forme animale par son identifiant, avec sa taille (pour tester le bonus de taille).
fn forme_par_taille(taille: &str) -> fiches::formes::FormeAnimale {
    fiches::Referentiel::partage()
        .formes
        .iter()
        .find(|f| f.categorie == "animal" && f.taille.as_deref() == Some(taille))
        .unwrap()
        .clone()
}

#[test]
fn le_pouvoir_de_forme_consomme_une_utilisation_et_applique_la_forme() {
    let ctx = ctx_avec(11, 0);
    let mut c = combat_avec_pouvoir(&ctx);
    let f = forme_par_taille("M");
    c.appliquer(
        ActionCombat::FormeAnimale {
            pj: 1,
            forme: f.id.clone(),
            creature: creature_pour(&f),
        },
        &ctx,
    )
    .unwrap();
    let e = c.etat();
    assert_eq!(e.aventure.pjs[&1].ressources["forme_animale"].utilisees, 1);
    assert_eq!(
        e.aventure
            .effets
            .actifs
            .iter()
            .filter(|t| t.proprietaire == "1")
            .count(),
        1
    );
}

#[test]
fn la_taille_change_ca_et_attaque() {
    // Taille P : +1 à la CA et à l'attaque par rapport à M (référence du druide)
    let p = forme_par_taille("P");
    let effets = crate::invocations::effets_de_forme(&p, "M");
    assert!(effets.iter().any(|e| e.cible.0 == "ca"
        && e.type_bonus == fiches::effet::TypeBonus::Taille
        && e.valeur == 1));
    assert!(effets.iter().any(|e| e.cible.0 == "bba" && e.valeur == 1));
    // Taille M : aucun bonus de taille ajouté (seulement les effets du livre)
    let m = forme_par_taille("M");
    let em = crate::invocations::effets_de_forme(&m, "M");
    assert!(!em.iter().any(|e| e.cible.0 == "bba"));
}

#[test]
fn le_pouvoir_s_arrete_quand_les_utilisations_sont_epuisees() {
    let ctx = ctx_avec(11, 0);
    let mut c = combat_avec_pouvoir(&ctx);
    let f = forme_par_taille("M");
    c.appliquer(
        ActionCombat::FormeAnimale {
            pj: 1,
            forme: f.id.clone(),
            creature: creature_pour(&f),
        },
        &ctx,
    )
    .unwrap();
    // une seule forme à la fois : refusé même s'il reste des utilisations
    assert!(c
        .appliquer(
            ActionCombat::FormeAnimale {
                pj: 1,
                forme: f.id.clone(),
                creature: creature_pour(&f)
            },
            &ctx
        )
        .is_err());
}

#[test]
fn la_taille_vient_de_la_race_et_la_forme_s_y_ajoute() {
    // druide halfelin (taille P) prenant une forme M : mod M (0) − mod P (+1) = −1
    let forme = forme_par_taille("M");
    let e = crate::invocations::effets_de_forme(&forme, "P");
    assert!(e.iter().any(|x| x.cible.0 == "ca"
        && x.type_bonus == fiches::effet::TypeBonus::Taille
        && x.valeur == -1));
    assert!(e.iter().any(|x| x.cible.0 == "bba" && x.valeur == -1));
    // forme G prise par un druide M : mod G (−1) − mod M (0) = −1
    let g = forme_par_taille("G");
    let eg = crate::invocations::effets_de_forme(&g, "M");
    assert!(eg.iter().any(|x| x.cible.0 == "bba" && x.valeur == -1));
    // élémentaire sans taille dans les données : taille du sort (corps élémentaire I = P)
    let elem = fiches::Referentiel::partage()
        .formes
        .iter()
        .find(|f| f.sort_en == "Elemental Body I")
        .unwrap()
        .clone();
    assert_eq!(crate::invocations::taille_de_forme(&elem), "P");
}

#[test]
fn le_niveau_de_druide_donne_la_duree_et_les_formes() {
    let ctx = ctx_avec(11, 0);
    let mut c = combat_avec_pouvoir(&ctx);
    // druide de niveau 11 (fixture) : une forme de niveau 12 est refusée
    let trop_haut = fiches::Referentiel::partage()
        .formes
        .iter()
        .find(|f| f.niveau_druide == 12)
        .unwrap()
        .id
        .clone();
    assert!(c
        .appliquer(
            ActionCombat::FormeAnimale {
                pj: 1,
                forme: trop_haut,
                creature: "Aigle".into(),
            },
            &ctx
        )
        .is_err());
}

#[test]
fn la_forme_hors_combat_donne_une_utilisation_et_les_effets() {
    let ctx = ctx_avec(11, 0);
    let mut e = EtatAventure::nouveau(Instant::depuis(1, 8, 0).unwrap(), false);
    e.appliquer(&EvenementAventure::Entree { pj: 1 }, &ctx)
        .unwrap();
    let f = forme_par_taille("M");
    let evs = crate::hors_combat::evenements_forme_animale(&e, 1, &f.id, &creature_pour(&f), &ctx)
        .unwrap();
    assert_eq!(evs.len(), 2);
    assert!(
        matches!(&evs[0], EvenementAventure::RessourceUtilisee { pj: 1, nom, quantite: 1 } if nom == "forme_animale")
    );
    assert!(matches!(
        &evs[1],
        EvenementAventure::EffetAjoute { pj: 1, .. }
    ));
    // niveau de druide insuffisant : refusé
    let haute = fiches::Referentiel::partage()
        .formes
        .iter()
        .find(|f| f.niveau_druide == 12)
        .unwrap()
        .id
        .clone();
    assert!(crate::hors_combat::evenements_forme_animale(&e, 1, &haute, "Aigle", &ctx).is_err());
}

/// Première créature compatible avec la forme (les données changent : on ne code pas un nom).
fn creature_pour(f: &fiches::formes::FormeAnimale) -> String {
    crate::invocations::creatures_pour_forme(f)
        .first()
        .map(|c| c.nom.clone())
        .unwrap_or_default()
}

#[test]
fn la_creature_doit_etre_compatible_avec_la_forme() {
    let ctx = ctx_avec(11, 0);
    let mut c = combat_avec_pouvoir(&ctx);
    let f = forme_par_taille("M");
    // l'Aigle (P) n'est pas de taille M : refusé pour une forme M
    let refus = c.appliquer(
        ActionCombat::FormeAnimale {
            pj: 1,
            forme: f.id.clone(),
            creature: "Aigle".into(),
        },
        &ctx,
    );
    assert!(refus.is_err());
    let elem = fiches::Referentiel::partage()
        .formes
        .iter()
        .find(|x| x.sort_en == "Elemental Body I")
        .unwrap()
        .clone();
    assert!(crate::invocations::creatures_pour_forme(&elem)
        .iter()
        .all(|x| x.type_creature.contains("élémentaire")));
    assert!(crate::invocations::creatures_pour_forme(&f)
        .iter()
        .all(|x| x.type_creature.starts_with("Animal") && x.taille == "M"));
}
