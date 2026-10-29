//! Catalogues du Core Rulebook : dons et sorts (`data/core/dons.json`, `data/core/sorts.json`).

use crate::donnees::SourceLivre;
use serde::Deserialize;
use std::collections::BTreeMap;

/// Un prérequis de don. Les formes non reconnues à l'import restent en `Texte` : le moteur les
/// signale « à vérifier » au lieu de les deviner.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(tag = "genre", rename_all = "snake_case")]
pub enum Prerequis {
    Caracteristique {
        carac: String,
        min: i32,
    },
    Bba {
        min: i32,
    },
    NiveauPersonnage {
        min: u32,
    },
    NiveauLanceur {
        min: u32,
    },
    Rangs {
        competence: String,
        sous_type: Option<String>,
        min: u32,
    },
    SortsDeNiveau {
        niveau_de_sort: u32,
        #[serde(rename = "type")]
        genre: Option<String>,
    },
    NiveauDeClasse {
        classe: String,
        min: u32,
    },
    Don {
        nom_en: String,
    },
    /// L'un au moins des termes.
    Ou {
        termes: Vec<Prerequis>,
    },
    Texte {
        texte_en: String,
    },
}

#[derive(Debug, Clone, Deserialize)]
pub struct Don {
    pub nom_en: String,
    pub types_en: Vec<String>,
    pub source: SourceLivre,
    pub resume_en: String,
    pub prerequis_en: Option<String>,
    pub prerequis: Vec<Prerequis>,
    pub avantage_en: Option<String>,
    pub normal_en: Option<String>,
    pub special_en: Option<String>,
}

impl Don {
    /// « You can gain this feat multiple times » : le don peut être repris.
    pub fn repetable(&self) -> bool {
        self.special_en
            .as_deref()
            .is_some_and(|s| s.contains("multiple times"))
    }

    /// Classes (clés anglaises en minuscules, ex. « fighter ») dont le don exige un niveau.
    /// Vide quand le don n'exige aucune classe : il est alors disponible pour toutes les classes.
    pub fn classes_requises(&self) -> Vec<&str> {
        self.prerequis
            .iter()
            .filter_map(|p| match p {
                Prerequis::NiveauDeClasse { classe, .. } => Some(classe.as_str()),
                _ => None,
            })
            .collect()
    }

    /// Le don est-il ouvert à cette classe (clé anglaise minuscule) ?
    /// Sans classe exigée, oui pour toutes les classes.
    pub fn ouvert_a_la_classe(&self, classe: &str) -> bool {
        let requises = self.classes_requises();
        requises.is_empty() || requises.iter().any(|c| c.eq_ignore_ascii_case(classe))
    }

    /// Le don porte-t-il un choix (arme, compétence, école…) ? Détecté d'après le texte du livre.
    pub fn a_un_choix(&self) -> bool {
        let t = format!(
            "{} {}",
            self.resume_en,
            self.avantage_en.clone().unwrap_or_default()
        );
        t.contains("Choose ") || t.contains("choose one") || t.contains("selected ")
    }
}

/// Nom d'un don sans son choix : « Weapon Focus (longsword) » → « Weapon Focus ».
pub fn nom_de_base(don: &str) -> &str {
    don.split(" (").next().unwrap_or(don).trim()
}

#[derive(Debug, Clone, Deserialize)]
pub struct Sort {
    pub nom_en: String,
    pub source: SourceLivre,
    pub ecole: String,
    pub sous_ecole: Option<String>,
    pub descripteurs: Vec<String>,
    /// Niveau de sort par classe (clés anglaises : `sorcerer`, `wizard`, `cleric`…).
    pub niveaux: BTreeMap<String, u32>,
    pub temps_d_incantation_en: Option<String>,
    pub composantes_en: Option<String>,
    pub portee_en: Option<String>,
    pub cible_en: Option<String>,
    pub cible_type: Option<String>,
    pub duree_en: Option<String>,
    pub jet_de_sauvegarde_en: Option<String>,
    pub resistance_a_la_magie_en: Option<String>,
    pub description_en: String,
}

impl Sort {
    pub fn niveau_pour(&self, classe_en: &str) -> Option<u32> {
        self.niveaux.get(classe_en).copied()
    }
}

#[derive(Debug, Deserialize)]
pub(crate) struct FichierDons {
    pub dons: Vec<Don>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct FichierSorts {
    pub sorts: Vec<Sort>,
}

#[cfg(test)]
mod tests {
    use crate::donnees::Referentiel;

    #[test]
    fn catalogue_de_sorts_du_core() {
        let r = Referentiel::core();
        assert_eq!(r.sorts.len(), 623);
        let de_l_ensorceleur = r
            .sorts
            .iter()
            .filter(|s| s.niveau_pour("sorcerer").is_some())
            .count();
        assert_eq!(de_l_ensorceleur, 390);
        let niveau = |n: &str| r.sort(n).unwrap().niveau_pour("sorcerer");
        assert_eq!(niveau("Light"), Some(0));
        assert_eq!(niveau("Magic Missile"), Some(1));
        assert_eq!(niveau("Fireball"), Some(3));
        assert_eq!(niveau("Wish"), Some(9));
        // tous ont une source du Core, une école et une description
        for s in &r.sorts {
            assert_eq!(s.source.livre, "PRPG Core Rulebook", "{}", s.nom_en);
            assert!(
                !s.ecole.is_empty() && !s.description_en.is_empty(),
                "{}",
                s.nom_en
            );
        }
    }

    #[test]
    fn don_sans_classe_ouvert_a_toutes_les_classes() {
        let r = Referentiel::core();
        let acrobatique = r.don("Acrobatic").unwrap();
        assert!(acrobatique.classes_requises().is_empty());
        assert!(acrobatique.ouvert_a_la_classe("fighter"));
        assert!(acrobatique.ouvert_a_la_classe("wizard"));
    }

    #[test]
    fn don_de_classe_reserve_a_sa_classe() {
        let r = Referentiel::core();
        let maitrise = r.don("Spell Mastery").unwrap();
        assert_eq!(maitrise.classes_requises(), vec!["wizard"]);
        assert!(maitrise.ouvert_a_la_classe("wizard"));
        assert!(!maitrise.ouvert_a_la_classe("fighter"));
    }

    #[test]
    fn sorts_de_lignage_presents_dans_le_catalogue() {
        let r = Referentiel::core();
        let classe = r.classe("ensorceleur").unwrap();
        let mut absents = Vec::new();
        for l in &classe.lignages {
            for s in &l.sorts_bonus {
                let nom = s
                    .nom_en
                    .trim_end_matches('*')
                    .split(" (")
                    .next()
                    .unwrap_or("")
                    .to_string();
                if r.sort_approx(&nom).is_none() {
                    absents.push(format!("{} : {}", l.id, s.nom_en));
                }
            }
        }
        assert!(absents.is_empty(), "sorts de lignage absents : {absents:?}");
    }
}
