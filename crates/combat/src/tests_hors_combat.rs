//! Objets utilisés hors combat : soins saisis aux dés, effets, charges, erreurs.

use crate::tests_sorts::ctx_avec;
use crate::*;
use aventure::{
    Charges, Consommable, ContextePj, EtatAventure, EvenementAventure, GenreObjet, IdPj,
};
use fiches::journal::EvenementFiche;
use temps::Instant;

type Ctx = dyn Fn(IdPj) -> Option<ContextePj>;

fn objet(nom: &str, genre: GenreObjet, sort: &str, niveau: u32, cl: u32) -> Consommable {
    Consommable {
        nom: nom.into(),
        genre,
        sort: Some(sort.into()),
        niveau_du_sort: niveau,
        niveau_de_lanceur: cl,
        arcanique: true,
        quantite: 1,
        charges: None,
        par_jour: None,
        notes: String::new(),
    }
}

/// Mialee (1) porte `objets` hors combat.
fn aventure_avec(objets: Vec<Consommable>, ctx: &Ctx) -> EtatAventure {
    let mut e = EtatAventure::nouveau(Instant::depuis(1, 8, 0).unwrap(), false);
    e.appliquer(&EvenementAventure::Entree { pj: 1 }, ctx)
        .unwrap();
    for o in objets {
        e.appliquer(&EvenementAventure::ObjetAjoute { pj: 1, objet: o }, ctx)
            .unwrap();
    }
    e
}

fn id_objet(e: &EtatAventure) -> u64 {
    *e.pjs[&1].inventaire.keys().next().unwrap()
}

fn utiliser(id: u64) -> ParamsObjet {
    ParamsObjet {
        pj: 1,
        objet: id,
        ..Default::default()
    }
}

#[test]
fn potion_de_soin_saisie_aux_des() {
    let ctx = ctx_avec(5, 0);
    let e = aventure_avec(
        vec![objet(
            "Potion",
            GenreObjet::Potion,
            "Cure Light Wounds",
            1,
            3,
        )],
        &ctx,
    );
    let id = id_objet(&e);
    let s = evenements_objet_hors_combat(
        &e,
        &ParamsObjet {
            resultat_de: Some(4),
            ..utiliser(id)
        },
        &ctx,
    )
    .unwrap();
    // 1d8+3 au niveau 3 : dés 4 + 3 = 7 PV
    assert!(s
        .evenements
        .contains(&EvenementAventure::Soin { pj: 1, nombre: 7 }));
    assert!(s.evenements.iter().any(|ev| matches!(
        ev,
        EvenementAventure::ObjetUtilise {
            pj: 1,
            charges: 1,
            ..
        }
    )));
}

#[test]
fn soin_sans_des_demande_la_saisie() {
    let ctx = ctx_avec(5, 0);
    let e = aventure_avec(
        vec![objet(
            "Potion",
            GenreObjet::Potion,
            "Cure Light Wounds",
            1,
            3,
        )],
        &ctx,
    );
    let s = evenements_objet_hors_combat(&e, &utiliser(id_objet(&e)), &ctx).unwrap();
    assert!(!s
        .evenements
        .iter()
        .any(|ev| matches!(ev, EvenementAventure::Soin { .. })));
    assert!(s
        .notes
        .iter()
        .any(|n| n.contains("saisir le total des dés")));
}

#[test]
fn des_hors_limites_refuses_sans_depense() {
    let ctx = ctx_avec(5, 0);
    let e = aventure_avec(
        vec![objet(
            "Potion",
            GenreObjet::Potion,
            "Cure Light Wounds",
            1,
            3,
        )],
        &ctx,
    );
    let r = evenements_objet_hors_combat(
        &e,
        &ParamsObjet {
            resultat_de: Some(99),
            ..utiliser(id_objet(&e))
        },
        &ctx,
    );
    assert!(r.is_err());
}

#[test]
fn baguette_hors_combat_decompte_une_charge_et_donne_un_effet() {
    let ctx = ctx_avec(5, 0);
    let baguette = Consommable {
        charges: Some(Charges {
            restantes: 3,
            max: 3,
        }),
        ..objet("Baguette", GenreObjet::Baguette, "Bull's Strength", 2, 1)
    };
    let e = aventure_avec(vec![baguette], &ctx);
    let s = evenements_objet_hors_combat(
        &e,
        &ParamsObjet {
            cibles: vec![1],
            ..utiliser(id_objet(&e))
        },
        &ctx,
    )
    .unwrap();
    assert!(s
        .evenements
        .iter()
        .any(|ev| matches!(ev, EvenementAventure::ObjetUtilise { charges: 1, .. })));
    assert!(s
        .evenements
        .iter()
        .any(|ev| matches!(ev, EvenementAventure::EffetAjoute { pj: 1, .. })));
}

#[test]
fn niveau_de_lanceur_modifiable() {
    let ctx = ctx_avec(5, 0);
    let e = aventure_avec(
        vec![objet(
            "Potion",
            GenreObjet::Potion,
            "Cure Light Wounds",
            1,
            1,
        )],
        &ctx,
    );
    let s = evenements_objet_hors_combat(
        &e,
        &ParamsObjet {
            resultat_de: Some(4),
            niveau_de_lanceur: Some(5),
            ..utiliser(id_objet(&e))
        },
        &ctx,
    )
    .unwrap();
    // niveau 5 : 1d8+5 -> 4 + 5 = 9
    assert!(s
        .evenements
        .contains(&EvenementAventure::Soin { pj: 1, nombre: 9 }));
}

#[test]
fn objet_inconnu_refuse() {
    let ctx = ctx_avec(5, 0);
    let e = aventure_avec(vec![], &ctx);
    assert!(evenements_objet_hors_combat(&e, &utiliser(42), &ctx).is_err());
}

#[test]
fn parchemin_profane_refuse_a_un_lanceur_divin() {
    let ctx = ctx_avec(5, 0);
    let mut parchemin = objet(
        "Parchemin",
        GenreObjet::Parchemin,
        "Cure Light Wounds",
        1,
        1,
    );
    parchemin.arcanique = false;
    let e = aventure_avec(vec![parchemin], &ctx);
    assert!(evenements_objet_hors_combat(&e, &utiliser(id_objet(&e)), &ctx).is_err());
}

// ---- sorts hors combat

use crate::sorts::ParamsLancer;

fn druide(
    ctx: &Ctx,
    prepares: Vec<(u32, String)>,
    restants_niv1: u32,
) -> impl Fn(IdPj) -> Option<ContextePj> + '_ {
    move |id| {
        ctx(id).map(|mut c| {
            if let Some(i) = c.incantation.as_mut() {
                i.spontane = false;
                i.classe_en = "druid".into();
                i.classe = "druide".into();
                i.sorts_connus = prepares.iter().map(|(_, n)| n.clone()).collect();
                i.prepares_disponibles = prepares
                    .iter()
                    .map(|(n, s)| (*n, s.clone(), Vec::new(), None))
                    .collect();
                if let Some(e) = i.emplacements.iter_mut().find(|e| e.niveau_de_sort == 1) {
                    e.restants = restants_niv1;
                }
            }
            c
        })
    }
}

fn lancer(sort: &str, cibles: Vec<IdPj>) -> ParamsLancer {
    ParamsLancer {
        pj: 1,
        sort: sort.into(),
        cibles,
        ..Default::default()
    }
}

#[test]
fn sort_prepare_hors_combat_marque_le_lancer_dans_la_fiche() {
    let base = ctx_avec(5, 0);
    let ctx = druide(&base, vec![(1, "Faerie Fire".into())], 2);
    let e = aventure_avec(vec![], &base);
    let s = lancer_hors_combat(&lancer("Faerie Fire", vec![]), &ctx).unwrap();
    assert_eq!(
        s.fiche,
        Some(EvenementFiche::SortLance {
            classe: "druide".into(),
            niveau_de_sort: 1,
            sort: "Faerie Fire".into(),
        })
    );
    let _ = e;
}

#[test]
fn sort_non_prepare_refuse_hors_combat() {
    let base = ctx_avec(5, 0);
    let ctx = druide(&base, vec![(1, "Faerie Fire".into())], 2);
    assert!(lancer_hors_combat(&lancer("Magic Missile", vec![1]), &ctx).is_err());
}

#[test]
fn emplacement_epuise_refuse_hors_combat() {
    let base = ctx_avec(5, 0);
    let ctx = druide(&base, vec![(1, "Faerie Fire".into())], 0);
    assert!(lancer_hors_combat(&lancer("Faerie Fire", vec![1]), &ctx).is_err());
}

#[test]
fn sort_spontane_hors_combat_donne_un_effet_qui_dure_jusqu_au_combat() {
    let ctx = ctx_avec(5, 0);
    let mut e = aventure_avec(vec![], &ctx);
    let s = lancer_hors_combat(&lancer("Bull's Strength", vec![1]), &ctx).unwrap();
    for ev in &s.evenements {
        e.appliquer(ev, &ctx).unwrap();
    }
    assert_eq!(e.effets.actifs.len(), 1);
    // l'effet reste actif quand un combat commence ensuite
    e.appliquer(&EvenementAventure::Entree { pj: 2 }, &ctx)
        .unwrap();
    let c = crate::Combat::nouveau(&e, vec![1, 2], &ctx).unwrap();
    assert_eq!(c.etat().aventure.effets.actifs.len(), 1);
}
