//! Scénarios de combat : initiative, tours, annulation, fin transactionnelle, reprise.

use crate::*;
use aventure::{ContextePj, EtatAventure, EvenementAventure, IdPj};
use fiches::journal::EvenementFiche;
use fiches::{Effet, TypeBonus};
use temps::{Duree, Instant, MomentDuTour, ROUNDS_PAR_JOUR};

fn ctx(id: IdPj) -> Option<ContextePj> {
    let (nom, init, niveau, pv_max) = match id {
        1 => ("Mialee", 2, 5, 31),
        2 => ("Valéros", 1, 5, 44),
        3 => ("Seoni", 2, 5, 33),
        _ => return None,
    };
    Some(ContextePj {
        nom: nom.into(),
        niveau,
        pv_max,
        con_score: 12,
        con_mod: 1,
        initiative: init,
        dommages: vec![],
    })
}

fn base() -> EtatAventure {
    let mut e = EtatAventure::nouveau(Instant::depuis(1, 8, 0).unwrap(), false);
    for pj in [1, 2, 3] {
        e.appliquer(&EvenementAventure::Entree { pj }, &ctx)
            .unwrap();
    }
    e
}

fn init(c: &mut Combat, pj: IdPj, d20: i32) {
    c.appliquer(ActionCombat::Initiative { pj, d20 }, &ctx)
        .unwrap();
}

/// Mialee 12 > Seoni 8 > Valéros 6 : ordre 1, 3, 2.
fn combat_demarre() -> Combat {
    let mut c = Combat::nouveau(&base(), vec![1, 2, 3], &ctx).unwrap();
    init(&mut c, 1, 10);
    init(&mut c, 2, 5);
    init(&mut c, 3, 6);
    c.appliquer(ActionCombat::Demarrer { surprise: None }, &ctx)
        .unwrap();
    c
}

fn effet(pj: IdPj, nom: &str, duree: Duree) -> ActionCombat {
    ActionCombat::Aventure(EvenementAventure::EffetAjoute {
        pj,
        nom: nom.into(),
        source: "sort".into(),
        duree,
        effets: vec![Effet::new("ca", TypeBonus::Parade, 1, nom)],
        etats: vec![],
        soin_par_round: 0,
        protections: vec![],
    })
}

fn tour(c: &mut Combat) {
    c.appliquer(ActionCombat::TourSuivant, &ctx).unwrap();
}

#[test]
fn initiative_triee_et_egalites_departagees() {
    let mut c = Combat::nouveau(&base(), vec![1, 2, 3], &ctx).unwrap();
    init(&mut c, 1, 10);
    init(&mut c, 3, 10); // même total (12) et même modificateur (+2) que Mialee
    init(&mut c, 2, 5);
    assert_eq!(c.etat().egalites(), vec![vec![1, 3]]);
    let r = c.appliquer(ActionCombat::Demarrer { surprise: None }, &ctx);
    assert!(r.unwrap_err().to_string().contains("égalité"));
    assert_eq!(c.etat().phase, Phase::Initiative); // état intact après l'erreur
    c.appliquer(ActionCombat::Departage { pj: 1, d20: 4 }, &ctx)
        .unwrap();
    c.appliquer(ActionCombat::Departage { pj: 3, d20: 17 }, &ctx)
        .unwrap();
    assert_eq!(c.etat().ordre, vec![3, 1, 2]);
    c.appliquer(ActionCombat::Demarrer { surprise: None }, &ctx)
        .unwrap();
    assert_eq!(c.etat().actif, Some(3));
    assert_eq!(c.etat().round, 1);
}

#[test]
fn le_modificateur_departage_avant_le_tirage() {
    let mut c = Combat::nouveau(&base(), vec![1, 2], &ctx).unwrap();
    init(&mut c, 1, 5); // 5 + 2 = 7
    init(&mut c, 2, 6); // 6 + 1 = 7, modificateur plus faible
    assert!(c.etat().egalites().is_empty());
    assert_eq!(c.etat().ordre, vec![1, 2]);
}

#[test]
fn demarrer_exige_toutes_les_initiatives() {
    let mut c = Combat::nouveau(&base(), vec![1, 2], &ctx).unwrap();
    init(&mut c, 1, 10);
    let e = c
        .appliquer(ActionCombat::Demarrer { surprise: None }, &ctx)
        .unwrap_err();
    assert!(e.to_string().contains("Valéros"));
}

#[test]
fn les_rounds_font_avancer_l_horloge_de_six_secondes() {
    let mut c = combat_demarre();
    let debut = c.etat().aventure.horloge.maintenant;
    assert_eq!(c.etat().ordre, vec![1, 3, 2]);
    tour(&mut c);
    tour(&mut c);
    tour(&mut c); // le round 1 s'achève
    assert_eq!(c.etat().round, 2);
    assert_eq!(c.etat().actif, Some(1));
    assert_eq!(c.etat().aventure.horloge.maintenant, debut.plus(1));
    assert_eq!(c.etat().rounds_joues, 1);
}

#[test]
fn annuler_un_tour_suivant_restaure_l_effet_expire() {
    let mut c = combat_demarre();
    c.appliquer(effet(1, "Bouclier", Duree::Rounds(1)), &ctx)
        .unwrap();
    assert_eq!(c.etat().aventure.effets.actifs.len(), 1);
    tour(&mut c);
    tour(&mut c);
    tour(&mut c); // fin du round : Bouclier expire
    assert!(c.etat().aventure.effets.actifs.is_empty());
    assert_eq!(c.etat().aventure.effets.alertes().len(), 1);
    assert!(c
        .etat()
        .fiche
        .iter()
        .any(|(_, e)| matches!(e, EvenementFiche::EffetTemporaireRetire(_))));

    assert!(c.annuler(&ctx).unwrap());
    assert_eq!(c.etat().aventure.effets.actifs.len(), 1); // l'effet est revenu
    assert!(c.etat().aventure.effets.alertes().is_empty());
    assert_eq!(c.etat().round, 1);

    assert!(c.retablir(&ctx).unwrap());
    assert!(c.etat().aventure.effets.actifs.is_empty());
}

#[test]
fn effet_jusqu_au_prochain_tour() {
    let mut c = combat_demarre();
    // Mialee joue en premier : effet jusqu'au début de son prochain tour
    c.appliquer(
        effet(
            1,
            "Bouclier de feu",
            Duree::JusquauProchainTour(MomentDuTour::Debut),
        ),
        &ctx,
    )
    .unwrap();
    tour(&mut c);
    tour(&mut c);
    assert_eq!(c.etat().aventure.effets.actifs.len(), 1);
    tour(&mut c); // nouveau round, début du tour de Mialee
    assert_eq!(c.etat().actif, Some(1));
    assert!(c.etat().aventure.effets.actifs.is_empty());
}

#[test]
fn round_de_surprise() {
    let mut c = Combat::nouveau(&base(), vec![1, 2, 3], &ctx).unwrap();
    init(&mut c, 1, 10);
    init(&mut c, 2, 5);
    init(&mut c, 3, 6);
    let debut = c.etat().aventure.horloge.maintenant;
    c.appliquer(
        ActionCombat::Demarrer {
            surprise: Some(vec![3, 1]),
        },
        &ctx,
    )
    .unwrap();
    assert_eq!((c.etat().round, c.etat().actif), (0, Some(1)));
    tour(&mut c);
    assert_eq!(c.etat().actif, Some(3));
    tour(&mut c); // fin du round de surprise
    assert_eq!((c.etat().round, c.etat().actif), (1, Some(1)));
    assert_eq!(c.etat().aventure.horloge.maintenant, debut.plus(1));
    assert!(c
        .appliquer(
            ActionCombat::Demarrer {
                surprise: Some(vec![9])
            },
            &ctx
        )
        .is_err());
}

#[test]
fn retarder_son_action() {
    let mut c = combat_demarre(); // ordre 1 (12), 3 (8), 2 (6)
    assert!(c
        .appliquer(
            ActionCombat::Retarder {
                pj: 3,
                nouvelle_initiative: 1
            },
            &ctx
        )
        .is_err()); // ce n'est pas son tour
    assert!(c
        .appliquer(
            ActionCombat::Retarder {
                pj: 1,
                nouvelle_initiative: 12
            },
            &ctx
        )
        .is_err()); // pas plus bas
    c.appliquer(
        ActionCombat::Retarder {
            pj: 1,
            nouvelle_initiative: 7,
        },
        &ctx,
    )
    .unwrap();
    assert_eq!(c.etat().ordre, vec![3, 1, 2]);
    assert_eq!(c.etat().actif, Some(3));
}

#[test]
fn modifier_l_initiative_en_cours_de_route() {
    let mut c = combat_demarre();
    c.appliquer(ActionCombat::ModifierInitiative { pj: 2, total: 20 }, &ctx)
        .unwrap();
    assert_eq!(c.etat().ordre, vec![2, 1, 3]);
    assert_eq!(c.etat().actif, Some(1)); // le personnage qui joue ne change pas
}

#[test]
fn le_temps_horaire_est_suspendu_et_les_non_engages_refuses() {
    let mut c = Combat::nouveau(&base(), vec![1, 2], &ctx).unwrap();
    for ev in [
        EvenementAventure::Pas {
            pas: temps::Pas::Heure1,
        },
        EvenementAventure::AvancerTemps { rounds: 1 },
        EvenementAventure::Repos {
            pjs: vec![1],
            heures: 8,
            alitement_complet: false,
            interrompu: false,
        },
    ] {
        assert!(c.appliquer(ActionCombat::Aventure(ev), &ctx).is_err());
    }
    let e = c
        .appliquer(
            ActionCombat::Aventure(EvenementAventure::Degats {
                pj: 3,
                nombre: 4,
                letaux: true,
            }),
            &ctx,
        )
        .unwrap_err();
    assert!(e.to_string().contains("pas engagé"));
    assert!(c.actions.is_empty());
}

#[test]
fn annuler_retablir_et_rejeu_deterministe() {
    let mut c = combat_demarre();
    let degats = |n| {
        ActionCombat::Aventure(EvenementAventure::Degats {
            pj: 1,
            nombre: n,
            letaux: true,
        })
    };
    c.appliquer(degats(5), &ctx).unwrap();
    c.appliquer(degats(7), &ctx).unwrap();
    assert_eq!(c.etat().aventure.pjs[&1].vital.degats, 12);
    c.annuler(&ctx).unwrap();
    assert_eq!(c.etat().aventure.pjs[&1].vital.degats, 5);
    // une nouvelle action efface ce qu'on pouvait rétablir
    c.appliquer(degats(1), &ctx).unwrap();
    assert!(!c.peut_retablir());
    assert_eq!(c.etat().aventure.pjs[&1].vital.degats, 6);
    // rejouer depuis zéro redonne le même état
    let avant = c.etat().clone();
    c.reconstruire(&ctx).unwrap();
    assert_eq!(*c.etat(), avant);
    assert_eq!(c.etat().stats[&1].degats_subis, 6);
}

#[test]
fn fin_de_combat_apercu_puis_validation() {
    let mut c = combat_demarre();
    let debut = c.etat().aventure.horloge.maintenant;
    c.appliquer(effet(1, "Bénédiction", Duree::Rounds(20)), &ctx)
        .unwrap(); // dissipé
    c.appliquer(effet(1, "Hâte", Duree::Minutes(10)), &ctx)
        .unwrap(); // arrondi
    c.appliquer(effet(2, "Endurance", Duree::Heures(1)), &ctx)
        .unwrap(); // inchangé
    c.appliquer(
        ActionCombat::Aventure(EvenementAventure::Degats {
            pj: 2,
            nombre: 9,
            letaux: true,
        }),
        &ctx,
    )
    .unwrap();
    for _ in 0..3 {
        tour(&mut c);
    }
    let ap = c.apercu_fin(&ctx).unwrap();
    assert_eq!(ap.rounds_joues, 1);
    assert_eq!(ap.bilan.dissipes.len(), 1);
    assert_eq!(ap.bilan.dissipes[0].nom, "Bénédiction");
    assert_eq!(ap.bilan.dissipes[0].perdu, Some(19));
    assert_eq!(ap.bilan.arrondis.len(), 1); // Hâte : 99 rounds restants → 10 min
    assert_eq!(ap.persistants.len(), 2);
    let valeros = ap.par_pj.iter().find(|r| r.pj == 2).unwrap();
    assert_eq!(
        (
            valeros.pv_avant,
            valeros.pv_apres,
            valeros.stats.degats_subis
        ),
        (44, 35, 9)
    );

    // validation complète
    let r = c.valider(&Validation::Tout, &[], &ctx).unwrap();
    assert_eq!(r.aventure.horloge.maintenant, debut.plus(1));
    assert!(!r.aventure.horloge.en_combat());
    assert_eq!(r.aventure.pjs[&2].vital.degats, 9);
    assert_eq!(r.aventure.effets.actifs.len(), 2);
    assert!(r.sortie.fiche.iter().any(|(pj, e)| *pj == 1
        && matches!(e, EvenementFiche::EffetTemporaireRetire(n) if n.starts_with("Bénédiction"))));

    // contestation : on maintient la Bénédiction 5 minutes
    let id = ap.bilan.dissipes[0].id;
    let r = c
        .valider(&Validation::Tout, &[(id, Duree::Minutes(5))], &ctx)
        .unwrap();
    assert_eq!(r.aventure.effets.actifs.len(), 3);
    assert!(c
        .valider(&Validation::Tout, &[(999, Duree::Minutes(5))], &ctx)
        .is_err());

    // abandon : l'état d'avant, horloge au début
    let r = c.valider(&Validation::Abandon, &[], &ctx).unwrap();
    assert_eq!(r.aventure, base());
    assert!(r.sortie.fiche.is_empty());
}

#[test]
fn validation_partielle() {
    let mut c = combat_demarre();
    for pj in [1, 2] {
        c.appliquer(
            ActionCombat::Aventure(EvenementAventure::Degats {
                pj,
                nombre: 6,
                letaux: true,
            }),
            &ctx,
        )
        .unwrap();
    }
    c.appliquer(effet(2, "Hâte", Duree::Heures(1)), &ctx)
        .unwrap();
    let r = c.valider(&Validation::Partiel(vec![1]), &[], &ctx).unwrap();
    assert_eq!(r.aventure.pjs[&1].vital.degats, 6); // Mialee reporte
    assert_eq!(r.aventure.pjs[&2].vital.degats, 0); // Valéros revient à l'état d'avant
    assert!(r.aventure.effets.actifs.is_empty());
    assert!(r.sortie.fiche.iter().all(|(pj, _)| *pj == 1));
    assert!(c.valider(&Validation::Partiel(vec![9]), &[], &ctx).is_err());
}

#[test]
fn minuit_pendant_un_combat() {
    let mut b = EtatAventure::nouveau(Instant(ROUNDS_PAR_JOUR - 1), false); // 23:59:54
    b.appliquer(&EvenementAventure::Entree { pj: 1 }, &ctx)
        .unwrap();
    let mut c = Combat::nouveau(&b, vec![1], &ctx).unwrap();
    init(&mut c, 1, 10);
    c.appliquer(ActionCombat::Demarrer { surprise: None }, &ctx)
        .unwrap();
    tour(&mut c); // un seul combattant : le round s'achève, minuit est franchi
    let r = c.valider(&Validation::Tout, &[], &ctx).unwrap();
    assert_eq!(r.sortie.minuits, 1);
    assert_eq!(r.aventure.horloge.maintenant.jour(), 2);
}

#[test]
fn reprise_apres_plantage() {
    let mut c = combat_demarre();
    c.appliquer(effet(1, "Bouclier", Duree::Rounds(3)), &ctx)
        .unwrap();
    tour(&mut c);
    let dossier = std::env::temp_dir().join(format!("combat-test-{}", std::process::id()));
    std::fs::create_dir_all(&dossier).unwrap();
    let chemin = dossier.join("combat_en_cours.json");
    persistance::sauvegarder(&c, &chemin).unwrap();
    assert!(!chemin.with_extension("tmp").exists());
    let relu = persistance::charger(&chemin, &ctx).unwrap();
    assert_eq!(relu.etat(), c.etat());
    assert_eq!(relu.curseur, c.curseur);
    std::fs::remove_dir_all(&dossier).unwrap();
}
