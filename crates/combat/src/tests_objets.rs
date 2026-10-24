//! Objets en combat : potions, parchemins, baguettes, bâtons, objets à charges, ressources.

use crate::tests_sorts::ctx_avec;
use crate::*;
use aventure::{
    Charges, Consommable, ContextePj, EtatAventure, EvenementAventure, GenreObjet, IdPj, Ressource,
    UniteRessource,
};
use fiches::journal::EvenementFiche;
use temps::{Duree, Instant};

type Ctx = dyn Fn(IdPj) -> Option<ContextePj>;

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

fn baguette(sort: &str, niveau: u32, cl: u32, charges: u32) -> Consommable {
    Consommable {
        charges: Some(Charges {
            restantes: charges,
            max: charges,
        }),
        ..objet("Baguette", GenreObjet::Baguette, sort, niveau, cl)
    }
}

/// Combat où Mialee (1) porte `objets` ; elle joue avant Valéros (2).
fn combat_avec(objets: Vec<Consommable>, ctx: &Ctx) -> Combat {
    let mut e = EtatAventure::nouveau(Instant::depuis(1, 8, 0).unwrap(), false);
    for pj in [1, 2] {
        e.appliquer(&EvenementAventure::Entree { pj }, ctx).unwrap();
    }
    for o in objets {
        e.appliquer(&EvenementAventure::ObjetAjoute { pj: 1, objet: o }, ctx)
            .unwrap();
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

fn utiliser(objet: u64) -> ParamsObjet {
    ParamsObjet {
        pj: 1,
        objet,
        ..Default::default()
    }
}

fn effets_actifs(c: &Combat) -> usize {
    c.etat().aventure.effets.actifs.len()
}

#[test]
fn boire_une_potion() {
    let ctx = ctx_avec(5, 0);
    let mut p = objet(
        "Potion de force de taureau",
        GenreObjet::Potion,
        "Bull's Strength",
        2,
        5,
    );
    p.quantite = 2;
    let mut c = combat_avec(vec![p], &ctx);
    // même si on désigne un autre personnage, le buveur est la seule cible
    c.appliquer(
        ActionCombat::UtiliserObjet(ParamsObjet {
            cibles: vec![2],
            ..utiliser(1)
        }),
        &ctx,
    )
    .unwrap();
    let e = &c.etat().aventure.effets.actifs[0];
    assert_eq!(e.proprietaire, "1");
    assert_eq!(e.duree, Duree::Minutes(5)); // 1 min./niveau, niveau de lanceur de la potion
    assert_eq!(c.etat().aventure.pjs[&1].inventaire[&1].quantite, 1);
    // action simple dépensée
    assert!(c
        .appliquer(ActionCombat::UtiliserObjet(utiliser(1)), &ctx)
        .is_err());
    // annuler rend la potion et retire l'effet
    c.annuler(&ctx).unwrap();
    assert_eq!(c.etat().aventure.pjs[&1].inventaire[&1].quantite, 2);
    assert_eq!(effets_actifs(&c), 0);
}

#[test]
fn la_derniere_potion_disparait_de_l_inventaire() {
    let ctx = ctx_avec(5, 0);
    let p = objet("Potion", GenreObjet::Potion, "Shield", 1, 5);
    let mut c = combat_avec(vec![p], &ctx);
    c.appliquer(ActionCombat::UtiliserObjet(utiliser(1)), &ctx)
        .unwrap();
    assert!(c.etat().aventure.pjs[&1].inventaire.is_empty());
}

#[test]
fn parchemin_conditions_d_emploi() {
    let ctx = ctx_avec(5, 0);
    // parchemin divin : refusé à une lanceuse profane
    let mut divin = objet("Parchemin divin", GenreObjet::Parchemin, "Mage Armor", 1, 3);
    divin.arcanique = false;
    let mut c = combat_avec(vec![divin], &ctx);
    let e = c
        .appliquer(ActionCombat::UtiliserObjet(utiliser(1)), &ctx)
        .unwrap_err();
    assert!(e.to_string().contains("profane"));
    // score insuffisant : un sort de niveau 9 demande 19, elle a 16
    let souhait = objet("Parchemin de souhait", GenreObjet::Parchemin, "Wish", 9, 17);
    let mut c2 = combat_avec(vec![souhait], &ctx);
    let e = c2
        .appliquer(ActionCombat::UtiliserObjet(utiliser(1)), &ctx)
        .unwrap_err();
    assert!(e.to_string().contains("score de 19"));
    // sort qui n'est pas sur la liste de l'ensorceleur
    let sacre = objet(
        "Parchemin de bénédiction",
        GenreObjet::Parchemin,
        "Bless",
        1,
        1,
    );
    let mut c3 = combat_avec(vec![sacre], &ctx);
    let e = c3
        .appliquer(ActionCombat::UtiliserObjet(utiliser(1)), &ctx)
        .unwrap_err();
    assert!(e.to_string().contains("liste de sorts"));
    // rien n'a été dépensé
    assert!(c.etat().aventure.pjs[&1].inventaire.contains_key(&1));
}

#[test]
fn parchemin_de_niveau_de_lanceur_inferieur_ou_egal() {
    let ctx = ctx_avec(5, 0);
    let p = objet(
        "Parchemin d'armure du mage",
        GenreObjet::Parchemin,
        "Mage Armor",
        1,
        3,
    );
    let mut c = combat_avec(vec![p], &ctx);
    c.appliquer(
        ActionCombat::UtiliserObjet(ParamsObjet {
            cibles: vec![1],
            ..utiliser(1)
        }),
        &ctx,
    )
    .unwrap();
    // niveau de lanceur du parchemin (3) : 1 h par niveau = 3 h
    assert_eq!(c.etat().aventure.effets.actifs[0].duree, Duree::Heures(3));
    assert!(c.etat().aventure.pjs[&1].inventaire.is_empty());
}

fn parchemin_fort() -> Consommable {
    // niveau de lanceur 9 > 5 : test de niveau de lanceur DD 10
    objet(
        "Parchemin puissant",
        GenreObjet::Parchemin,
        "Mage Armor",
        1,
        9,
    )
}

#[test]
fn parchemin_test_de_niveau_de_lanceur_reussi() {
    let ctx = ctx_avec(5, 0);
    let mut c = combat_avec(vec![parchemin_fort()], &ctx);
    // le d20 est demandé
    assert!(c
        .appliquer(ActionCombat::UtiliserObjet(utiliser(1)), &ctx)
        .is_err());
    c.appliquer(
        ActionCombat::UtiliserObjet(ParamsObjet {
            cibles: vec![1],
            niveau_de_lanceur_d20: Some(5), // 5 + 5 = 10 : DD 10
            ..utiliser(1)
        }),
        &ctx,
    )
    .unwrap();
    assert_eq!(effets_actifs(&c), 1);
    assert_eq!(c.etat().aventure.effets.actifs[0].duree, Duree::Heures(9));
}

#[test]
fn parchemin_test_rate_sans_accident() {
    let ctx = ctx_avec(5, 0);
    let mut c = combat_avec(vec![parchemin_fort()], &ctx);
    let p = ParamsObjet {
        cibles: vec![1],
        niveau_de_lanceur_d20: Some(4), // 9 contre DD 10 : raté
        ..utiliser(1)
    };
    // il faut aussi le test de Sagesse
    assert!(c
        .appliquer(ActionCombat::UtiliserObjet(p.clone()), &ctx)
        .is_err());
    c.appliquer(
        ActionCombat::UtiliserObjet(ParamsObjet {
            accident_d20: Some(4), // 4 + 1 = 5 : DD 5 réussi
            ..p
        }),
        &ctx,
    )
    .unwrap();
    // pas d'accident : le parchemin reste, rien ne se passe, mais l'action est dépensée
    assert!(c.etat().aventure.pjs[&1].inventaire.contains_key(&1));
    assert_eq!(effets_actifs(&c), 0);
    assert!(c
        .appliquer(ActionCombat::UtiliserObjet(utiliser(1)), &ctx)
        .is_err());
}

#[test]
fn parchemin_accident() {
    let ctx = ctx_avec(5, 0);
    let mut c = combat_avec(vec![parchemin_fort()], &ctx);
    c.appliquer(
        ActionCombat::UtiliserObjet(ParamsObjet {
            cibles: vec![1],
            niveau_de_lanceur_d20: Some(4),
            accident_d20: Some(1), // 1 naturel : toujours raté
            ..utiliser(1)
        }),
        &ctx,
    )
    .unwrap();
    assert!(c.etat().aventure.pjs[&1].inventaire.is_empty()); // consommé
    assert_eq!(effets_actifs(&c), 0);
    assert!(c
        .etat()
        .journal
        .iter()
        .any(|j| j.texte.contains("accident de parchemin")));
}

#[test]
fn parchemin_et_echec_des_sorts_profanes() {
    let ctx = ctx_avec(5, 30);
    let p = objet("Parchemin", GenreObjet::Parchemin, "Mage Armor", 1, 3);
    let mut c = combat_avec(vec![p], &ctx);
    assert!(c
        .appliquer(ActionCombat::UtiliserObjet(utiliser(1)), &ctx)
        .is_err()); // d100 demandé
    c.appliquer(
        ActionCombat::UtiliserObjet(ParamsObjet {
            cibles: vec![1],
            echec_arcanique_d100: Some(12),
            ..utiliser(1)
        }),
        &ctx,
    )
    .unwrap();
    assert_eq!(effets_actifs(&c), 0); // le sort est perdu
    assert!(c.etat().aventure.pjs[&1].inventaire.is_empty()); // le parchemin est consommé
}

#[test]
fn baguette_charges_et_liste_de_classe() {
    let ctx = ctx_avec(5, 0);
    let mut c = combat_avec(
        vec![baguette("Shield", 1, 5, 2), baguette("Bless", 1, 1, 5)],
        &ctx,
    );
    // « Bless » n'est pas sur la liste de l'ensorceleur
    assert!(c
        .appliquer(ActionCombat::UtiliserObjet(utiliser(2)), &ctx)
        .is_err());
    c.appliquer(
        ActionCombat::UtiliserObjet(ParamsObjet {
            cibles: vec![1],
            ..utiliser(1)
        }),
        &ctx,
    )
    .unwrap();
    assert_eq!(
        c.etat().aventure.pjs[&1].inventaire[&1]
            .charges
            .unwrap()
            .restantes,
        1
    );
    assert_eq!(effets_actifs(&c), 1);
    assert_eq!(c.etat().stats[&1].objets_utilises, 1);
}

#[test]
fn baguette_vide() {
    let ctx = ctx_avec(5, 0);
    let mut c = combat_avec(vec![baguette("Shield", 1, 5, 1)], &ctx);
    c.appliquer(ActionCombat::UtiliserObjet(utiliser(1)), &ctx)
        .unwrap();
    c.appliquer(ActionCombat::TourSuivant, &ctx).unwrap();
    c.appliquer(ActionCombat::TourSuivant, &ctx).unwrap();
    let e = c
        .appliquer(ActionCombat::UtiliserObjet(utiliser(1)), &ctx)
        .unwrap_err();
    assert!(e.to_string().contains("charge"));
}

#[test]
fn baton_niveau_de_lanceur_du_porteur_et_charges() {
    let ctx = ctx_avec(5, 0);
    let baton = Consommable {
        charges: Some(Charges {
            restantes: 10,
            max: 10,
        }),
        ..objet("Bâton", GenreObjet::Baton, "Resist Energy", 2, 8)
    };
    let mut c = combat_avec(vec![baton], &ctx);
    c.appliquer(
        ActionCombat::UtiliserObjet(ParamsObjet {
            cibles: vec![1],
            charges: 2,
            choix_energie: Some("feu".into()),
            ..utiliser(1)
        }),
        &ctx,
    )
    .unwrap();
    assert_eq!(
        c.etat().aventure.pjs[&1].inventaire[&1]
            .charges
            .unwrap()
            .restantes,
        8
    );
    // niveau de lanceur 8 (celui du bâton, plus haut que le sien) : 10 min./niveau, résistance 20
    let e = &c.etat().aventure.effets.actifs[0];
    assert_eq!(e.duree, Duree::Minutes(80));
}

#[test]
fn objet_a_utilisations_quotidiennes_sans_sort() {
    let ctx = ctx_avec(5, 0);
    let o = Consommable {
        nom: "Anneau".into(),
        genre: GenreObjet::Charges,
        sort: None,
        niveau_du_sort: 0,
        niveau_de_lanceur: 0,
        arcanique: false,
        quantite: 1,
        charges: None,
        par_jour: Some(Charges {
            restantes: 1,
            max: 1,
        }),
        notes: String::new(),
    };
    let mut c = combat_avec(vec![o], &ctx);
    c.appliquer(
        ActionCombat::UtiliserObjet(ParamsObjet {
            type_action: Some(TypeAction::Rapide),
            ..utiliser(1)
        }),
        &ctx,
    )
    .unwrap();
    c.appliquer(ActionCombat::TourSuivant, &ctx).unwrap();
    c.appliquer(ActionCombat::TourSuivant, &ctx).unwrap();
    assert!(c
        .appliquer(ActionCombat::UtiliserObjet(utiliser(1)), &ctx)
        .is_err());
}

#[test]
fn les_munitions_se_decomptent_a_part() {
    let ctx = ctx_avec(5, 0);
    let m = Consommable {
        nom: "Flèches".into(),
        genre: GenreObjet::Munition,
        sort: None,
        niveau_du_sort: 0,
        niveau_de_lanceur: 0,
        arcanique: false,
        quantite: 20,
        charges: None,
        par_jour: None,
        notes: String::new(),
    };
    let mut c = combat_avec(vec![m], &ctx);
    assert!(c
        .appliquer(ActionCombat::UtiliserObjet(utiliser(1)), &ctx)
        .is_err());
    for _ in 0..3 {
        c.appliquer(
            ActionCombat::Aventure(EvenementAventure::ObjetUtilise {
                pj: 1,
                id: 1,
                charges: 1,
            }),
            &ctx,
        )
        .unwrap();
    }
    assert_eq!(c.etat().aventure.pjs[&1].inventaire[&1].quantite, 17);
    c.appliquer(
        ActionCombat::Aventure(EvenementAventure::ObjetRecupere {
            pj: 1,
            id: 1,
            nombre: 2,
        }),
        &ctx,
    )
    .unwrap();
    assert_eq!(c.etat().aventure.pjs[&1].inventaire[&1].quantite, 19);
}

#[test]
fn pouvoir_de_lignage_a_utilisations_quotidiennes() {
    let ctx = ctx_avec(5, 0);
    let mut c = combat_avec(vec![], &ctx);
    let ressources = aventure::ressources_de_lignage("aberrant", 5, 3);
    assert_eq!(ressources[0].1.max, 6); // 3 + modificateur de Charisme
    assert_eq!(ressources[0].1.unite, UniteRessource::Utilisations);
    c.appliquer(
        ActionCombat::Aventure(EvenementAventure::RessourcesDefinies { pj: 1, ressources }),
        &ctx,
    )
    .unwrap();
    for _ in 0..6 {
        c.appliquer(
            ActionCombat::Aventure(EvenementAventure::RessourceUtilisee {
                pj: 1,
                nom: "Acidic Ray".into(),
                quantite: 1,
            }),
            &ctx,
        )
        .unwrap();
    }
    let e = c.appliquer(
        ActionCombat::Aventure(EvenementAventure::RessourceUtilisee {
            pj: 1,
            nom: "Acidic Ray".into(),
            quantite: 1,
        }),
        &ctx,
    );
    assert!(e.is_err());
    let r: &Ressource = &c.etat().aventure.pjs[&1].ressources["Acidic Ray"];
    assert_eq!(r.restantes(), 0);
    // l'état se valide dans la campagne
    let res = c.valider(&Validation::Tout, &[], &ctx).unwrap();
    assert_eq!(res.aventure.pjs[&1].ressources["Acidic Ray"].utilisees, 6);
    // et une nuit de repos les rend (l'état d'aventure hors combat)
    let mut apres = res.aventure;
    apres
        .appliquer(
            &EvenementAventure::Repos {
                pjs: vec![1],
                heures: 8,
                alitement_complet: false,
                interrompu: false,
            },
            &ctx,
        )
        .unwrap();
    assert_eq!(apres.pjs[&1].ressources["Acidic Ray"].utilisees, 0);
    let _ = EvenementFiche::Repos;
}
