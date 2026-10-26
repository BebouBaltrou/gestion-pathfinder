//! Commandes des notes : ce fichier ne fait que relier l'interface au cœur (`notes`,
//! `campagne::evenements`, `vues::notes`). Prendre une note reste permis pendant un combat.

use crate::session::Retour;
use crate::{session, S};
use campagne::evenements::Evenement;
use notes::{Commande, Critere, Genre};
use std::time::{SystemTime, UNIX_EPOCH};
use vues::notes as vn;

/// Heure réelle, en secondes depuis 1970 (UTC).
fn maintenant() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| i64::try_from(d.as_secs()).unwrap_or(0))
}

#[tauri::command]
pub fn page_notes(s: S) -> Option<vn::PageNotes> {
    let mut x = session(&s);
    let r = x.r.clone();
    x.campagne.as_mut().map(|c| vn::page_notes(c, &r))
}

#[tauri::command]
pub fn liste_notes(s: S, critere: Critere) -> Vec<vn::NoteVue> {
    let mut x = session(&s);
    let r = x.r.clone();
    x.campagne
        .as_mut()
        .map(|c| vn::liste_notes(c, &r, &critere))
        .unwrap_or_default()
}

#[tauri::command]
pub fn chronologie(s: S, filtre: vn::FiltreChrono) -> Vec<vn::LigneChrono> {
    let mut x = session(&s);
    let r = x.r.clone();
    x.campagne
        .as_mut()
        .map(|c| vn::chronologie(c, &r, &filtre))
        .unwrap_or_default()
}

#[tauri::command]
pub fn creer_note(
    s: S,
    genre: Genre,
    rubrique: Option<String>,
    titre: String,
    texte: String,
    etiquettes: Vec<String>,
) -> Retour {
    let mut x = session(&s);
    let r = x.r.clone();
    let quand = {
        let Some(c) = x.campagne.as_mut() else {
            return Retour::erreur("Aucune campagne ouverte");
        };
        vn::quand_courant(c, &r, maintenant())
    };
    x.executer(
        Evenement::Notes(Commande::Creer {
            genre,
            rubrique,
            titre,
            texte,
            etiquettes,
            quand,
        }),
        "Note ajoutée",
    )
}

#[tauri::command]
pub fn modifier_note(
    s: S,
    id: u64,
    rubrique: Option<String>,
    titre: String,
    texte: String,
    etiquettes: Vec<String>,
) -> Retour {
    session(&s).executer(
        Evenement::Notes(Commande::Modifier {
            id,
            rubrique,
            titre,
            texte,
            etiquettes,
        }),
        "Note enregistrée",
    )
}

#[tauri::command]
pub fn epingler_note(s: S, id: u64, epingle: bool) -> Retour {
    session(&s).executer(
        Evenement::Notes(Commande::Epingler { id, epingle }),
        if epingle {
            "Note épinglée"
        } else {
            "Note désépinglée"
        },
    )
}

#[tauri::command]
pub fn supprimer_note(s: S, id: u64) -> Retour {
    session(&s).executer(
        Evenement::Notes(Commande::Supprimer { id }),
        "Note supprimée (annulable)",
    )
}

/// Note attachée à un objet de l'inventaire (provenance, histoire).
#[tauri::command]
pub fn noter_objet(s: S, id: u64, note: String) -> Retour {
    session(&s).executer_inventaires(
        vec![inventaire::Commande::DefinirNote { id, note }],
        "Note de l'objet enregistrée",
    )
}

/// Exporte les notes qui répondent au critère, en Markdown ou en texte, vers le fichier choisi.
#[tauri::command]
pub fn exporter_notes(s: S, critere: Critere) -> Retour {
    let mut x = session(&s);
    let r = x.r.clone();
    let Some(c) = x.campagne.as_mut() else {
        return Retour::erreur("Aucune campagne ouverte");
    };
    let contenu = vn::exporter_notes(c, &r, &critere);
    let nom: String = c
        .nom
        .chars()
        .map(|ch| {
            if ch.is_alphanumeric() || ch == ' ' || ch == '-' {
                ch
            } else {
                '_'
            }
        })
        .collect();
    let Some(chemin) = rfd::FileDialog::new()
        .set_title("Exporter les notes")
        .set_file_name(format!("{} - notes.md", nom.trim()))
        .add_filter("Markdown", &["md"])
        .add_filter("Texte", &["txt"])
        .save_file()
    else {
        return Retour::ok("Export annulé");
    };
    match std::fs::write(&chemin, contenu) {
        Ok(()) => Retour::ok(format!("Notes exportées : {}", chemin.display())),
        Err(e) => Retour::erreur(format!("Écriture impossible ({}) : {e}", chemin.display())),
    }
}
