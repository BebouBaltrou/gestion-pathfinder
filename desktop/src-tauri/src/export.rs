//! Export et import de campagnes : choix du fichier par une boîte de dialogue du système.
//! En simulation, les exports portent sur la campagne simulée.

use crate::session::{Retour, Session};
use std::path::Path;

fn nom_fichier(nom: &str) -> String {
    nom.chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' || c == ' ' {
                c
            } else {
                '_'
            }
        })
        .collect::<String>()
        .trim()
        .to_string()
}

/// Ouvre un fichier ou un dossier avec l'application par défaut du système.
pub fn ouvrir_externe(p: &Path) {
    #[cfg(windows)]
    let r = std::process::Command::new("explorer").arg(p).spawn();
    #[cfg(not(windows))]
    let r = std::process::Command::new("xdg-open").arg(p).spawn();
    let _ = r;
}

pub fn ouvrir_dossier(x: &Session) -> Retour {
    let _ = std::fs::create_dir_all(&x.dossier);
    ouvrir_externe(&x.dossier);
    Retour::ok("Dossier des campagnes ouvert")
}

fn ecrire(chemin: &Path, contenu: &str, quoi: &str, ouvrir: bool) -> Retour {
    match std::fs::write(chemin, contenu) {
        Ok(()) => {
            if ouvrir {
                ouvrir_externe(chemin);
            }
            Retour::ok(format!("{quoi} enregistré : {}", chemin.display()))
        }
        Err(e) => Retour::erreur(format!("Écriture impossible ({}) : {e}", chemin.display())),
    }
}

/// Nature de l'export demandé.
pub enum Export {
    /// Copie complète de la campagne (.json).
    Campagne,
    /// Page lisible, à imprimer en PDF depuis le navigateur.
    Recapitulatif,
    /// Historique pour un tableur (.csv).
    Historique,
}

pub fn exporter(x: &mut Session, quoi: Export) -> Retour {
    let r = x.r.clone();
    let Some(c) = x.campagne.as_mut() else {
        return Retour::erreur("Aucune campagne ouverte");
    };
    let base = nom_fichier(&c.nom);
    let (titre, fichier, filtre, extensions, libelle, ouvrir) = match quoi {
        Export::Campagne => (
            "Exporter la campagne",
            format!("{base}.json"),
            "Campagne",
            &["json"][..],
            "Export",
            false,
        ),
        Export::Recapitulatif => (
            "Récapitulatif de la campagne",
            format!("{base} - récapitulatif.html"),
            "Page web",
            &["html"][..],
            "Récapitulatif",
            true,
        ),
        Export::Historique => (
            "Historique de la campagne",
            format!("{base} - historique.csv"),
            "Tableur",
            &["csv"][..],
            "Historique",
            false,
        ),
    };
    let Some(chemin) = rfd::FileDialog::new()
        .set_title(titre)
        .set_file_name(fichier)
        .add_filter(filtre, extensions)
        .save_file()
    else {
        return Retour::ok(format!("{libelle} annulé"));
    };
    let contenu = match quoi {
        Export::Campagne => match campagne::fichier::vers_json(c) {
            Ok(j) => j,
            Err(e) => return Retour::erreur(format!("Export impossible : {e}")),
        },
        Export::Recapitulatif => campagne::rapport::html(c, &r),
        Export::Historique => campagne::rapport::historique_csv(c, &r),
    };
    ecrire(&chemin, &contenu, libelle, ouvrir)
}

/// Ajoute une campagne exportée (.json) aux campagnes et l'ouvre ; un fichier existant n'est
/// jamais écrasé (le nom reçoit un numéro).
pub fn importer(x: &mut Session) -> Retour {
    let Some(source) = rfd::FileDialog::new()
        .set_title("Importer une campagne")
        .add_filter("Campagne", &["json"])
        .pick_file()
    else {
        return Retour::ok("Import annulé");
    };
    let c = match campagne::fichier::charger(&source) {
        Ok(c) => c,
        Err(e) => return Retour::erreur(format!("Import impossible : {e}")),
    };
    let _ = std::fs::create_dir_all(&x.dossier);
    let base = nom_fichier(&c.nom);
    let mut cible = x.dossier.join(format!("{base}.json"));
    let mut n = 2;
    while cible.exists() {
        cible = x.dossier.join(format!("{base} ({n}).json"));
        n += 1;
    }
    match campagne::fichier::enregistrer(&c, &cible) {
        Ok(()) => x.ouvrir(&cible),
        Err(e) => Retour::erreur(format!("Import impossible : {e}")),
    }
}
