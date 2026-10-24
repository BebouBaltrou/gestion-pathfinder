//! Scénarios du mode aventure : une journée de voyage, repos, faim et soif, recul d'horloge.

use crate::*;
use fiches::journal::EvenementFiche;
use fiches::{Carac, Effet, TypeBonus};
use temps::{Duree, Instant, Pas, ROUNDS_PAR_HEURE, ROUNDS_PAR_JOUR};

fn ctx(id: IdPj) -> Option<ContextePj> {
    match id {
        1 => Some(ContextePj {
            nom: "Mialee".into(),
            niveau: 5,
            pv_max: 31,
            con_score: 12,
            con_mod: 1,
            vigueur: 4,
            initiative: 2,
            dommages: vec![(Carac::Int, 3)],
        }),
        2 => Some(ContextePj {
            nom: "Valéros".into(),
            niveau: 3,
            pv_max: 30,
            con_score: 16,
            con_mod: 3,
            vigueur: 4,
            initiative: 1,
            dommages: vec![],
        }),
        _ => None,
    }
}

fn aventure(survie: bool) -> EtatAventure {
    // départ : jour 1, 08:00
    let mut e = EtatAventure::nouveau(Instant::depuis(1, 8, 0).unwrap(), survie);
    for pj in [1, 2] {
        e.appliquer(&EvenementAventure::Entree { pj }, &ctx)
            .unwrap();
    }
    e
}

fn hate() -> EvenementAventure {
    EvenementAventure::EffetAjoute {
        pj: 1,
        nom: "Hâte".into(),
        source: "sort, lancé par Mialee".into(),
        duree: Duree::Minutes(10),
        effets: vec![Effet::new("ca", TypeBonus::Esquive, 1, "Hâte")],
        etats: vec![],
        soin_par_round: 0,
        protections: vec![],
    }
}

fn repos(
    pjs: Vec<IdPj>,
    heures: u32,
    alitement_complet: bool,
    interrompu: bool,
) -> EvenementAventure {
    EvenementAventure::Repos {
        pjs,
        heures,
        alitement_complet,
        interrompu,
    }
}

fn degats(pj: IdPj, nombre: i32, letaux: bool) -> EvenementAventure {
    EvenementAventure::Degats { pj, nombre, letaux }
}

#[test]
fn effet_cree_puis_expire_avec_le_temps() {
    let mut e = aventure(false);
    let s = e.appliquer(&hate(), &ctx).unwrap();
    assert_eq!(s.fiche.len(), 1);
    let (pj, EvenementFiche::EffetTemporaireAjoute(t)) = &s.fiche[0] else {
        panic!("attendu : effet ajouté à la fiche");
    };
    assert_eq!((*pj, t.rounds_restants), (1, None));
    assert_eq!(t.nom, "Hâte #1");

    let s = e
        .appliquer(&EvenementAventure::Pas { pas: Pas::Minutes5 }, &ctx)
        .unwrap();
    assert!(s.expires.is_empty());
    let s = e
        .appliquer(&EvenementAventure::Pas { pas: Pas::Minutes5 }, &ctx)
        .unwrap();
    assert_eq!(s.expires, vec![1]);
    assert_eq!(
        s.fiche,
        vec![(1, EvenementFiche::EffetTemporaireRetire("Hâte #1".into()))]
    );
    assert_eq!(e.effets.alertes().len(), 1);
    e.appliquer(&EvenementAventure::AlerteConfirmee { id: 1 }, &ctx)
        .unwrap();
    assert!(e.effets.alertes().is_empty());
    assert!(e.journal.iter().any(|j| j.texte.contains("a pris fin")));
}

#[test]
fn minuit_avance_l_intermede() {
    let mut e = aventure(false);
    let s = e
        .appliquer(
            &EvenementAventure::AvancerTemps {
                rounds: 15 * ROUNDS_PAR_HEURE,
            },
            &ctx,
        )
        .unwrap();
    assert_eq!(s.minuits, 0); // 08:00 + 15 h = 23:00
    let s = e
        .appliquer(&EvenementAventure::Pas { pas: Pas::Heure1 }, &ctx)
        .unwrap();
    assert_eq!(s.minuits, 1);
    assert_eq!(e.horloge.maintenant.jour(), 2);
}

#[test]
fn repos_complet_guerit_et_rend_les_sorts() {
    let mut e = aventure(false);
    e.appliquer(&degats(1, 12, true), &ctx).unwrap();
    let s = e
        .appliquer(&repos(vec![1, 2], 8, false, false), &ctx)
        .unwrap();
    // niveau 5 : 5 PV rendus ; 3 points de dommages d'Intelligence : 1 récupéré
    assert_eq!(e.pjs[&1].vital.degats, 7);
    assert!(s.fiche.contains(&(1, EvenementFiche::Repos)));
    assert!(s.fiche.contains(&(2, EvenementFiche::Repos)));
    assert!(s.fiche.contains(&(
        1,
        EvenementFiche::DommagesCaracteristique {
            carac: Carac::Int,
            delta: -1
        }
    )));
    assert_eq!(e.horloge.maintenant, Instant::depuis(1, 16, 0).unwrap());
}

#[test]
fn repos_interrompu_ou_trop_court_ne_donne_rien() {
    let mut e = aventure(false);
    e.appliquer(&degats(1, 12, true), &ctx).unwrap();
    for (heures, interrompu) in [(8, true), (7, false)] {
        let s = e
            .appliquer(&repos(vec![1], heures, false, interrompu), &ctx)
            .unwrap();
        assert!(s.fiche.is_empty());
    }
    assert_eq!(e.pjs[&1].vital.degats, 12);
}

#[test]
fn alitement_complet_double_la_guerison() {
    let mut e = aventure(false);
    e.appliquer(&degats(1, 20, true), &ctx).unwrap();
    assert!(e.appliquer(&repos(vec![1], 12, true, false), &ctx).is_err());
    let s = e.appliquer(&repos(vec![1], 24, true, false), &ctx).unwrap();
    assert_eq!(e.pjs[&1].vital.degats, 10); // 2 fois le niveau 5
    assert!(s.fiche.contains(&(
        1,
        EvenementFiche::DommagesCaracteristique {
            carac: Carac::Int,
            delta: -2
        }
    )));
}

#[test]
fn les_soins_ne_depassent_pas_les_degats() {
    let mut e = aventure(false);
    e.appliquer(&degats(2, 4, true), &ctx).unwrap();
    e.appliquer(&EvenementAventure::Soin { pj: 2, nombre: 10 }, &ctx)
        .unwrap();
    assert_eq!(e.pjs[&2].vital.degats, 0);
}

#[test]
fn soif_test_rate_puis_boire() {
    let mut e = aventure(true);
    // Mialee, CON 12 : grâce 36 h, premier test à 37 h
    assert!(e.tests_survie_dus(&ctx).is_empty());
    e.appliquer(
        &EvenementAventure::AvancerTemps {
            rounds: 37 * ROUNDS_PAR_HEURE,
        },
        &ctx,
    )
    .unwrap();
    let dus = e.tests_survie_dus(&ctx);
    assert_eq!(dus.len(), 1); // Valéros, CON 16 : grâce 40 h, pas encore
    assert_eq!((dus[0].0, dus[0].1.dd), (1, 10));

    // d20 = 8, +1 = 9 < 10 : raté, il faut le 1d6
    let rate = EvenementAventure::TestSurvie {
        pj: 1,
        besoin: Besoin::Eau,
        d20: 8,
        degats: None,
    };
    assert!(e.appliquer(&rate, &ctx).is_err());
    assert_eq!(e.tests_survie_dus(&ctx).len(), 1); // état inchangé après l'erreur
    let s = e
        .appliquer(
            &EvenementAventure::TestSurvie {
                pj: 1,
                besoin: Besoin::Eau,
                d20: 8,
                degats: Some(4),
            },
            &ctx,
        )
        .unwrap();
    assert_eq!(
        s.fiche,
        vec![(1, EvenementFiche::EtatAjoute("fatigued".into()))]
    );
    assert_eq!(e.pjs[&1].vital.non_letaux, 4);
    assert_eq!(e.pjs[&1].vital.non_letaux_guerissables(), 0);
    assert!(e.tests_survie_dus(&ctx).is_empty());

    // réussi : pas de dégâts à saisir
    e.appliquer(&EvenementAventure::Pas { pas: Pas::Heure1 }, &ctx)
        .unwrap();
    assert!(e
        .appliquer(
            &EvenementAventure::TestSurvie {
                pj: 1,
                besoin: Besoin::Eau,
                d20: 15,
                degats: Some(2)
            },
            &ctx
        )
        .is_err());
    e.appliquer(
        &EvenementAventure::TestSurvie {
            pj: 1,
            besoin: Besoin::Eau,
            d20: 15,
            degats: None,
        },
        &ctx,
    )
    .unwrap();

    // boire libère les dégâts et retire la fatigue
    let s = e
        .appliquer(&EvenementAventure::Boire { pj: 1 }, &ctx)
        .unwrap();
    assert_eq!(
        s.fiche,
        vec![(1, EvenementFiche::EtatRetire("fatigued".into()))]
    );
    assert_eq!(e.pjs[&1].vital.non_letaux_guerissables(), 4);
}

#[test]
fn survie_desactivee() {
    let mut e = aventure(false);
    e.appliquer(
        &EvenementAventure::AvancerTemps {
            rounds: 100 * ROUNDS_PAR_HEURE,
        },
        &ctx,
    )
    .unwrap();
    assert!(e.tests_survie_dus(&ctx).is_empty());
    assert!(e
        .appliquer(
            &EvenementAventure::TestSurvie {
                pj: 1,
                besoin: Besoin::Eau,
                d20: 1,
                degats: Some(1)
            },
            &ctx
        )
        .is_err());
}

fn regle(h: u32, m: u32, reactiver: Vec<u64>, retirer_crees: bool) -> EvenementAventure {
    EvenementAventure::RegleHeure {
        instant: Instant::depuis(1, h, m).unwrap(),
        reactiver,
        retirer_crees,
    }
}

#[test]
fn recul_d_horloge() {
    let mut e = aventure(false);
    e.appliquer(&hate(), &ctx).unwrap(); // 08:00, 10 min
    e.appliquer(&EvenementAventure::AvancerTemps { rounds: 150 }, &ctx)
        .unwrap(); // 08:15, expiré
    assert!(e.effets.actifs.is_empty());
    let recul = Instant::depuis(1, 8, 5).unwrap();
    assert_eq!(e.apercu_recul(recul).reactivables, vec![1]);
    // sans réactiver : l'effet reste expiré
    let s = e.appliquer(&regle(8, 5, vec![], true), &ctx).unwrap();
    assert!(s.fiche.is_empty());
    assert!(e.effets.actifs.is_empty());
    assert_eq!(e.horloge.maintenant, recul);

    // avec réactivation : l'effet revient sur la fiche
    let mut e = aventure(false);
    e.appliquer(&hate(), &ctx).unwrap();
    e.appliquer(&EvenementAventure::AvancerTemps { rounds: 150 }, &ctx)
        .unwrap();
    let s = e.appliquer(&regle(8, 5, vec![1], true), &ctx).unwrap();
    assert!(matches!(
        s.fiche[0],
        (1, EvenementFiche::EffetTemporaireAjoute(_))
    ));
    assert_eq!(e.effets.actifs.len(), 1);
}

#[test]
fn recul_avant_minuit_refuse_et_etat_intact() {
    let mut e = aventure(false);
    e.appliquer(
        &EvenementAventure::AvancerTemps {
            rounds: ROUNDS_PAR_JOUR,
        },
        &ctx,
    )
    .unwrap();
    let avant = e.clone();
    assert!(e.appliquer(&regle(23, 0, vec![], true), &ctx).is_err());
    assert_eq!(e, avant);
}

#[test]
fn recul_demande_confirmation_pour_les_effets_crees() {
    let mut e = aventure(false);
    e.appliquer(&EvenementAventure::AvancerTemps { rounds: 100 }, &ctx)
        .unwrap();
    e.appliquer(&hate(), &ctx).unwrap();
    assert!(e.appliquer(&regle(8, 0, vec![], false), &ctx).is_err());
    assert_eq!(e.effets.actifs.len(), 1);
    let s = e.appliquer(&regle(8, 0, vec![], true), &ctx).unwrap();
    assert_eq!(
        s.fiche,
        vec![(1, EvenementFiche::EffetTemporaireRetire("Hâte #1".into()))]
    );
}

#[test]
fn le_non_letal_guerit_avec_le_temps() {
    let mut e = aventure(false);
    e.appliquer(&degats(2, 7, false), &ctx).unwrap();
    e.appliquer(
        &EvenementAventure::AvancerTemps {
            rounds: 2 * ROUNDS_PAR_HEURE,
        },
        &ctx,
    )
    .unwrap();
    assert_eq!(e.pjs[&2].vital.non_letaux, 1); // niveau 3 : 3 par heure
}

#[test]
fn pj_inconnu_et_erreurs_atomiques() {
    let mut e = aventure(false);
    assert!(e.appliquer(&degats(9, 1, true), &ctx).is_err());
    let avant = e.clone();
    // Mialee est valide, le personnage 3 n'existe pas : rien ne change
    assert!(e
        .appliquer(&repos(vec![1, 3], 8, false, false), &ctx)
        .is_err());
    assert_eq!(e, avant);
}

#[test]
fn evenements_et_etat_serialisables() {
    let ev = hate();
    let json = serde_json::to_string(&ev).unwrap();
    let relu: EvenementAventure = serde_json::from_str(&json).unwrap();
    assert_eq!(ev, relu);
    let e = aventure(true);
    let relu: EtatAventure = serde_json::from_str(&serde_json::to_string(&e).unwrap()).unwrap();
    assert_eq!(e, relu);
}
