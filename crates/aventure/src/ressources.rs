//! Utilisations quotidiennes des pouvoirs de lignage (`data/combat/ressources_lignage.json`).
//! Les maxima sont calculés d'après le niveau d'ensorceleur et le modificateur de Charisme que la
//! fiche fournit ; le moteur de personnage n'est pas modifié.

use crate::objets::{Ressource, UniteRessource};
use serde::Deserialize;
use std::sync::OnceLock;

const JSON: &str = include_str!("../../../data/combat/ressources_lignage.json");

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "genre", rename_all = "snake_case")]
pub enum Formule {
    TroisPlusCha,
    Fixe {
        n: u32,
    },
    Niveau,
    Paliers {
        paliers: Vec<(u32, u32)>,
    },
    #[serde(rename = "un_par_4_niveaux")]
    UnPar4Niveaux {
        debut: u32,
        plafond: u32,
    },
}

#[derive(Debug, Clone, Deserialize)]
pub struct RessourceLignage {
    pub lignage: String,
    pub pouvoir: String,
    pub niveau_classe: u32,
    pub unite: UniteRessource,
    pub formule: Formule,
}

#[derive(Deserialize)]
struct Fichier {
    ressources: Vec<RessourceLignage>,
}

pub fn donnees() -> &'static [RessourceLignage] {
    static D: OnceLock<Vec<RessourceLignage>> = OnceLock::new();
    D.get_or_init(|| {
        serde_json::from_str::<Fichier>(JSON)
            .expect("data/combat/ressources_lignage.json est valide")
            .ressources
    })
}

impl Formule {
    /// Maximum quotidien pour un niveau d'ensorceleur et un modificateur de Charisme.
    pub fn maximum(&self, niveau: u32, mod_cha: i32) -> u32 {
        match self {
            Formule::TroisPlusCha => (3 + mod_cha).max(0) as u32,
            Formule::Fixe { n } => *n,
            Formule::Niveau => niveau,
            Formule::Paliers { paliers } => paliers
                .iter()
                .filter(|(n, _)| *n <= niveau)
                .map(|(_, v)| *v)
                .max()
                .unwrap_or(0),
            Formule::UnPar4Niveaux { debut, plafond } => {
                if niveau < *debut {
                    0
                } else {
                    (1 + (niveau - debut) / 4).min(*plafond)
                }
            }
        }
    }
}

/// Ressources d'un lignage à ce niveau d'ensorceleur : (nom anglais du pouvoir, ressource neuve).
pub fn ressources_de_lignage(lignage: &str, niveau: u32, mod_cha: i32) -> Vec<(String, Ressource)> {
    donnees()
        .iter()
        .filter(|r| r.lignage == lignage && r.niveau_classe <= niveau)
        .filter_map(|r| {
            let max = r.formule.maximum(niveau, mod_cha);
            (max > 0).then(|| {
                (
                    r.pouvoir.clone(),
                    Ressource {
                        unite: r.unite,
                        max,
                        utilisees: 0,
                        source: format!(
                            "lignage {}, {} (niveau {})",
                            r.lignage, r.pouvoir, r.niveau_classe
                        ),
                    },
                )
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use fiches::Referentiel;

    #[test]
    fn les_formules_sont_celles_du_texte_du_livre() {
        let r = Referentiel::core();
        let classe = r.classe("ensorceleur").unwrap();
        for d in donnees() {
            let l = classe
                .lignages
                .iter()
                .find(|l| l.id == d.lignage)
                .unwrap_or_else(|| panic!("lignage absent : {}", d.lignage));
            let p = l
                .pouvoirs
                .iter()
                .find(|p| p.nom_en == d.pouvoir)
                .unwrap_or_else(|| panic!("pouvoir absent : {} / {}", d.lignage, d.pouvoir));
            assert_eq!(
                p.niveau_classe,
                Some(d.niveau_classe),
                "niveau de {}",
                d.pouvoir
            );
            let t = &p.texte_en.to_lowercase();
            let ok = match &d.formule {
                Formule::TroisPlusCha => t.contains("3 + your charisma modifier"),
                Formule::Fixe { .. } => t.contains("once per day"),
                Formule::Niveau => t.contains("equal to your sorcerer level"),
                Formule::Paliers { paliers } => {
                    t.contains("once per day")
                        && (paliers.len() < 2
                            || t.contains("17th level, you can use this ability twice per day"))
                        && (paliers.len() < 3
                            || t.contains(
                                "20th level, you can use this ability three times per day",
                            ))
                        && (paliers.len() > 2 || !t.contains("three times per day"))
                }
                Formule::UnPar4Niveaux { .. } => {
                    t.contains("once per day at 3rd level")
                        && t.contains("up to five times per day at 19th level")
                }
            };
            assert!(
                ok,
                "formule de {} / {} absente du texte : {t}",
                d.lignage, d.pouvoir
            );
        }
    }

    #[test]
    fn maxima_par_niveau() {
        let f = |l: &str, n, c| {
            ressources_de_lignage(l, n, c)
                .into_iter()
                .map(|(nom, r)| (nom, r.max))
                .collect::<Vec<_>>()
        };
        assert_eq!(f("aberrant", 1, 3), vec![("Acidic Ray".to_string(), 6)]);
        assert_eq!(f("aberrant", 1, -1), vec![("Acidic Ray".to_string(), 2)]);
        assert_eq!(f("draconic", 8, 2), vec![("Claws".to_string(), 5)]);
        let d = f("draconic", 17, 0);
        assert_eq!(
            d,
            vec![("Claws".to_string(), 3), ("Breath Weapon".to_string(), 2)]
        );
        assert_eq!(f("draconic", 20, 0)[1].1, 3);
        let a = f("arcane", 19, 0);
        assert_eq!(a[1], ("Metamagic Adept".to_string(), 5));
        assert_eq!(f("arcane", 3, 0)[1].1, 1);
        assert_eq!(f("arcane", 7, 0)[1].1, 2);
        assert_eq!(f("fey", 9, 0)[1], ("Fleeting Glance".to_string(), 9));
        assert_eq!(f("celestial", 9, 0)[1], ("Wings of Heaven".to_string(), 9));
        // un modificateur de Charisme très négatif ne donne pas de ressource négative
        assert!(f("aberrant", 1, -4).is_empty());
    }
}
