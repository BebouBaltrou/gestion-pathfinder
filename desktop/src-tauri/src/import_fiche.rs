//! Importation d'une fiche de personnage PDF : choix du fichier, rapport de cohérence, puis
//! création du PJ avec les choix « corriger / garder » de l'utilisateur. Tout le calcul est dans
//! `importation` et `fiches` ; ce fichier ne fait que relier l'interface au cœur.

use crate::session::Retour;
use crate::{session, S};
use campagne::evenements::Evenement;
use campagne::modele::IdPersonnage;
use domaine::Montant;
use fiches::journal::EvenementFiche;
use fiches::progression::{xp_requis, Vitesse};
use importation::{appliquer_gardes, lire_champs, lire_feuille, proposer, Gravite, Proposition};
use serde::Serialize;
use std::path::Path;

#[derive(Serialize)]
pub struct LigneImport {
    pub cle: String,
    pub rubrique: String,
    pub libelle: String,
    pub champ: String,
    pub feuille: String,
    pub regles: String,
    pub explication: String,
    /// `accord`, `a_valider` ou `ecart`.
    pub gravite: &'static str,
    /// « Garder » est possible (valeur numérique que la fiche sait recevoir).
    pub peut_garder: bool,
    /// Écart ajouté si la valeur est gardée.
    pub delta: Option<i32>,
}

#[derive(Serialize)]
pub struct ApercuImport {
    pub chemin: String,
    pub nom: String,
    pub niveau: u32,
    pub pv: i32,
    pub dons: Vec<String>,
    pub lignes: Vec<LigneImport>,
    pub non_verifie: Vec<String>,
}

fn proposition(chemin: &str) -> Result<Proposition, String> {
    let champs = lire_champs(Path::new(chemin))?;
    let feuille = lire_feuille(&champs);
    proposer(&feuille, fiches::Referentiel::partage())
}

fn apercu(chemin: &str) -> Result<ApercuImport, String> {
    let p = proposition(chemin)?;
    let v = p
        .fiche
        .calculer(fiches::Referentiel::partage())
        .map_err(|e| format!("La fiche reconstruite est invalide : {e}"))?;
    Ok(ApercuImport {
        chemin: chemin.to_string(),
        nom: p.fiche.nom.clone(),
        niveau: p.niveau,
        pv: v.pv_max.total,
        dons: p.fiche.dons_choisis(),
        lignes: p
            .rapport
            .lignes
            .iter()
            .map(|l| LigneImport {
                cle: l.cle.clone(),
                rubrique: l.rubrique.to_string(),
                libelle: l.libelle.clone(),
                champ: l.champ.clone(),
                feuille: l.feuille.clone(),
                regles: l.regles.clone(),
                explication: l.explication.clone(),
                gravite: match l.gravite {
                    Gravite::Accord => "accord",
                    Gravite::AValider => "a_valider",
                    Gravite::Ecart => "ecart",
                },
                peut_garder: l.garde.is_some(),
                delta: l.garde.as_ref().map(|g| g.delta),
            })
            .collect(),
        non_verifie: p.rapport.non_verifie.clone(),
    })
}

/// Boîte de dialogue du système, puis rapport de cohérence. `Ok(None)` : choix annulé.
#[tauri::command]
pub fn choisir_fiche_pdf() -> Result<Option<ApercuImport>, String> {
    let Some(chemin) = rfd::FileDialog::new()
        .set_title("Choisir une fiche de personnage (PDF)")
        .add_filter("Fiche PDF", &["pdf"])
        .pick_file()
    else {
        return Ok(None);
    };
    apercu(&chemin.to_string_lossy()).map(Some)
}

/// L'événement unique (groupé) qui crée le PJ, sa fiche et les XP de son niveau.
fn evenement_import(
    chemin: &str,
    gardes: &[String],
    po: i64,
    id: IdPersonnage,
    vitesse: Vitesse,
) -> Result<(Evenement, String), String> {
    let p = proposition(chemin)?;
    let fiche = appliquer_gardes(p.fiche, &p.rapport, gardes);
    fiche
        .calculer(fiches::Referentiel::partage())
        .map_err(|e| format!("La fiche est invalide : {e}"))?;
    let nom = fiche.nom.trim().to_string();
    // Les XP (ou le jalon) du niveau de la fiche, pour qu'aucune montée ne reste « en attente ».
    let niveau_atteint = if vitesse == Vitesse::Jalons {
        EvenementFiche::JalonAccorde { niveau: p.niveau }
    } else {
        let nombre = xp_requis(fiches::Referentiel::partage(), vitesse, p.niveau)
            .ok_or("Niveau inconnu du tableau de progression.")?;
        EvenementFiche::XpGagnes {
            nombre,
            motif: format!("Importé au niveau {}", p.niveau),
        }
    };
    let message = format!("Fiche de « {nom} » importée (niveau {})", p.niveau);
    Ok((
        Evenement::Groupe {
            libelle: message.clone(),
            evenements: vec![
                Evenement::PersonnageCree {
                    id,
                    nom,
                    compte_initial: Montant::po(po.max(0)),
                    prestige: None,
                },
                Evenement::Fiche {
                    pj: id,
                    evenement: EvenementFiche::Creation(Box::new(fiche)),
                },
                Evenement::Fiche {
                    pj: id,
                    evenement: niveau_atteint,
                },
            ],
        },
        message,
    ))
}

/// Crée le PJ et sa fiche à partir du PDF. `gardes` : clés des lignes dont la valeur de la
/// feuille est gardée (elles deviennent des effets « personnalisés ») ; toutes les autres
/// prennent la valeur des règles.
#[tauri::command]
pub fn importer_fiche_pdf(s: S, chemin: String, gardes: Vec<String>, po: i64) -> Retour {
    let mut x = session(&s);
    let Some((id, vitesse)) = x.avec_etat(|e| {
        (
            IdPersonnage(e.personnages.keys().map(|k| k.0).max().unwrap_or(0) + 1),
            e.regles_personnages.fiche().vitesse,
        )
    }) else {
        return Retour::erreur("Aucune campagne ouverte");
    };
    match evenement_import(&chemin, &gardes, po, id, vitesse) {
        Ok((ev, message)) => x.executer(ev, &message),
        Err(m) => Retour::erreur(m),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use campagne::Campagne;

    /// Essai de bout en bout sur une vraie fiche : `FICHE_PDF_ESSAI=chemin.pdf cargo test`.
    /// Ignoré si la variable n'est pas définie (la fiche d'un joueur n'est pas dans le dépôt).
    #[test]
    fn import_d_une_fiche_reelle_de_bout_en_bout() {
        let Ok(chemin) = std::env::var("FICHE_PDF_ESSAI") else {
            return;
        };
        let r = regles::referentiel_officiel();
        let mut c = Campagne::nouvelle("Essai");
        let id = c.nouvel_id_personnage();
        let vitesse = c.etat(&r).regles_personnages.fiche().vitesse;
        let a = apercu(&chemin).expect("rapport");
        let cle_vigueur = a
            .lignes
            .iter()
            .find(|l| l.champ == "JdS2" && l.peut_garder)
            .map(|l| l.cle.clone());
        let (ev, _) =
            evenement_import(&chemin, cle_vigueur.as_slice(), 100, id, vitesse).expect("import");
        c.executer(ev, &r).expect("exécution");
        let e = c.etat(&r).fiche_de(id).expect("fiche");
        assert_eq!(e.fiche.niveau_de_personnage(), a.niveau);
        assert_eq!(e.niveau_atteignable, a.niveau, "aucune montée en attente");
        let v = e.fiche.calculer(fiches::Referentiel::partage()).unwrap();
        assert_eq!(v.pv_max.total, a.pv);
        if cle_vigueur.is_some() {
            assert!(v.vigueur.explication().contains("personnalisé"));
        }
        // Annuler défait tout d'un coup.
        c.annuler().unwrap();
        assert!(c.etat(&r).fiche_de(id).is_none());
    }
}
