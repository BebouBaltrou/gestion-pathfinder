//! Tests de propriétés : durées, expirations et conversion de fin de combat.

use crate::*;
use proptest::prelude::*;

fn duree_strategie() -> impl Strategy<Value = Duree> {
    prop_oneof![
        (1u32..200).prop_map(Duree::Rounds),
        (1u32..120).prop_map(Duree::Minutes),
        (1u32..30).prop_map(Duree::Heures),
        (1u32..5).prop_map(Duree::Jours),
        Just(Duree::Permanente),
        Just(Duree::Concentration),
        Just(Duree::JusquaDecharge),
        Just(Duree::JusquauProchainTour(MomentDuTour::Fin)),
        Just(Duree::LieAUnEvenement("repos".into())),
    ]
}

fn ensemble(durees: &[(Duree, i64)]) -> Ensemble<u8> {
    let mut e = Ensemble::nouveau();
    for (i, (d, debut)) in durees.iter().enumerate() {
        e.ajouter(&format!("e{i}"), "s", "A", 0u8, d.clone(), Instant(*debut))
            .unwrap();
    }
    e
}

proptest! {
    #[test]
    fn instant_aller_retour(jour in 1i64..10_000, h in 0u32..24, m in 0u32..60) {
        let i = Instant::depuis(jour, h, m).unwrap();
        prop_assert_eq!((i.jour(), i.heure(), i.minute(), i.seconde()), (jour, h, m, 0));
    }

    #[test]
    fn minuits_franchis_egal_difference_de_jours(a in 0i64..1_000_000, d in 0i64..100_000) {
        let n = Instant::minuits_franchis(Instant(a), Instant(a + d));
        prop_assert_eq!(n, Instant(a + d).jour() - Instant(a).jour());
        prop_assert!(n <= d / ROUNDS_PAR_JOUR + 1);
    }

    /// Avancer en un grand pas ou en plusieurs petits donne exactement le même résultat.
    #[test]
    fn decoupage_du_temps_sans_effet(
        durees in prop::collection::vec((duree_strategie(), 0i64..500), 1..8),
        pas in prop::collection::vec(0i64..400, 1..10),
    ) {
        let mut un_pas = ensemble(&durees);
        let mut plusieurs = ensemble(&durees);
        let mut t = Instant(500);
        for p in &pas {
            t = t.plus(*p);
            plusieurs.avancer_a(t);
        }
        un_pas.avancer_a(t);
        let cle = |e: &Ensemble<u8>| {
            let mut v: Vec<_> = e.termines.iter().map(|x| (x.effet.id, x.a)).collect();
            v.sort();
            let mut a: Vec<_> = e.actifs.iter().map(|x| x.id).collect();
            a.sort();
            (v, a)
        };
        prop_assert_eq!(cle(&un_pas), cle(&plusieurs));
    }

    /// Aucun effet actif n'a une échéance déjà passée après `avancer_a`.
    #[test]
    fn rien_d_expire_reste_actif(
        durees in prop::collection::vec((duree_strategie(), 0i64..500), 1..8),
        t in 0i64..50_000,
    ) {
        let mut e = ensemble(&durees);
        e.avancer_a(Instant(t));
        for a in &e.actifs {
            if let Echeance::A(fin) = a.echeance {
                prop_assert!(fin > Instant(t));
            }
        }
    }

    #[test]
    fn minutes_arrondies_proches(r in 1i64..100_000) {
        let m = minutes_arrondies(r);
        prop_assert!(m >= 1);
        if r >= 5 {
            prop_assert!((m * 10 - r).abs() <= 5);
        } else {
            prop_assert_eq!(m, 1);
        }
    }

    #[test]
    fn conversion_de_fin_de_combat(
        durees in prop::collection::vec((duree_strategie(), 0i64..100), 1..10),
        rounds in 0i64..60,
    ) {
        let debut = Instant(100);
        let mut e = ensemble(&durees);
        let maintenant = debut.plus(rounds);
        let avant: Vec<_> = e.actifs.clone();
        let bilan = e.fin_de_combat(maintenant);
        for a in &e.actifs {
            // ni durée en rounds ni « jusqu'à… » ne survit
            prop_assert!(!a.duree.dissipee_en_fin_de_combat());
            let origine = avant.iter().find(|x| x.id == a.id).unwrap();
            match &a.duree {
                Duree::Minutes(_) => {
                    let Echeance::A(fin) = a.echeance else { unreachable!() };
                    let reste = fin.ecart(maintenant);
                    prop_assert!(reste >= 10 && reste % 10 == 0);
                }
                // heures, jours, permanent, concentration, décharge : inchangés
                _ => prop_assert_eq!(a.echeance, origine.echeance),
            }
        }
        // tout ce qui a été dissipé figure dans le bilan
        let n_diss = e.termines.iter().filter(|t| t.raison == RaisonFin::Dissipe).count();
        prop_assert_eq!(n_diss, bilan.dissipes.len());
        // deuxième conversion au même instant : rien de plus
        let bilan2 = e.fin_de_combat(maintenant);
        prop_assert!(bilan2.dissipes.is_empty() && bilan2.arrondis.is_empty());
    }

    /// Reculer sans rien réactiver ne ressuscite jamais un effet expiré.
    #[test]
    fn recul_sans_resurrection(
        durees in prop::collection::vec((duree_strategie(), 0i64..100), 1..8),
        t in 100i64..20_000,
        recul in 0i64..20_000,
    ) {
        let mut e = ensemble(&durees);
        e.avancer_a(Instant(t));
        let termines_avant = e.termines.len();
        let nouvel = Instant((t - recul).max(0));
        e.appliquer_recul(nouvel, &[], true).unwrap();
        // les effets créés après `nouvel` sont retirés, les autres expirés le restent
        let retires = durees.iter().filter(|(_, d)| Instant(*d) > nouvel).count();
        prop_assert!(e.termines.len() + retires >= termines_avant);
        prop_assert!(e.termines.len() <= termines_avant);
    }
}
