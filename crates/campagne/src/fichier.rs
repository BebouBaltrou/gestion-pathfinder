//! Sauvegarde d'une campagne dans un fichier JSON lisible.
//!
//! Écriture atomique (fichier temporaire puis renommage) et copie de sécurité `.bak`
//! de la version précédente.

use crate::Campagne;
use std::path::Path;

/// Version du format de fichier ; à incrémenter en cas de changement incompatible.
pub const FORMAT: u32 = 1;

#[derive(serde::Serialize, serde::Deserialize)]
struct Fichier {
    format: u32,
    application: String,
    campagne: Campagne,
}

#[derive(Debug)]
pub enum ErreurFichier {
    Io(std::io::Error),
    Json(serde_json::Error),
    FormatInconnu(u32),
}

impl std::fmt::Display for ErreurFichier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ErreurFichier::Io(e) => write!(f, "erreur de fichier : {e}"),
            ErreurFichier::Json(e) => write!(f, "fichier illisible : {e}"),
            ErreurFichier::FormatInconnu(v) => {
                write!(f, "format de fichier {v} non pris en charge")
            }
        }
    }
}

impl std::error::Error for ErreurFichier {}

pub fn vers_json(c: &Campagne) -> Result<String, ErreurFichier> {
    let f = Fichier {
        format: FORMAT,
        application: "gestion-pathfinder".into(),
        campagne: c.clone(),
    };
    serde_json::to_string_pretty(&f).map_err(ErreurFichier::Json)
}

pub fn depuis_json(s: &str) -> Result<Campagne, ErreurFichier> {
    let f: Fichier = serde_json::from_str(s).map_err(ErreurFichier::Json)?;
    if f.format > FORMAT {
        return Err(ErreurFichier::FormatInconnu(f.format));
    }
    Ok(f.campagne)
}

pub fn enregistrer(c: &Campagne, chemin: &Path) -> Result<(), ErreurFichier> {
    let json = vers_json(c)?;
    if let Some(dossier) = chemin.parent() {
        std::fs::create_dir_all(dossier).map_err(ErreurFichier::Io)?;
    }
    let tmp = chemin.with_extension("json.tmp");
    std::fs::write(&tmp, json).map_err(ErreurFichier::Io)?;
    if chemin.exists() {
        let _ = std::fs::copy(chemin, chemin.with_extension("json.bak"));
    }
    std::fs::rename(&tmp, chemin).map_err(ErreurFichier::Io)
}

pub fn charger(chemin: &Path) -> Result<Campagne, ErreurFichier> {
    let s = std::fs::read_to_string(chemin).map_err(ErreurFichier::Io)?;
    depuis_json(&s)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::evenements::Evenement;
    use crate::modele::*;
    use domaine::Montant;

    #[test]
    fn aller_retour_json() {
        let r = regles::referentiel_officiel();
        let mut c = Campagne::nouvelle("Kingmaker");
        let pj = c.nouvel_id_personnage();
        c.executer(
            Evenement::PersonnageCree {
                id: pj,
                nom: "Amiri".into(),
                compte_initial: Montant::po(50),
                prestige: None,
            },
            &r,
        )
        .unwrap();
        let l = c.nouvel_id_localite();
        c.executer(
            Evenement::LocaliteCreee {
                id: l,
                nom: "Restov".into(),
                genre: "ville".into(),
                population: Some(14_000),
                categorie: CategorieLocalite::Officielle {
                    id: "grande-ville".into(),
                },
                notes: "Brevoy".into(),
            },
            &r,
        )
        .unwrap();
        c.executer(
            Evenement::InjectionPo {
                pj,
                localite: l,
                montant: Montant::pa(255),
            },
            &r,
        )
        .unwrap();
        c.annuler();
        let json = vers_json(&c).unwrap();
        let mut c2 = depuis_json(&json).unwrap();
        assert_eq!(c2.journal, c.journal);
        assert_eq!(c2.curseur, 2);
        assert_eq!(c2.etat(&r), c.etat(&r));
        assert!(c2.peut_retablir());

        let d = std::env::temp_dir().join(format!("gp-test-{}", std::process::id()));
        let f = d.join("campagne.json");
        enregistrer(&c, &f).unwrap();
        enregistrer(&c, &f).unwrap();
        assert!(f.with_extension("json.bak").exists());
        assert_eq!(charger(&f).unwrap().journal, c.journal);
        let _ = std::fs::remove_dir_all(d);
    }
}
