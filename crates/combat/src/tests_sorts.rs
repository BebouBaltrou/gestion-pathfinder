//! Sorts en combat : aperçu, lancement, métamagie, échecs, effets sur les personnages joués.

use crate::*;
use aventure::{ContextePj, EmplacementPj, EtatAventure, EvenementAventure, IdPj, IncantationPj};
use fiches::journal::EvenementFiche;
use temps::{Duree, Instant};

fn incantation(niveau_de_lanceur: u32, echec_pct: u32) -> IncantationPj {
    IncantationPj {
        classe_en: "sorcerer".into(),
        classe: "ensorceleur".into(),
        arcanique: true,
        spontane: true,
        niveau_de_lanceur,
        mod_incantation: 3,
        score_incantation: 16,
        mod_sagesse: 1,
        bonus_concentration: 0,
        emplacements: (1..=6)
            .map(|n| EmplacementPj {
                niveau_de_sort: n,
                restants: if n == 1 { 2 } else { 3 },
                dd: 10 + n as i32 + 3,
            })
            .collect(),
        sorts_connus: [
            "Light",
            "Mage Armor",
            "Shield",
            "Magic Missile",
            "Fireball",
            "Resist Energy",
            "Bull's Strength",
            "Haste",
            "Invisibility",
        ]
        .map(String::from)
        .to_vec(),
        metamagies: [
            "Empower Spell",
            "Quicken Spell",
            "Silent Spell",
            "Still Spell",
            "Extend Spell",
            "Heighten Spell",
        ]
        .map(String::from)
        .to_vec(),
        echec_arcanique_pct: echec_pct,
        metamagie_active: true,
    }
}

pub(crate) fn ctx_avec(
    niveau_de_lanceur: u32,
    echec_pct: u32,
) -> impl Fn(IdPj) -> Option<ContextePj> {
    move |id| {
        let (nom, inc) = match id {
            1 => ("Mialee", Some(incantation(niveau_de_lanceur, echec_pct))),
            2 => ("Valéros", None),
            _ => return None,
        };
        Some(ContextePj {
            nom: nom.into(),
            niveau: niveau_de_lanceur,
            pv_max: 40,
            con_score: 12,
            con_mod: 1,
            vigueur: 4,
            initiative: 2,
            incantation: inc,
            dommages: vec![],
        })
    }
}

/// Combat commencé : Mialee (12) joue avant Valéros (6).
fn combat(ctx: &dyn Fn(IdPj) -> Option<ContextePj>) -> Combat {
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

fn lancer(sort: &str) -> ParamsLancer {
    ParamsLancer {
        pj: 1,
        sort: sort.into(),
        ..Default::default()
    }
}

pub(crate) fn metas(l: &[&str]) -> Vec<String> {
    l.iter().map(|s| s.to_string()).collect()
}

fn restants(c: &Combat, niveau: u32) -> u32 {
    let ctx = ctx_avec(11, 0);
    let base = ctx(1).unwrap().incantation.unwrap();
    let total = base
        .emplacements
        .iter()
        .find(|e| e.niveau_de_sort == niveau)
        .unwrap()
        .restants;
    total
        - c.etat()
            .emplacements_depenses
            .get(&(1, niveau))
            .copied()
            .unwrap_or(0)
}

#[test]
fn apercu_d_une_boule_de_feu() {
    let ctx = ctx_avec(5, 30);
    let c = combat(&ctx);
    let a = c
        .etat()
        .apercu_sort(1, "Fireball", &[], None, &ctx)
        .unwrap();
    assert_eq!((a.niveau_du_sort, a.niveau_emplacement), (3, 3));
    assert_eq!((a.dd, a.niveau_de_lanceur), (16, 5)); // 10 + 3 + CHA +3
    assert_eq!(a.type_action, TypeAction::Simple);
    assert!(a.verbal && a.gestuel);
    assert_eq!(a.echec_arcanique_pct, 30);
    assert_eq!(a.dd_lancement_defensif, 21); // 15 + 2 × 3
    assert_eq!(a.bonus_concentration, 8); // niveau de lanceur 5 + CHA 3
    assert_eq!(a.duree_suggeree, Some(Duree::Instantanee));
    assert!(a.jet_de_sauvegarde_en.unwrap().contains("Reflex"));
    assert!(!a.effet_automatique);
}

#[test]
fn metamagie_niveau_d_emplacement_et_temps() {
    let ctx = ctx_avec(11, 0);
    let c = combat(&ctx);
    let ap = |m: &[&str], h| c.etat().apercu_sort(1, "Magic Missile", &metas(m), h, &ctx);
    // Empower (+2) : emplacement 3, DD du niveau 1 ; sorcier spontané : action complexe
    let a = ap(&["Empower Spell"], None).unwrap();
    assert_eq!(
        (a.niveau_emplacement, a.niveau_effectif, a.type_action),
        (3, 1, TypeAction::Complexe)
    );
    // Quicken (+4) : emplacement 5, action rapide
    let a = ap(&["Quicken Spell"], None).unwrap();
    assert_eq!(
        (a.niveau_emplacement, a.type_action),
        (5, TypeAction::Rapide)
    );
    // Quicken + Empower : 1 + 4 + 2 = 7 : pas d'emplacement de niveau 7
    assert!(ap(&["Quicken Spell", "Empower Spell"], None).is_err());
    // Heighten à 3 : le DD est celui du niveau 3, emplacement 3
    let a = ap(&["Heighten Spell"], Some(3)).unwrap();
    assert_eq!((a.niveau_effectif, a.niveau_emplacement, a.dd), (3, 3, 16));
    assert!(ap(&["Heighten Spell"], None).is_err());
    assert!(ap(&["Heighten Spell"], Some(1)).is_err());
    assert!(ap(&[], Some(3)).is_err());
    // don que le personnage n'a pas
    assert!(ap(&["Maximize Spell"], None).is_err());
}

#[test]
fn tour_de_magie_sans_emplacement() {
    let ctx = ctx_avec(5, 0);
    let mut c = combat(&ctx);
    c.appliquer(ActionCombat::Lancer(lancer("Light")), &ctx)
        .unwrap();
    c.appliquer(ActionCombat::TourSuivant, &ctx).unwrap();
    c.appliquer(ActionCombat::TourSuivant, &ctx).unwrap();
    c.appliquer(ActionCombat::Lancer(lancer("Light")), &ctx)
        .unwrap(); // à volonté
    assert!(c.etat().emplacements_depenses.is_empty());
    assert!(c
        .etat()
        .fiche
        .iter()
        .all(|(_, e)| !matches!(e, EvenementFiche::SortLance { .. })));
}

#[test]
fn lancer_une_armure_du_mage_sur_soi() {
    let ctx = ctx_avec(5, 0);
    let mut c = combat(&ctx);
    c.appliquer(
        ActionCombat::Lancer(ParamsLancer {
            cibles: vec![1],
            ..lancer("Mage Armor")
        }),
        &ctx,
    )
    .unwrap();
    // emplacement décompté, noté pour la fiche
    assert_eq!(c.etat().emplacements_depenses[&(1, 1)], 1);
    assert!(c.etat().fiche.contains(&(
        1,
        EvenementFiche::SortLance {
            classe: "ensorceleur".into(),
            niveau_de_sort: 1,
            sort: "Mage Armor".into()
        }
    )));
    // effet : +4 d'armure à la CA pour 1 heure par niveau (5 h), durée prise dans le livre
    let effet = &c.etat().aventure.effets.actifs[0];
    assert_eq!(effet.duree, Duree::Heures(5));
    assert_eq!(effet.contenu.effets.len(), 1);
    assert_eq!(effet.contenu.effets[0].valeur, 4);
    assert!(c
        .etat()
        .fiche
        .iter()
        .any(|(_, e)| matches!(e, EvenementFiche::EffetTemporaireAjoute(_))));
    assert_eq!(c.etat().stats[&1].sorts_lances, 1);
    // l'action simple est dépensée
    assert!(c
        .appliquer(ActionCombat::Lancer(lancer("Shield")), &ctx)
        .is_err());
    // annuler rend tout : emplacement, effet, événement de fiche
    c.annuler(&ctx).unwrap();
    assert!(c.etat().emplacements_depenses.is_empty());
    assert!(c.etat().aventure.effets.actifs.is_empty());
}

#[test]
fn lancer_sur_un_allie_et_duree_etendue() {
    let ctx = ctx_avec(5, 0);
    let mut c = combat(&ctx);
    c.appliquer(
        ActionCombat::Lancer(ParamsLancer {
            cibles: vec![2],
            metamagies: metas(&["Extend Spell"]),
            ..lancer("Bull's Strength")
        }),
        &ctx,
    )
    .unwrap();
    let e = &c.etat().aventure.effets.actifs[0];
    assert_eq!(e.proprietaire, "2");
    assert_eq!(e.duree, Duree::Minutes(10)); // 1 min./niveau × 5, doublé
    assert_eq!(e.contenu.effets[0].cible.0, "carac.for");
    // emplacement de niveau 2 + 1 = 3, en action complexe
    assert_eq!(c.etat().emplacements_depenses[&(1, 3)], 1);
}

#[test]
fn sort_sans_effet_automatique_decompte_et_note() {
    let ctx = ctx_avec(5, 0);
    let mut c = combat(&ctx);
    c.appliquer(
        ActionCombat::Lancer(ParamsLancer {
            cibles: vec![1],
            resultat_de: Some(17),
            ..lancer("Magic Missile")
        }),
        &ctx,
    )
    .unwrap();
    assert!(c.etat().aventure.effets.actifs.is_empty());
    let j = &c.etat().journal;
    assert!(j
        .iter()
        .any(|e| e.texte.contains("n'a pas d'effet automatique")));
    assert!(j
        .iter()
        .any(|e| e.texte.contains("résultat du dé du sort : 17")));
}

#[test]
fn echec_des_sorts_profanes() {
    let ctx = ctx_avec(5, 30);
    let mut c = combat(&ctx);
    let mage = |d100| ParamsLancer {
        cibles: vec![1],
        echec_arcanique_d100: d100,
        ..lancer("Mage Armor")
    };
    // le d100 est demandé
    assert!(c.appliquer(ActionCombat::Lancer(mage(None)), &ctx).is_err());
    assert!(c.etat().emplacements_depenses.is_empty());
    // 30 ou moins : le sort est perdu, l'emplacement aussi
    c.appliquer(ActionCombat::Lancer(mage(Some(30))), &ctx)
        .unwrap();
    assert_eq!(c.etat().emplacements_depenses[&(1, 1)], 1);
    assert!(c.etat().aventure.effets.actifs.is_empty());
    c.appliquer(ActionCombat::TourSuivant, &ctx).unwrap();
    c.appliquer(ActionCombat::TourSuivant, &ctx).unwrap();
    // 31 : le sort fonctionne
    c.appliquer(ActionCombat::Lancer(mage(Some(31))), &ctx)
        .unwrap();
    assert_eq!(c.etat().aventure.effets.actifs.len(), 1);
}

#[test]
fn still_spell_evite_l_echec_arcanique() {
    let ctx = ctx_avec(11, 30);
    let mut c = combat(&ctx);
    let a = c
        .etat()
        .apercu_sort(1, "Mage Armor", &metas(&["Still Spell"]), None, &ctx)
        .unwrap();
    assert!(!a.gestuel);
    assert_eq!(a.echec_arcanique_pct, 0);
    assert_eq!(a.niveau_emplacement, 2);
    c.appliquer(
        ActionCombat::Lancer(ParamsLancer {
            cibles: vec![1],
            metamagies: metas(&["Still Spell"]),
            ..lancer("Mage Armor")
        }),
        &ctx,
    )
    .unwrap();
    assert_eq!(c.etat().aventure.effets.actifs.len(), 1);
    // sort sans composante gestuelle : pas d'échec non plus (Light n'a pas de S)
    let a = c.etat().apercu_sort(1, "Light", &[], None, &ctx).unwrap();
    assert_eq!(a.echec_arcanique_pct, 0);
}

#[test]
fn lancement_defensif() {
    let ctx = ctx_avec(5, 0);
    let mut c = combat(&ctx);
    // Mage Armor, niveau 1 : DD 15 + 2 = 17 ; bonus 5 + 3 = 8
    let mage = |d20| ParamsLancer {
        cibles: vec![1],
        defensif_d20: Some(d20),
        ..lancer("Mage Armor")
    };
    c.appliquer(ActionCombat::Lancer(mage(8)), &ctx).unwrap(); // 16 < 17 : perdu
    assert!(c.etat().aventure.effets.actifs.is_empty());
    assert_eq!(c.etat().emplacements_depenses[&(1, 1)], 1);
    c.appliquer(ActionCombat::TourSuivant, &ctx).unwrap();
    c.appliquer(ActionCombat::TourSuivant, &ctx).unwrap();
    c.appliquer(ActionCombat::Lancer(mage(9)), &ctx).unwrap(); // 17 : réussi
    assert_eq!(c.etat().aventure.effets.actifs.len(), 1);
}

#[test]
fn plus_d_emplacement() {
    let ctx = ctx_avec(5, 0);
    let mut c = combat(&ctx);
    let lance = |c: &mut Combat| c.appliquer(ActionCombat::Lancer(lancer("Magic Missile")), &ctx);
    for _ in 0..2 {
        lance(&mut c).unwrap();
        c.appliquer(ActionCombat::TourSuivant, &ctx).unwrap();
        c.appliquer(ActionCombat::TourSuivant, &ctx).unwrap();
    }
    let e = lance(&mut c).unwrap_err(); // 2 emplacements de niveau 1 seulement
    assert!(e.to_string().contains("plus d'emplacement"));
    assert_eq!(restants(&c, 1), 0);
}

#[test]
fn sort_inconnu_ou_hors_de_portee() {
    let ctx = ctx_avec(5, 0);
    let mut c = combat(&ctx);
    assert!(c
        .appliquer(ActionCombat::Lancer(lancer("Wish")), &ctx)
        .is_err()); // pas connu
    assert!(c
        .appliquer(ActionCombat::Lancer(lancer("Pas un sort")), &ctx)
        .is_err());
    let mut p = lancer("Light");
    p.pj = 2; // Valéros ne lance pas de sorts
    assert!(c.appliquer(ActionCombat::TourSuivant, &ctx).is_ok());
    assert!(c.appliquer(ActionCombat::Lancer(p), &ctx).is_err());
}

#[test]
fn resistance_a_l_energie_selon_le_niveau() {
    let ctx = ctx_avec(11, 0); // niveau de lanceur 11 : résistance 30
    let mut c = combat(&ctx);
    let p = ParamsLancer {
        cibles: vec![1],
        ..lancer("Resist Energy")
    };
    // le type d'énergie est demandé
    assert!(c.appliquer(ActionCombat::Lancer(p.clone()), &ctx).is_err());
    c.appliquer(
        ActionCombat::Lancer(ParamsLancer {
            choix_energie: Some("feu".into()),
            ..p
        }),
        &ctx,
    )
    .unwrap();
    c.appliquer(
        ActionCombat::Aventure(EvenementAventure::DegatsTypes {
            pj: 1,
            nombre: 35,
            type_degats: "feu".into(),
            contourne_rd: false,
            letaux: true,
        }),
        &ctx,
    )
    .unwrap();
    assert_eq!(c.etat().aventure.pjs[&1].vital.degats, 5); // 35 - 30
    assert_eq!(
        c.etat().aventure.effets.actifs[0].duree,
        Duree::Minutes(110)
    ); // 10 min./niveau
}

#[test]
fn duree_a_saisir_quand_la_formule_n_est_pas_simple() {
    let ctx = ctx_avec(5, 0);
    let mut c = combat(&ctx);
    // Haste : « 1 round/level » : proposée ; on saisit quand même une autre durée si on veut
    c.appliquer(
        ActionCombat::Lancer(ParamsLancer {
            cibles: vec![1, 2],
            duree: Some(Duree::Rounds(4)),
            ..lancer("Haste")
        }),
        &ctx,
    )
    .unwrap();
    assert_eq!(c.etat().aventure.effets.actifs.len(), 2);
    assert!(c
        .etat()
        .aventure
        .effets
        .actifs
        .iter()
        .all(|e| e.duree == Duree::Rounds(4)));
}

#[test]
fn concentration_pour_maintenir_un_sort() {
    let ctx = ctx_avec(5, 0);
    let mut c = combat(&ctx);
    c.appliquer(
        ActionCombat::Aventure(EvenementAventure::EffetAjoute {
            pj: 1,
            nom: "Sort maintenu".into(),
            source: "sort".into(),
            duree: Duree::Concentration,
            effets: vec![],
            etats: vec![],
            soin_par_round: 0,
            protections: vec![],
        }),
        &ctx,
    )
    .unwrap();
    // blessure de 7 en maintenant un sort de niveau 2 : DD 10 + 7 + 2 = 19 ; bonus 8
    let test = |d20| ActionCombat::Concentration {
        pj: 1,
        d20,
        cause: Distraction::Blessure { degats: 7 },
        niveau_du_sort: 2,
        effet: Some(1),
    };
    c.appliquer(test(11), &ctx).unwrap(); // 19 : réussi
    assert_eq!(c.etat().aventure.effets.actifs.len(), 1);
    c.appliquer(test(10), &ctx).unwrap(); // 18 : le sort est rompu
    assert!(c.etat().aventure.effets.actifs.is_empty());
}

#[test]
fn un_combat_avec_des_sorts_se_rejoue_et_se_sauvegarde() {
    let ctx = ctx_avec(5, 0);
    let mut c = combat(&ctx);
    c.appliquer(
        ActionCombat::Lancer(ParamsLancer {
            cibles: vec![1],
            ..lancer("Mage Armor")
        }),
        &ctx,
    )
    .unwrap();
    let avant = c.etat().clone();
    c.reconstruire(&ctx).unwrap();
    assert_eq!(*c.etat(), avant);
    let dossier = std::env::temp_dir().join(format!("combat-sorts-{}", std::process::id()));
    std::fs::create_dir_all(&dossier).unwrap();
    let chemin = dossier.join("combat.json");
    persistance::sauvegarder(&c, &chemin).unwrap();
    let relu = persistance::charger(&chemin, &ctx).unwrap();
    assert_eq!(relu.etat(), c.etat());
    std::fs::remove_dir_all(&dossier).unwrap();
    // la validation reporte le sort sur la fiche
    let r = c.valider(&Validation::Tout, &[], &ctx).unwrap();
    assert!(r
        .sortie
        .fiche
        .iter()
        .any(|(_, e)| matches!(e, EvenementFiche::SortLance { .. })));
    assert_eq!(r.aventure.effets.actifs.len(), 1);
}

#[test]
fn la_metamagie_est_desactivee_tant_qu_elle_n_est_pas_integree() {
    let base = ctx_avec(11, 0);
    let ctx = move |id: IdPj| {
        base(id).map(|mut c| {
            if let Some(i) = c.incantation.as_mut() {
                i.metamagie_active = false;
            }
            c
        })
    };
    let c = combat(&ctx);
    let e = c
        .etat()
        .apercu_sort(1, "Magic Missile", &metas(&["Empower Spell"]), None, &ctx)
        .unwrap_err();
    assert!(e.to_string().contains("pas encore intégrée"));
    // sans métamagie, tout fonctionne comme avant
    assert!(c
        .etat()
        .apercu_sort(1, "Magic Missile", &[], None, &ctx)
        .is_ok());
}
