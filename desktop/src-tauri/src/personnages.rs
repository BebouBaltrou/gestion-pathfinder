//! Commandes des personnages et des règles de campagne : ce fichier ne fait que relier
//! l'interface au cœur (`fiches`, `campagne::personnages`, `vues::personnage`).

use crate::session::Retour;
use crate::{session, S};
use campagne::evenements::Evenement;
use campagne::modele::IdPersonnage;
use campagne::personnages::{changer, registre, Reglage, RegleInfo};
use domaine::Montant;
use fiches::fiche::Fiche;
use fiches::journal::EvenementFiche;
use fiches::progression::Montee;
use vues::personnage as vp;

#[tauri::command]
pub fn liste_personnages(s: S) -> Vec<vp::LignePersonnage> {
    session(&s)
        .avec_etat(vp::liste_personnages)
        .unwrap_or_default()
}

/// `Ok(None)` : le PJ n'a pas encore de fiche.
#[tauri::command]
pub fn fiche_personnage(s: S, id: u32) -> Result<Option<vp::FicheVue>, String> {
    match session(&s).avec_etat(|e| vp::fiche_du_pj(e, IdPersonnage(id))) {
        Some(Some(Ok(f))) => Ok(Some(f)),
        Some(Some(Err(m))) => Err(m),
        _ => Ok(None),
    }
}

#[tauri::command]
pub fn catalogue_creation() -> vp::CatalogueCreation {
    vp::catalogue_creation()
}

/// Valeurs et problèmes d'une fiche en cours de saisie (rien n'est enregistré).
#[tauri::command]
pub fn apercu_fiche(fiche: Fiche) -> Result<vp::FicheVue, String> {
    vp::apercu_fiche(&fiche)
}

#[tauri::command]
pub fn creer_personnage(s: S, nom: String, po: i64) -> Retour {
    let mut x = session(&s);
    let Some(id) = x.avec_etat(|e| {
        IdPersonnage(e.personnages.keys().map(|k| k.0).max().unwrap_or(0) + 1)
    }) else {
        return Retour::erreur("Aucune campagne ouverte");
    };
    let nom = nom.trim().to_string();
    let message = format!("Personnage « {nom} » créé");
    x.executer(
        Evenement::PersonnageCree {
            id,
            nom,
            compte_initial: Montant::po(po.max(0)),
            prestige: None,
        },
        &message,
    )
}

#[tauri::command]
pub fn creer_fiche(s: S, pj: u32, fiche: Fiche) -> Retour {
    let message = format!("Fiche de {} créée", fiche.nom);
    session(&s).executer(
        Evenement::Fiche {
            pj: IdPersonnage(pj),
            evenement: EvenementFiche::Creation(Box::new(fiche)),
        },
        &message,
    )
}

/// Applique un événement de la fiche (XP, repos, sort lancé, équipement, état, montée…).
#[tauri::command]
pub fn evenement_fiche(s: S, pj: u32, evenement: EvenementFiche) -> Retour {
    let message = evenement.description();
    session(&s).executer(
        Evenement::Fiche {
            pj: IdPersonnage(pj),
            evenement,
        },
        &message,
    )
}

#[tauri::command]
pub fn apercu_montee(s: S, pj: u32, classe: String) -> Option<vp::ApercuMonteeVue> {
    session(&s)
        .avec_etat(|e| vp::apercu_montee(e, IdPersonnage(pj), &classe))
        .flatten()
}

#[tauri::command]
pub fn essai_montee(s: S, pj: u32, montee: Montee) -> Option<vp::EssaiMontee> {
    session(&s)
        .avec_etat(|e| vp::essai_montee(e, IdPersonnage(pj), &montee))
        .flatten()
}

/// Toutes les règles optionnelles de la campagne ouverte, avec leur valeur actuelle.
#[tauri::command]
pub fn regles_campagne(s: S) -> Vec<RegleInfo> {
    let mut x = session(&s);
    let Some(c) = x.campagne.as_mut() else {
        return Vec::new();
    };
    let o = c.options.clone();
    let r = x.r.clone();
    let p = x
        .campagne
        .as_mut()
        .map(|c| c.etat(&r).regles_personnages)
        .unwrap_or_default();
    registre(&o, &p)
}

#[tauri::command]
pub fn changer_regle(s: S, id: String, valeur: String) -> Retour {
    let mut x = session(&s);
    let r = x.r.clone();
    let Some((o, p)) = x.campagne.as_mut().map(|c| {
        let o = c.options.clone();
        let p = c.etat(&r).regles_personnages;
        (o, p)
    }) else {
        return Retour::erreur("Aucune campagne ouverte");
    };
    match changer(&id, &valeur, &o, &p) {
        Err(m) => Retour::erreur(m),
        Ok(Reglage::Personnages(n)) => x.executer(
            Evenement::ReglagePersonnages { regles: n },
            "Règle modifiée",
        ),
        Ok(Reglage::Intermede(n)) => x.regler_options(n, "Règle modifiée"),
    }
}
