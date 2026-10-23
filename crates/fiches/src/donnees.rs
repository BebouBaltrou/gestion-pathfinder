//! Référentiels du Core Rulebook (races, compétences, classes), chargés depuis `data/core/*.json`.
//! Les fichiers sont produits par `outils/importer_aon.js` et embarqués dans l'application.

use crate::effet::{Effet, TypeBonus};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct SourceLivre {
    pub livre: String,
    pub page: Option<u32>,
    pub url: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EffetDonnee {
    pub cible: String,
    #[serde(rename = "type")]
    pub type_bonus: TypeBonus,
    pub valeur: i32,
    pub source: String,
}

impl EffetDonnee {
    pub fn vers_effet(&self) -> Effet {
        Effet::new(&self.cible, self.type_bonus, self.valeur, &self.source)
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct TraitTexte {
    pub nom_en: String,
    pub texte_en: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Race {
    pub id: String,
    pub nom: String,
    pub nom_en: String,
    pub taille: String,
    pub vitesse: u32,
    #[serde(default)]
    pub vitesse_jamais_reduite: bool,
    pub vision: String,
    /// Ajustements fixes : identifiant de caractéristique (`for`, `dex`…) et valeur.
    pub ajustements: Vec<(String, i32)>,
    /// Bonus à placer au choix du joueur (humain, demi-elfe, demi-orque).
    pub choix_libre: Option<i32>,
    pub langues: Vec<String>,
    pub langues_choix: String,
    #[serde(default)]
    pub bonus_don: u32,
    #[serde(default)]
    pub rang_supplementaire: u32,
    #[serde(default)]
    pub deux_classes_de_predilection: bool,
    pub effets: Vec<EffetDonnee>,
    pub source: SourceLivre,
    pub traits: Vec<TraitTexte>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Competence {
    pub id: String,
    pub nom_en: String,
    pub caracteristique: String,
    pub formation_requise: bool,
    pub malus_d_armure: bool,
    pub sous_types: bool,
    pub source: SourceLivre,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ProgressionNiveau {
    pub niveau: u32,
    /// Bonus de base à l'attaque (plusieurs valeurs : attaques itératives).
    pub bba: Vec<i32>,
    pub vigueur: i32,
    pub reflexes: i32,
    pub volonte: i32,
    pub special_en: String,
    /// Sorts par jour de niveau 1 à 9 (`None` : aucun de ce niveau).
    pub sorts_par_jour: Vec<Option<u32>>,
    /// Sorts connus de niveau 0 à 9.
    pub sorts_connus: Vec<Option<u32>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SortLignage {
    pub nom_en: String,
    pub niveau_classe: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Pouvoir {
    pub nom_en: String,
    #[serde(rename = "type")]
    pub genre: String,
    pub niveau_classe: Option<u32>,
    pub texte_en: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Lignage {
    pub id: String,
    pub nom_en: String,
    pub source: SourceLivre,
    pub competence_de_classe_en: String,
    pub sorts_bonus: Vec<SortLignage>,
    pub dons_bonus_en: Vec<String>,
    pub arcane_en: String,
    pub pouvoirs: Vec<Pouvoir>,
    pub choix_en: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Incantation {
    #[serde(rename = "type")]
    pub genre: String,
    pub mode: String,
    pub caracteristique: String,
    /// Échange d’un sort connu : à partir de ce niveau de classe, puis tous les `echange_pas` niveaux.
    pub echange_niveau_min: Option<u32>,
    pub echange_pas: Option<u32>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Classe {
    pub id: String,
    pub nom: String,
    pub nom_en: String,
    pub source: SourceLivre,
    pub de_vie: u32,
    pub rangs_par_niveau: i32,
    pub caracteristique_rangs: String,
    pub competences_de_classe_en: Vec<String>,
    pub incantation: Option<Incantation>,
    pub progression: Vec<ProgressionNiveau>,
    #[serde(default)]
    pub lignages: Vec<Lignage>,
}

/// Ligne du tableau 3-1 (Core p. 31) : XP requis et gains du niveau.
#[derive(Debug, Clone, Deserialize)]
pub struct LigneProgression {
    pub niveau: u32,
    pub xp_lente: u32,
    pub xp_moyenne: u32,
    pub xp_rapide: u32,
    pub don: bool,
    pub augmentation: bool,
}

#[derive(Debug, Deserialize)]
struct FichierProgression {
    niveaux: Vec<LigneProgression>,
}

#[derive(Debug, Deserialize)]
struct FichierRaces {
    races: Vec<Race>,
}
#[derive(Debug, Deserialize)]
struct FichierCompetences {
    competences: Vec<Competence>,
}
#[derive(Debug, Deserialize)]
struct FichierClasse {
    classe: Classe,
}

/// Tout le référentiel Core chargé.
#[derive(Debug, Clone)]
pub struct Referentiel {
    pub races: Vec<Race>,
    pub competences: Vec<Competence>,
    pub classes: Vec<Classe>,
    pub progression: Vec<LigneProgression>,
}

impl Referentiel {
    /// Référentiel Core embarqué dans l'application.
    pub fn core() -> Referentiel {
        Self::depuis_json(
            include_str!("../../../data/core/races.json"),
            include_str!("../../../data/core/competences.json"),
            &[include_str!("../../../data/core/ensorceleur.json")],
            include_str!("../../../data/core/progression.json"),
        )
        .expect("référentiel Core embarqué valide")
    }

    pub fn depuis_json(
        races: &str,
        competences: &str,
        classes: &[&str],
        progression: &str,
    ) -> Result<Referentiel, serde_json::Error> {
        let races: FichierRaces = serde_json::from_str(races)?;
        let competences: FichierCompetences = serde_json::from_str(competences)?;
        let mut cl = Vec::new();
        for c in classes {
            let f: FichierClasse = serde_json::from_str(c)?;
            cl.push(f.classe);
        }
        let progression: FichierProgression = serde_json::from_str(progression)?;
        Ok(Referentiel {
            progression: progression.niveaux,
            races: races.races,
            competences: competences.competences,
            classes: cl,
        })
    }

    /// Ligne du tableau de progression pour un niveau de personnage (1 à 20).
    pub fn ligne_progression(&self, niveau: u32) -> Option<&LigneProgression> {
        self.progression.iter().find(|l| l.niveau == niveau)
    }

    pub fn race(&self, id: &str) -> Option<&Race> {
        self.races.iter().find(|r| r.id == id)
    }
    pub fn classe(&self, id: &str) -> Option<&Classe> {
        self.classes.iter().find(|c| c.id == id)
    }
    pub fn competence(&self, id: &str) -> Option<&Competence> {
        self.competences.iter().find(|c| c.id == id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn le_referentiel_embarque_se_charge() {
        let r = Referentiel::core();
        assert_eq!(r.races.len(), 7);
        assert_eq!(r.competences.len(), 26);
        let e = r.classe("ensorceleur").unwrap();
        assert_eq!(e.progression.len(), 20);
        assert_eq!(e.lignages.len(), 10);
        for l in &e.lignages {
            assert_eq!(l.sorts_bonus.len(), 9, "{}", l.id);
        }
    }

    #[test]
    fn chaque_donnee_a_sa_source() {
        let r = Referentiel::core();
        assert!(r.races.iter().all(|x| x.source.page.is_some()));
        assert!(r.competences.iter().all(|x| x.source.page.is_some()));
        let e = r.classe("ensorceleur").unwrap();
        assert!(e.lignages.iter().all(|l| l.source.page.is_some()));
    }
}
