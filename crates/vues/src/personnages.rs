//! Page Personnages : création, compte personnel, déplacements, Prestige.

use crate::localites::Choix;
use crate::Somme;
use campagne::evenements::Evenement;
use campagne::modele::{IdLocalite, IdPersonnage};
use domaine::Montant;
use regles::Referentiel;

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct PagePersonnages {
    pub personnages: Vec<FichePersonnage>,
    /// Localités où un PJ peut se trouver.
    pub localites: Vec<Choix>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct FichePersonnage {
    pub id: u32,
    pub nom: String,
    pub compte: Somme,
    /// Localité où se trouve le PJ (`None` : hors localité).
    pub localite: Option<u32>,
    pub lieu: String,
    pub prestige: Option<i32>,
}

pub fn page_personnages(c: &mut campagne::Campagne, r: &Referentiel) -> PagePersonnages {
    let etat = c.etat(r).clone();
    PagePersonnages {
        personnages: etat
            .personnages
            .values()
            .map(|p| FichePersonnage {
                id: p.id.0,
                nom: p.nom.clone(),
                compte: p.compte.into(),
                localite: p.localite.map(|l| l.0),
                lieu: p
                    .localite
                    .and_then(|id| etat.localites.get(&id))
                    .map(|l| l.nom.clone())
                    .unwrap_or_else(|| "hors localité".into()),
                prestige: p.prestige,
            })
            .collect(),
        localites: etat
            .localites
            .values()
            .map(|l| Choix {
                id: l.id.0,
                libelle: l.nom.clone(),
            })
            .collect(),
    }
}

fn lire_prestige(s: &str) -> Result<Option<i32>, String> {
    let t = s.trim();
    if t.is_empty() {
        return Ok(None);
    }
    t.parse()
        .map(Some)
        .map_err(|_| "Le Prestige doit être un nombre entier".to_string())
}

fn lire_nom(s: &str) -> Result<String, String> {
    let n = s.trim();
    if n.is_empty() {
        Err("Donnez un nom au personnage".into())
    } else {
        Ok(n.into())
    }
}

/// Nouveau personnage ; compte initial vide = 0 po.
pub fn creation(
    id: IdPersonnage,
    nom: &str,
    compte: &str,
    prestige: &str,
) -> Result<Evenement, String> {
    let compte_initial = if compte.trim().is_empty() {
        Montant::ZERO
    } else {
        Montant::depuis_texte(compte)
            .ok_or("Montant illisible : écrivez par exemple 12, 12,5 ou 4 po 7 pa")?
    };
    Ok(Evenement::PersonnageCree {
        id,
        nom: lire_nom(nom)?,
        compte_initial,
        prestige: lire_prestige(prestige)?,
    })
}

pub fn modification(id: u32, nom: &str, prestige: &str) -> Result<Evenement, String> {
    Ok(Evenement::PersonnageModifie {
        id: IdPersonnage(id),
        nom: lire_nom(nom)?,
        prestige: lire_prestige(prestige)?,
    })
}

pub fn deplacement(id: u32, vers: Option<u32>) -> Evenement {
    Evenement::PersonnageDeplace {
        id: IdPersonnage(id),
        vers: vers.map(IdLocalite),
    }
}

/// Gain ou dépense hors de l'application (montant négatif pour une dépense).
pub fn ajustement_compte(pj: u32, delta: &str, motif: &str) -> Result<Evenement, String> {
    let delta = Montant::depuis_texte(delta)
        .ok_or("Montant illisible : écrivez par exemple -25, 12,5 ou 4 po 7 pa")?;
    if delta.pa == 0 {
        return Err("Le montant ne doit pas être nul".into());
    }
    if motif.trim().is_empty() {
        return Err("Indiquez le motif (butin, achat…)".into());
    }
    Ok(Evenement::AjustementCompte {
        pj: IdPersonnage(pj),
        delta,
        motif: motif.trim().into(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use campagne::Campagne;

    #[test]
    fn creation_deplacement_et_compte() {
        let r = regles::referentiel_officiel();
        let mut c = Campagne::nouvelle("Test");
        let l = c.nouvel_id_localite();
        let s = crate::localites::SaisieLocalite {
            nom: "Sandpoint".into(),
            officielle: false,
            categorie_perso_limite: "3".into(),
            ..Default::default()
        };
        c.executer(s.creation(l, &r).unwrap(), &r).unwrap();
        let id = c.nouvel_id_personnage();
        c.executer(creation(id, "Seoni", "", "4").unwrap(), &r)
            .unwrap();
        c.executer(deplacement(id.0, Some(l.0)), &r).unwrap();
        c.executer(ajustement_compte(id.0, "-2,5", "achat").unwrap(), &r)
            .unwrap();
        c.executer(modification(id.0, "Seoni la mage", "").unwrap(), &r)
            .unwrap();
        let p = page_personnages(&mut c, &r);
        let f = &p.personnages[0];
        assert_eq!(f.nom, "Seoni la mage");
        assert_eq!(f.compte.pa, -25);
        assert_eq!(f.lieu, "Sandpoint");
        assert_eq!(f.prestige, None);
        assert_eq!(p.localites.len(), 1);
    }

    #[test]
    fn saisies_refusees() {
        let id = IdPersonnage(1);
        assert!(creation(id, " ", "", "").is_err());
        assert!(creation(id, "A", "beaucoup", "").is_err());
        assert!(creation(id, "A", "", "x").is_err());
        assert!(ajustement_compte(1, "0", "x").is_err());
        assert!(ajustement_compte(1, "5", "").is_err());
    }
}
