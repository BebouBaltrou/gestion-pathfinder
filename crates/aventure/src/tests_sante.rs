//! Santé (hors de combat, mourant, stable, mort), états par effets, dégâts typés.

use crate::*;
use fiches::journal::EvenementFiche;
use temps::{Duree, Instant, Pas};

fn ctx(id: IdPj) -> Option<ContextePj> {
    (id == 1).then(|| ContextePj {
        nom: "Mialee".into(),
        niveau: 5,
        pv_max: 31,
        con_score: 12,
        con_mod: 1,
        vigueur: 4,
        initiative: 2,
        dommages: vec![],
    })
}

fn aventure() -> EtatAventure {
    let mut e = EtatAventure::nouveau(Instant::depuis(1, 8, 0).unwrap(), false);
    e.appliquer(&EvenementAventure::Entree { pj: 1 }, &ctx)
        .unwrap();
    e
}

fn degats(nombre: i32, letaux: bool) -> EvenementAventure {
    EvenementAventure::Degats {
        pj: 1,
        nombre,
        letaux,
    }
}

fn etats(s: &Sortie) -> Vec<String> {
    s.fiche
        .iter()
        .map(|(_, e)| match e {
            EvenementFiche::EtatAjoute(x) => format!("+{x}"),
            EvenementFiche::EtatRetire(x) => format!("-{x}"),
            _ => "?".to_string(),
        })
        .collect()
}

fn sante(e: &EtatAventure) -> Sante {
    let c = ctx(1).unwrap();
    e.pjs[&1].vital.sante(c.pv_max, c.con_score)
}

#[test]
fn seuils_de_sante_et_etats_de_la_fiche() {
    let mut e = aventure();
    assert_eq!(sante(&e), Sante::Conscient);
    let s = e.appliquer(&degats(31, true), &ctx).unwrap(); // 0 PV
    assert_eq!(sante(&e), Sante::HorsDeCombat);
    assert_eq!(etats(&s), vec!["+disabled", "+staggered"]);
    let s = e.appliquer(&degats(1, true), &ctx).unwrap(); // -1
    assert_eq!(sante(&e), Sante::Mourant);
    assert_eq!(
        etats(&s),
        vec!["+dying", "+unconscious", "-disabled", "-staggered"]
    );
    let s = e.appliquer(&degats(10, true), &ctx).unwrap(); // -11 : encore vivant
    assert_eq!(sante(&e), Sante::Mourant);
    assert!(etats(&s).is_empty());
    let s = e.appliquer(&degats(1, true), &ctx).unwrap(); // -12 = -Con : mort
    assert_eq!(sante(&e), Sante::Mort);
    assert_eq!(etats(&s), vec!["+dead", "-dying", "-unconscious"]);
}

#[test]
fn soigner_retire_les_etats() {
    let mut e = aventure();
    e.appliquer(&degats(35, true), &ctx).unwrap(); // -4, mourant
    let s = e
        .appliquer(&EvenementAventure::Soin { pj: 1, nombre: 10 }, &ctx)
        .unwrap();
    assert_eq!(sante(&e), Sante::Conscient);
    assert_eq!(etats(&s), vec!["-dying", "-unconscious"]);
}

#[test]
fn stabilisation() {
    let mut e = aventure();
    let jet = |d20| EvenementAventure::Stabilisation { pj: 1, d20 };
    assert!(e.appliquer(&jet(10), &ctx).is_err()); // pas mourant
    e.appliquer(&degats(35, true), &ctx).unwrap(); // -4
                                                   // d20 8 + 1 (Con.) - 4 = 5 < 10 : raté, perd 1 PV
    e.appliquer(&jet(8), &ctx).unwrap();
    assert_eq!(e.pjs[&1].vital.pv_actuels(31), -5);
    assert_eq!(sante(&e), Sante::Mourant);
    // d20 14 + 1 - 5 = 10 : réussi (DD 10), stable
    let s = e.appliquer(&jet(14), &ctx).unwrap();
    assert_eq!(sante(&e), Sante::Stable);
    assert_eq!(etats(&s), vec!["+stable", "-dying"]);
    assert!(e.appliquer(&jet(14), &ctx).is_err()); // un stable ne refait pas de jet
                                                   // de nouveaux dégâts le font replonger (lecture à valider)
    e.appliquer(&degats(1, true), &ctx).unwrap();
    assert_eq!(sante(&e), Sante::Mourant);
}

#[test]
fn vingt_naturel_reussit_toujours() {
    let mut e = aventure();
    e.appliquer(&degats(42, true), &ctx).unwrap(); // -11 : malus énorme
    e.appliquer(&EvenementAventure::Stabilisation { pj: 1, d20: 20 }, &ctx)
        .unwrap();
    assert_eq!(sante(&e), Sante::Stable);
}

#[test]
fn un_soin_stabilise_meme_sans_remonter_a_zero() {
    let mut e = aventure();
    e.appliquer(&degats(40, true), &ctx).unwrap(); // -9
    e.appliquer(&EvenementAventure::Soin { pj: 1, nombre: 2 }, &ctx)
        .unwrap(); // -7
    assert_eq!(sante(&e), Sante::Stable);
}

#[test]
fn degats_non_letaux_chancelant_puis_inconscient() {
    let mut e = aventure();
    e.appliquer(&degats(21, true), &ctx).unwrap(); // 10 PV
    let s = e.appliquer(&degats(10, false), &ctx).unwrap();
    assert_eq!(sante(&e), Sante::Chancelant);
    assert_eq!(etats(&s), vec!["+staggered"]);
    let s = e.appliquer(&degats(1, false), &ctx).unwrap();
    assert_eq!(sante(&e), Sante::Inconscient);
    assert_eq!(etats(&s), vec!["+unconscious", "-staggered"]);
}

fn effet_etat(nom: &str, etat: &str, duree: Duree) -> EvenementAventure {
    EvenementAventure::EffetAjoute {
        pj: 1,
        nom: nom.into(),
        source: "sort".into(),
        duree,
        effets: vec![],
        etats: vec![etat.into()],
        soin_par_round: 0,
        protections: vec![],
    }
}

#[test]
fn un_etat_dure_autant_que_son_effet_et_se_partage() {
    let mut e = aventure();
    let s = e
        .appliquer(&effet_etat("Terreur", "shaken", Duree::Minutes(5)), &ctx)
        .unwrap();
    assert_eq!(etats(&s), vec!["?", "+shaken"]); // effet temporaire puis état
                                                 // un second effet donne le même état : rien de nouveau pour la fiche
    let s = e
        .appliquer(&effet_etat("Chant", "shaken", Duree::Minutes(10)), &ctx)
        .unwrap();
    assert!(!etats(&s).contains(&"+shaken".to_string()));
    // le premier expire : l'état reste (le second le donne encore)
    let s = e
        .appliquer(&EvenementAventure::Pas { pas: Pas::Minutes5 }, &ctx)
        .unwrap();
    assert!(!etats(&s).contains(&"-shaken".to_string()));
    assert!(e.pjs[&1].etats_fiche.contains("shaken"));
    // le second expire : l'état part
    let s = e
        .appliquer(
            &EvenementAventure::Pas {
                pas: Pas::Minutes10,
            },
            &ctx,
        )
        .unwrap();
    assert!(etats(&s).contains(&"-shaken".to_string()));
    assert!(e.pjs[&1].etats_fiche.is_empty());
}

#[test]
fn un_etat_permanent_se_retire_a_la_main() {
    let mut e = aventure();
    e.appliquer(
        &effet_etat("Malédiction", "sickened", Duree::Permanente),
        &ctx,
    )
    .unwrap();
    e.appliquer(&EvenementAventure::Pas { pas: Pas::Heure1 }, &ctx)
        .unwrap();
    assert!(e.pjs[&1].etats_fiche.contains("sickened"));
    let s = e
        .appliquer(&EvenementAventure::EffetRetire { id: 1 }, &ctx)
        .unwrap();
    assert!(etats(&s).contains(&"-sickened".to_string()));
}

fn protege(protections: Vec<Protection>) -> EvenementAventure {
    EvenementAventure::EffetAjoute {
        pj: 1,
        nom: "Protection".into(),
        source: "sort".into(),
        duree: Duree::Heures(1),
        effets: vec![],
        etats: vec![],
        soin_par_round: 0,
        protections,
    }
}

fn typés(nombre: i32, t: &str, contourne_rd: bool) -> EvenementAventure {
    EvenementAventure::DegatsTypes {
        pj: 1,
        nombre,
        type_degats: t.into(),
        contourne_rd,
        letaux: true,
    }
}

#[test]
fn resistance_immunite_et_rd() {
    let mut e = aventure();
    e.appliquer(
        &protege(vec![
            Protection::Resistance {
                type_degats: "feu".into(),
                valeur: 10,
            },
            Protection::Resistance {
                type_degats: "feu".into(),
                valeur: 5,
            },
            Protection::Immunite {
                type_degats: "froid".into(),
            },
            Protection::Reduction { valeur: 5 },
        ]),
        &ctx,
    )
    .unwrap();
    let pv = |e: &EtatAventure| e.pjs[&1].vital.degats;
    e.appliquer(&typés(20, "feu", false), &ctx).unwrap(); // la plus haute résistance : 20 - 10
    assert_eq!(pv(&e), 10);
    e.appliquer(&typés(30, "froid", false), &ctx).unwrap(); // immunité
    assert_eq!(pv(&e), 10);
    e.appliquer(&typés(8, "tranchant", false), &ctx).unwrap(); // RD 5 : 3
    assert_eq!(pv(&e), 13);
    e.appliquer(&typés(8, "tranchant", true), &ctx).unwrap(); // contourne la RD : 8
    assert_eq!(pv(&e), 21);
    e.appliquer(&typés(8, "feu", false), &ctx).unwrap(); // la RD ne touche pas l'énergie ; résistance 10 : 0
    assert_eq!(pv(&e), 21);
    let dernier = e.journal.last().unwrap();
    assert!(
        dernier.texte.contains("résistance au feu (Protection)"),
        "{}",
        dernier.texte
    );
}

#[test]
fn la_protection_prend_fin_avec_l_effet() {
    let mut e = aventure();
    e.appliquer(
        &protege(vec![Protection::Resistance {
            type_degats: "feu".into(),
            valeur: 10,
        }]),
        &ctx,
    )
    .unwrap();
    e.appliquer(&EvenementAventure::Pas { pas: Pas::Heure1 }, &ctx)
        .unwrap();
    e.appliquer(&typés(20, "feu", false), &ctx).unwrap();
    assert_eq!(e.pjs[&1].vital.degats, 20);
}

#[test]
fn guerison_acceleree_au_debut_du_tour() {
    let mut e = aventure();
    e.appliquer(&degats(10, true), &ctx).unwrap();
    e.appliquer(
        &EvenementAventure::EffetAjoute {
            pj: 1,
            nom: "Guérison accélérée".into(),
            source: "capacité".into(),
            duree: Duree::Heures(1),
            effets: vec![],
            etats: vec![],
            soin_par_round: 3,
            protections: vec![],
        },
        &ctx,
    )
    .unwrap();
    e.soins_de_debut_de_tour(1, &ctx);
    assert_eq!(e.pjs[&1].vital.degats, 7);
    e.soins_de_debut_de_tour(1, &ctx);
    e.soins_de_debut_de_tour(1, &ctx);
    e.soins_de_debut_de_tour(1, &ctx);
    assert_eq!(e.pjs[&1].vital.degats, 0); // jamais plus que les dégâts
}

fn gros_pj() -> EtatAventure {
    // 120 PV max : seuil de dégâts massifs = max(50, 60) = 60
    let mut e = aventure();
    e.degats_massifs = true;
    e
}

fn ctx_gros(id: IdPj) -> Option<ContextePj> {
    ctx(id).map(|mut c| {
        c.pv_max = 120;
        c
    })
}

#[test]
fn degats_massifs_seuil_et_jet_de_vigueur() {
    let mut e = gros_pj();
    let gros = |n| EvenementAventure::Degats {
        pj: 1,
        nombre: n,
        letaux: true,
    };
    e.appliquer(&gros(59), &ctx_gros).unwrap();
    assert!(e.massifs_demandes().is_empty()); // moins de la moitié des PV
    e.appliquer(&gros(60), &ctx_gros).unwrap();
    assert_eq!(e.massifs_demandes(), vec![1]);
    // Vigueur +4 : d20 10 + 4 = 14 < 15 : raté, mort quels que soient les PV
    e.appliquer(
        &EvenementAventure::DegatsMassifs { pj: 1, d20: 10 },
        &ctx_gros,
    )
    .unwrap();
    assert!(e.massifs_demandes().is_empty());
    let c = ctx_gros(1).unwrap();
    assert_eq!(e.pjs[&1].vital.sante(c.pv_max, c.con_score), Sante::Mort);
    assert!(e.pjs[&1].etats_fiche.contains("dead"));
}

#[test]
fn degats_massifs_reussi() {
    let mut e = gros_pj();
    e.appliquer(
        &EvenementAventure::Degats {
            pj: 1,
            nombre: 70,
            letaux: true,
        },
        &ctx_gros,
    )
    .unwrap();
    e.appliquer(
        &EvenementAventure::DegatsMassifs { pj: 1, d20: 11 },
        &ctx_gros,
    )
    .unwrap(); // 15
    let c = ctx_gros(1).unwrap();
    assert_eq!(
        e.pjs[&1].vital.sante(c.pv_max, c.con_score),
        Sante::Conscient
    );
    // plus de jet à saisir
    assert!(e
        .appliquer(
            &EvenementAventure::DegatsMassifs { pj: 1, d20: 11 },
            &ctx_gros
        )
        .is_err());
}

#[test]
fn degats_massifs_inactifs_par_defaut_et_minimum_de_50() {
    let mut e = aventure(); // règle désactivée
    e.appliquer(
        &EvenementAventure::Degats {
            pj: 1,
            nombre: 30,
            letaux: true,
        },
        &ctx,
    )
    .unwrap();
    assert!(e.massifs_demandes().is_empty());
    // règle active mais 31 PV max : 50 minimum, donc 49 ne déclenche rien
    e.degats_massifs = true;
    e.appliquer(&EvenementAventure::Soin { pj: 1, nombre: 30 }, &ctx)
        .unwrap();
    e.appliquer(
        &EvenementAventure::Degats {
            pj: 1,
            nombre: 20,
            letaux: true,
        },
        &ctx,
    )
    .unwrap();
    assert!(e.massifs_demandes().is_empty());
}

#[test]
fn degats_massifs_n_arrivent_pas_si_l_attaque_tue() {
    let mut e = gros_pj();
    e.appliquer(
        &EvenementAventure::Degats {
            pj: 1,
            nombre: 140,
            letaux: true,
        },
        &ctx_gros,
    )
    .unwrap();
    assert!(e.massifs_demandes().is_empty()); // 120 + 12 = mort d'office
}

#[test]
fn degats_continus_il_faut_un_soin_puis_la_stabilisation() {
    let mut e = aventure();
    e.appliquer(&degats(35, true), &ctx).unwrap(); // -4
    e.appliquer(
        &EvenementAventure::DegatsContinus { pj: 1, actif: true },
        &ctx,
    )
    .unwrap();
    let jet = EvenementAventure::Stabilisation { pj: 1, d20: 19 };
    let err = e.appliquer(&jet, &ctx).unwrap_err();
    assert!(err.to_string().contains("d'abord un soin"));
    // perte de début de tour : jet raté d'office (1) + 1 PV de plus
    e.pertes_continues_de_debut_de_tour(1, &ctx);
    assert_eq!(e.pjs[&1].vital.pv_actuels(31), -6);
    // un soin arrête les dégâts continus et stabilise
    e.appliquer(&EvenementAventure::Soin { pj: 1, nombre: 1 }, &ctx)
        .unwrap();
    assert!(!e.pjs[&1].vital.continu);
    assert_eq!(sante(&e), Sante::Stable);
    // plus de perte
    e.pertes_continues_de_debut_de_tour(1, &ctx);
    assert_eq!(e.pjs[&1].vital.pv_actuels(31), -5);
}
