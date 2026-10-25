//! Divinités et domaines du prêtre (Manuel des joueurs, tables 3-14 et domaines p. 69 à 81).
//!
//! Les données sont des fichiers JSON (`data/core/domaines.json`, `data/core/divinites.json`) :
//! le moteur ne contient aucun nom de domaine ni de dieu.

use serde::Deserialize;

/// Combien de fois un pouvoir de domaine peut servir par jour, quand le livre le dit.
#[derive(Debug, Clone, Deserialize)]
pub struct UsagesPouvoir {
    /// `sagesse` (3 + modificateur de Sagesse), `niveau` (une fois au niveau `premier`, puis une de
    /// plus tous les `pas` niveaux) ou `rounds_par_niveau` (autant de rounds que le niveau de prêtre).
    #[serde(rename = "type")]
    pub genre: String,
    pub base: Option<i32>,
    pub premier: Option<u32>,
    pub pas: Option<u32>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PouvoirDomaine {
    pub nom: String,
    /// `Mag`, `Sur` ou `Ext`.
    pub genre: String,
    /// Niveau de prêtre à partir duquel le pouvoir est accordé.
    pub niveau: u32,
    #[serde(default)]
    pub usages: Option<UsagesPouvoir>,
    pub texte: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SortDomaine {
    /// Niveau de sort (1 à 9).
    pub niveau: u32,
    pub nom_fr: String,
    /// Nom du sort dans le catalogue, quand il a pu être rapproché.
    pub nom_en: Option<String>,
    pub note: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Domaine {
    pub id: String,
    pub nom: String,
    /// Dieux (noms) qui offrent ce domaine.
    pub divinites: Vec<String>,
    pub description: String,
    pub pouvoirs: Vec<PouvoirDomaine>,
    pub sorts_de_domaine: Vec<SortDomaine>,
    /// `Bien`, `Mal`, `Loi` ou `Chaos` : domaine réservé à un alignement correspondant.
    pub alignement: Option<String>,
    /// Compétences que le domaine rend « de classe » (Connaissance, Duperie, Faune).
    #[serde(default)]
    pub competences_de_classe_en: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Divinite {
    pub id: String,
    pub nom: String,
    /// `LB`, `NB`, `CB`, `LN`, `N`, `CN`, `LM`, `NM`, `CM`.
    pub alignement: String,
    pub theme: String,
    /// Identifiants des domaines.
    pub domaines: Vec<String>,
    pub arme_de_predilection: String,
    /// Nom de l'arme dans le catalogue (`None` : « mains nues » ou nom ambigu).
    pub arme_de_predilection_en: Option<String>,
    pub ecart_livre: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct FichierDomaines {
    pub domaines: Vec<Domaine>,
}

#[derive(Debug, Deserialize)]
pub struct FichierDivinites {
    pub divinites: Vec<Divinite>,
}

/// Axes d'un alignement : (loi/chaos, bien/mal), chacun codé `0` (Loi / Bien), `1` (Neutre) ou
/// `2` (Chaos / Mal). `LB` → (0, 0), `N` → (1, 1), `CM` → (2, 2).
pub fn axes(code: &str) -> Option<(i32, i32)> {
    if code == "N" {
        return Some((1, 1));
    }
    let mut c = code.chars();
    let (a, b) = (c.next()?, c.next()?);
    if c.next().is_some() {
        return None;
    }
    let loi = match a {
        'L' => 0,
        'N' => 1,
        'C' => 2,
        _ => return None,
    };
    let bien = match b {
        'B' => 0,
        'N' => 1,
        'M' => 2,
        _ => return None,
    };
    Some((loi, bien))
}

/// Alignement d'un prêtre : il peut différer d'au plus une catégorie de celui de son dieu, sur
/// un seul axe (Manuel des joueurs, prêtre, « Alignement »).
pub fn alignement_compatible(pretre: &str, dieu: &str) -> bool {
    match (axes(pretre), axes(dieu)) {
        (Some((a, b)), Some((c, d))) => (a - c).abs() + (b - d).abs() <= 1,
        _ => false,
    }
}

/// Un domaine d'alignement (Bien, Mal, Loi, Chaos) n'est permis qu'à un prêtre de cet alignement.
pub fn domaine_permis(domaine: &Domaine, alignement: &str) -> bool {
    let Some(req) = &domaine.alignement else {
        return true;
    };
    let Some((loi, bien)) = axes(alignement) else {
        return false;
    };
    match req.as_str() {
        "Loi" => loi == 0,
        "Chaos" => loi == 2,
        "Bien" => bien == 0,
        "Mal" => bien == 2,
        _ => true,
    }
}

impl PouvoirDomaine {
    /// Utilisations par jour au niveau de classe `nc` (`None` : pas de limite chiffrée dans le livre).
    pub fn utilisations(&self, nc: u32, mod_sagesse: i32) -> Option<i32> {
        let u = self.usages.as_ref()?;
        match u.genre.as_str() {
            "sagesse" => Some((u.base.unwrap_or(3) + mod_sagesse).max(0)),
            "niveau" => {
                let (premier, pas) = (u.premier?, u.pas?.max(1));
                (nc >= premier).then(|| 1 + ((nc - premier) / pas) as i32)
            }
            "rounds_par_niveau" => Some(nc as i32),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Referentiel;

    #[test]
    fn alignements() {
        assert_eq!(axes("LB"), Some((0, 0)));
        assert_eq!(axes("N"), Some((1, 1)));
        assert_eq!(axes("CM"), Some((2, 2)));
        assert_eq!(axes("XX"), None);
        // Prêtre d'un dieu Loyal Bon : NB ou LN permis ; NN, CB ou LM non.
        assert!(alignement_compatible("LB", "LB"));
        assert!(alignement_compatible("NB", "LB"));
        assert!(alignement_compatible("LN", "LB"));
        assert!(!alignement_compatible("N", "LB"));
        assert!(!alignement_compatible("CB", "LB"));
    }

    #[test]
    fn le_catalogue_des_domaines_et_des_dieux_vient_du_livre() {
        let r = Referentiel::partage();
        assert_eq!(r.domaines.len(), 33);
        assert_eq!(r.divinites.len(), 20);
        for d in &r.domaines {
            assert_eq!(d.pouvoirs.len(), 2, "{}", d.nom);
            assert_eq!(d.sorts_de_domaine.len(), 9, "{}", d.nom);
            let niv: Vec<u32> = d.sorts_de_domaine.iter().map(|s| s.niveau).collect();
            assert_eq!(niv, (1..=9).collect::<Vec<_>>(), "{}", d.nom);
        }
        // Chaque dieu offre cinq domaines existants ; chaque domaine cité par un dieu le liste aussi.
        for dieu in &r.divinites {
            assert_eq!(dieu.domaines.len(), 5, "{}", dieu.nom);
            for id in &dieu.domaines {
                let d = r.domaine(id).unwrap_or_else(|| panic!("domaine {id}"));
                assert!(d.divinites.contains(&dieu.nom), "{} / {}", dieu.nom, d.nom);
            }
        }
        // Les sorts de domaine rapprochés existent dans le catalogue des sorts.
        for d in &r.domaines {
            for s in d
                .sorts_de_domaine
                .iter()
                .filter_map(|s| s.nom_en.as_deref())
            {
                assert!(r.sort(s).is_some(), "{} : {s}", d.nom);
            }
        }
    }

    #[test]
    fn domaines_d_alignement_et_usages() {
        let r = Referentiel::partage();
        let bien = r.domaine("bien").unwrap();
        assert!(domaine_permis(bien, "NB"));
        assert!(!domaine_permis(bien, "LM"));
        assert!(domaine_permis(r.domaine("feu").unwrap(), "CM"));
        // Toucher du Bien : 3 + modificateur de Sagesse ; Lance du Bien : une fois au niveau 8, puis tous les 4 niveaux.
        let [toucher, lance] = &bien.pouvoirs[..] else {
            panic!()
        };
        assert_eq!(toucher.utilisations(5, 4), Some(7));
        assert_eq!(lance.utilisations(7, 4), None);
        assert_eq!(lance.utilisations(8, 4), Some(1));
        assert_eq!(lance.utilisations(12, 4), Some(2));
        assert_eq!(lance.utilisations(20, 4), Some(4));
    }
}
