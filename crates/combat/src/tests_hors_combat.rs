//! Sorts et objets hors combat : emplacements et charges, effets qui restent pour le combat.

use crate::hors_combat::{lancer_hors_combat, utiliser_objet_hors_combat};
use crate::tests_sorts::ctx_avec;
use crate::*;
use aventure::{
    Charges, Consommable, ContextePj, EtatAventure, EvenementAventure, GenreObjet, IdPj,
};
use fiches::journal::EvenementFiche;
use temps::{Duree, Instant};

type Ctx = dyn Fn(IdPj) -> Option<ContextePj>;

/// Aventure où Mialee (1) et Valéros (2) sont entrés.
fn aventure(ctx: &Ctx) -> EtatAventure {
    let mut e = EtatAventure::nouveau(Instant::depuis(1, 8, 0).unwrap(), false);
    for pj in [1, 2] {
        e.appliquer(&EvenementAventure::Entree { pj }, ctx).unwrap();
    }
    e
}

/// Applique les événements d'un plan à l'aventure, comme le fait la campagne.
fn appliquer(e: &mut EtatAventure, evenements: &[EvenementAventure], ctx: &Ctx) {
    for ev in evenements {
        e.appliquer(ev, ctx).unwrap();
    }
}

fn objet(nom: &str, genre: GenreObjet, sort: &str, niveau_du_sort: u32, cl: u32) -> Consommable {
    Consommable {
        nom: nom.into(),
        genre,
        sort: Some(sort.into()),
        niveau_du_sort,
        niveau_de_lanceur: cl,
        arcanique: true,
        quantite: 1,
        charges: None,
        par_jour: None,
        notes: String::new(),
    }
}

/// Ajoute un objet à Mialee et renvoie son identifiant.
fn ajouter(e: &mut EtatAventure, o: Consommable, ctx: &Ctx) -> u64 {
    e.appliquer(&EvenementAventure::ObjetAjoute { pj: 1, objet: o }, ctx)
        .unwrap();
    *e.pjs[&1].inventaire.keys().max().unwrap()
}

fn lancer(sort: &str) -> ParamsLancer {
    ParamsLancer {
        pj: 1,
        sort: sort.into(),
        ..Default::default()
    }
}

#[test]
fn sort_hors_combat_depense_l_emplacement_et_ajoute_l_effet() {
    let ctx = ctx_avec(5, 0);
    let mut e = aventure(&ctx);
    let plan = lancer_hors_combat(
        &ParamsLancer {
            cibles: vec![1],
            ..lancer("Mage Armor")
        },
        &ctx,
    )
    .unwrap();
    assert!(plan.fiche.contains(&(
        1,
        EvenementFiche::SortLance {
            classe: "ensorceleur".into(),
            niveau_de_sort: 1,
            sort: "Mage Armor".into()
        }
    )));
    appliquer(&mut e, &plan.evenements, &ctx);
    assert_eq!(e.effets.actifs.len(), 1);
    assert_eq!(e.effets.actifs[0].duree, Duree::Heures(5));
    assert_eq!(e.effets.actifs[0].contenu.effets[0].valeur, 4);
}

#[test]
fn effet_hors_combat_reste_en_combat() {
    let ctx = ctx_avec(5, 0);
    let mut e = aventure(&ctx);
    let plan = lancer_hors_combat(
        &ParamsLancer {
            cibles: vec![1],
            ..lancer("Mage Armor")
        },
        &ctx,
    )
    .unwrap();
    appliquer(&mut e, &plan.evenements, &ctx);
    // le combat part de l'aventure : l'effet continu démarré hors combat s'y applique
    let mut c = Combat::nouveau(&e, vec![1, 2], &ctx).unwrap();
    assert_eq!(c.etat().aventure.effets.actifs.len(), 1);
    c.appliquer(ActionCombat::Initiative { pj: 1, d20: 10 }, &ctx)
        .unwrap();
    c.appliquer(ActionCombat::Initiative { pj: 2, d20: 5 }, &ctx)
        .unwrap();
    c.appliquer(ActionCombat::Demarrer { surprise: None }, &ctx)
        .unwrap();
    assert_eq!(c.etat().aventure.effets.actifs.len(), 1);
}

#[test]
fn lancement_defensif_refuse_hors_combat() {
    let ctx = ctx_avec(5, 0);
    let r = lancer_hors_combat(
        &ParamsLancer {
            defensif_d20: Some(15),
            ..lancer("Shield")
        },
        &ctx,
    );
    assert!(r.is_err());
}

#[test]
fn potion_hors_combat_s_applique_au_buveur() {
    let ctx = ctx_avec(5, 0);
    let mut e = aventure(&ctx);
    let id = ajouter(
        &mut e,
        objet("Potion de bouclier", GenreObjet::Potion, "Mage Armor", 1, 3),
        &ctx,
    );
    let plan = utiliser_objet_hors_combat(
        &e,
        &ParamsObjet {
            pj: 1,
            objet: id,
            ..Default::default()
        },
        &ctx,
    )
    .unwrap();
    appliquer(&mut e, &plan.evenements, &ctx);
    // potion d'un seul exemplaire : retirée de l'inventaire, effet ajouté
    assert!(e.pjs[&1].inventaire.is_empty());
    assert_eq!(e.effets.actifs.len(), 1);
    // durée d'après le niveau de lanceur de la potion (3 : 3 heures)
    assert_eq!(e.effets.actifs[0].duree, Duree::Heures(3));
}

#[test]
fn baguette_hors_combat_decompte_les_charges() {
    let ctx = ctx_avec(5, 0);
    let mut e = aventure(&ctx);
    let mut b = objet(
        "Baguette de bouclier",
        GenreObjet::Baguette,
        "Mage Armor",
        1,
        5,
    );
    b.charges = Some(Charges {
        restantes: 3,
        max: 10,
    });
    let id = ajouter(&mut e, b, &ctx);
    let plan = utiliser_objet_hors_combat(
        &e,
        &ParamsObjet {
            pj: 1,
            objet: id,
            ..Default::default()
        },
        &ctx,
    )
    .unwrap();
    appliquer(&mut e, &plan.evenements, &ctx);
    let reste = e.pjs[&1].inventaire[&id].charges.unwrap();
    assert_eq!((reste.restantes, reste.max), (2, 10));
}

#[test]
fn parchemin_hors_combat_exige_le_test_de_niveau_de_lanceur() {
    let ctx = ctx_avec(5, 0);
    let mut e = aventure(&ctx);
    // parchemin de niveau de lanceur 8 : Mialee en a 5, un test est donc demandé
    let id = ajouter(
        &mut e,
        objet(
            "Parchemin de bouclier",
            GenreObjet::Parchemin,
            "Mage Armor",
            1,
            8,
        ),
        &ctx,
    );
    let sans_d20 = ParamsObjet {
        pj: 1,
        objet: id,
        ..Default::default()
    };
    assert!(utiliser_objet_hors_combat(&e, &sans_d20, &ctx).is_err());
    let reussi = ParamsObjet {
        niveau_de_lanceur_d20: Some(18),
        cibles: vec![1],
        ..sans_d20
    };
    let plan = utiliser_objet_hors_combat(&e, &reussi, &ctx).unwrap();
    appliquer(&mut e, &plan.evenements, &ctx);
    assert!(e.pjs[&1].inventaire.is_empty());
    assert_eq!(e.effets.actifs.len(), 1);
}

#[test]
fn parchemin_rate_avec_accident_evite_n_est_pas_consomme() {
    let ctx = ctx_avec(5, 0);
    let mut e = aventure(&ctx);
    let id = ajouter(
        &mut e,
        objet(
            "Parchemin de bouclier",
            GenreObjet::Parchemin,
            "Mage Armor",
            1,
            8,
        ),
        &ctx,
    );
    // d20 de 2 + 5 = 7 contre DD 9 : raté ; Sagesse 15 (+1) : pas d'accident
    let plan = utiliser_objet_hors_combat(
        &e,
        &ParamsObjet {
            pj: 1,
            objet: id,
            niveau_de_lanceur_d20: Some(2),
            accident_d20: Some(15),
            ..Default::default()
        },
        &ctx,
    )
    .unwrap();
    assert!(plan
        .evenements
        .iter()
        .all(|ev| !matches!(ev, EvenementAventure::ObjetUtilise { .. })));
    appliquer(&mut e, &plan.evenements, &ctx);
    assert_eq!(e.pjs[&1].inventaire.len(), 1);
}

#[test]
fn objet_modifie_garde_l_identifiant_et_respecte_les_regles() {
    let ctx = ctx_avec(5, 0);
    let mut e = aventure(&ctx);
    let mut b = objet("Baguette", GenreObjet::Baguette, "Mage Armor", 1, 5);
    b.charges = Some(Charges {
        restantes: 50,
        max: 50,
    });
    let id = ajouter(&mut e, b.clone(), &ctx);
    // charges restantes choisies : 12 sur 50
    b.charges = Some(Charges {
        restantes: 12,
        max: 50,
    });
    b.niveau_de_lanceur = 7;
    e.appliquer(
        &EvenementAventure::ObjetModifie {
            pj: 1,
            id,
            objet: b.clone(),
        },
        &ctx,
    )
    .unwrap();
    let rendu = &e.pjs[&1].inventaire[&id];
    assert_eq!(rendu.charges.unwrap().restantes, 12);
    assert_eq!(rendu.niveau_de_lanceur, 7);
    // plus de charges restantes que le maximum : refusé
    b.charges = Some(Charges {
        restantes: 51,
        max: 50,
    });
    assert!(e
        .appliquer(
            &EvenementAventure::ObjetModifie {
                pj: 1,
                id,
                objet: b,
            },
            &ctx,
        )
        .is_err());
}
