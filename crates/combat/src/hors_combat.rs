//! Sorts et objets utilisés hors combat (phase aventure) : mêmes règles qu'en combat, sans action
//! à dépenser. Les effets continus créés ici vivent dans l'état d'aventure : un combat commencé
//! ensuite en fait une copie, donc ils s'appliquent pendant le combat.
//!
//! Ces fonctions ne modifient rien : elles renvoient un `Plan` (événements d'aventure et de fiche)
//! que la campagne applique d'un bloc.

use crate::etat::{ErreurCombat, EtatCombat};
use crate::objets::{
    charges_a_depenser, controler_saisies, echec_arcanique, invalide, lancement, sort_de_l_objet,
    test_lanceur, ParamsObjet, TestLanceur,
};
use crate::sorts::{nom_fr, ParamsLancer};
use aventure::{ContextePj, ErreurAventure, EtatAventure, EvenementAventure, IdPj};
use fiches::journal::EvenementFiche;

/// Ce qu'une action hors combat produit.
#[derive(Debug, Clone, Default)]
pub struct Plan {
    /// Événements du mode aventure, dans l'ordre (les remarques y figurent aussi).
    pub evenements: Vec<EvenementAventure>,
    /// Événements de fiche (emplacement de sort dépensé) : (personnage, événement).
    pub fiche: Vec<(IdPj, EvenementFiche)>,
}

impl Plan {
    fn remarque(&mut self, texte: String) {
        self.evenements.push(EvenementAventure::Remarque { texte });
    }
}

/// Lance un sort hors combat : emplacement dépensé, effets automatiques ajoutés à l'aventure.
pub fn lancer_hors_combat(
    a: &ParamsLancer,
    ctx: &dyn Fn(IdPj) -> Option<ContextePj>,
) -> Result<Plan, ErreurCombat> {
    if a.defensif_d20.is_some() {
        return invalide("le lancement défensif se joue en combat");
    }
    // les emplacements restants viennent de la fiche : aucune dépense en cours à déduire
    let p = EtatCombat::preparer_sort(
        a.pj,
        &a.sort,
        &a.metamagies,
        a.niveau_heighten,
        ctx,
        &|_| 0,
        &|_, _| 0,
    )?;
    let c = ctx(a.pj).ok_or(ErreurAventure::PjInconnu(a.pj))?;
    if p.restants == 0 {
        return invalide(format!(
            "plus d'emplacement de sort de niveau {} pour {}",
            p.niveau_emplacement, c.nom
        ));
    }
    for cible in &a.cibles {
        if ctx(*cible).is_none() {
            return Err(ErreurAventure::PjInconnu(*cible).into());
        }
    }
    if p.echec_pct > 0 && a.echec_arcanique_d100.is_none() {
        return invalide(format!(
            "échec des sorts profanes : saisir le d100 (échec à {} % ou moins)",
            p.echec_pct
        ));
    }
    if let Some(d) = a.echec_arcanique_d100 {
        if !(1..=100).contains(&d) {
            return invalide("d100 : de 1 à 100");
        }
    }
    let cl = p.inc.niveau_de_lanceur;
    let mut plan = Plan::default();
    let metas_txt = if p.metas.is_empty() {
        String::new()
    } else {
        format!(
            " ({})",
            p.metas
                .iter()
                .map(|m| m.nom_en.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        )
    };
    plan.remarque(format!(
        "{} lance {}{metas_txt} : emplacement de niveau {}, DD {}, niveau de lanceur {cl}.",
        c.nom,
        nom_fr(&p.sort.nom_en),
        p.niveau_emplacement,
        p.dd
    ));
    plan.remarque(format!(
        "{} : {}",
        nom_fr(&p.sort.nom_en),
        crate::stats_sorts::resume(p.sort, cl, p.dd)
    ));
    if let Some(r) = a.resultat_de {
        plan.remarque(format!("{} : résultat du dé du sort : {r}.", c.nom));
    }
    // le sort est dépensé même s'il échoue (échec des sorts profanes)
    if p.niveau_emplacement > 0 {
        plan.fiche.push((
            a.pj,
            EvenementFiche::SortLance {
                classe: p.inc.classe.clone(),
                niveau_de_sort: p.niveau_emplacement,
                sort: p.sort.nom_en.clone(),
            },
        ));
    }
    if let Some((n, perdu)) = echec_arcanique(a.echec_arcanique_d100, p.echec_pct, &c.nom) {
        plan.remarque(n);
        if perdu {
            return Ok(plan);
        }
    }
    let effets = EtatCombat::evenements_de_sort(
        &p.sort.nom_en,
        cl,
        &c.nom,
        &a.cibles,
        a.duree.clone(),
        a.choix_energie.as_deref(),
        p.metas.iter().any(|m| m.double_duree),
    )?;
    plan.evenements.extend(effets.evenements);
    for n in effets.notes {
        plan.remarque(n);
    }
    Ok(plan)
}

/// Utilise un objet hors combat : potion, parchemin, baguette, bâton ou objet à charges.
pub fn utiliser_objet_hors_combat(
    etat: &EtatAventure,
    a: &ParamsObjet,
    ctx: &dyn Fn(IdPj) -> Option<ContextePj>,
) -> Result<Plan, ErreurCombat> {
    let c = ctx(a.pj).ok_or(ErreurAventure::PjInconnu(a.pj))?;
    let objet = etat
        .pjs
        .get(&a.pj)
        .and_then(|s| s.inventaire.get(&a.objet))
        .cloned()
        .ok_or_else(|| ErreurCombat::Invalide(format!("objet n° {} inconnu", a.objet)))?;
    let sort = sort_de_l_objet(&objet)?;
    let inc = c.incantation.clone();
    let l = lancement(&objet, sort, inc.as_ref(), a, &c.nom)?;
    controler_saisies(&objet, &l, inc.as_ref(), a)?;
    let mut plan = Plan::default();
    match test_lanceur(&objet, inc.as_ref(), a, &c.nom)? {
        TestLanceur::Inutile => {}
        TestLanceur::Reussi(n) => plan.remarque(n),
        TestLanceur::Evite(n) => {
            plan.remarque(n);
            return Ok(plan);
        }
        TestLanceur::Accident(n) => {
            plan.evenements.push(EvenementAventure::ObjetUtilise {
                pj: a.pj,
                id: a.objet,
                charges: 1,
            });
            plan.remarque(n);
            return Ok(plan);
        }
    }
    plan.evenements.push(EvenementAventure::ObjetUtilise {
        pj: a.pj,
        id: a.objet,
        charges: charges_a_depenser(&objet, a),
    });
    if let Some(r) = a.resultat_de {
        plan.remarque(format!("{} : résultat du dé : {r}.", c.nom));
    }
    if let Some((n, perdu)) = echec_arcanique(a.echec_arcanique_d100, l.echec_pct, &c.nom) {
        plan.remarque(n);
        if perdu {
            return Ok(plan);
        }
    }
    if let Some(nom_sort) = &objet.sort {
        let effets = EtatCombat::evenements_de_sort(
            nom_sort,
            l.cl,
            &c.nom,
            &l.cibles,
            a.duree.clone(),
            a.choix_energie.as_deref(),
            false,
        )?;
        plan.evenements.extend(effets.evenements);
        for n in effets.notes {
            plan.remarque(n);
        }
    }
    Ok(plan)
}
