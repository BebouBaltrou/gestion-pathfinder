//! États du Core Rulebook (p. 565, `data/core/etats.json`) et effets temporaires.
//!
//! Chaque état donne des effets typés quand le texte du livre les chiffre sans ambiguïté
//! (ex. « secoué » : –2 aux attaques, jets de sauvegarde et compétences). Le reste (jets
//! conditionnels, pertes d'actions…) est gardé en texte et en note, jamais deviné.

use crate::donnees::SourceLivre;
use crate::effet::{Effet, TypeBonus};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct EffetEtat {
    /// `attaque`, `degats`, `ca`, `init`, `carac.xxx`, `jds` (les trois jets), `comp` (toutes les compétences).
    pub cible: String,
    #[serde(rename = "type")]
    pub type_bonus: TypeBonus,
    pub valeur: i32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EtatDef {
    pub id: String,
    pub nom_en: String,
    /// Nom français (traduction non officielle).
    #[serde(default)]
    pub nom: String,
    pub texte_en: String,
    /// Texte français (traduction non officielle).
    #[serde(default)]
    pub texte: String,
    pub effets: Vec<EffetEtat>,
    pub perd_bonus_dex_ca: bool,
    pub vitesse_moitie: bool,
    /// Ce que le texte prévoit mais que le moteur ne calcule pas.
    pub note: Option<String>,
    pub source: SourceLivre,
}

#[derive(Debug, Deserialize)]
pub(crate) struct FichierEtats {
    pub etats: Vec<EtatDef>,
}

impl EtatDef {
    /// Effets chiffrés de l'état, cibles génériques développées (`jds` → les trois jets).
    pub fn effets_chiffres(&self) -> Vec<Effet> {
        let source = format!("état « {} »", self.nom);
        let mut v = Vec::new();
        for e in &self.effets {
            let cibles: Vec<&str> = match e.cible.as_str() {
                "jds" => vec!["jds.vig", "jds.ref", "jds.vol"],
                "comp" => vec!["comp.*"],
                autre => vec![autre],
            };
            for c in cibles {
                v.push(Effet::new(c, e.type_bonus, e.valeur, &source));
            }
        }
        v
    }
}

/// Un effet qui dure (sort lancé sur soi, potion…) : ses effets typés et le temps qu'il reste.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EffetTemporaire {
    pub nom: String,
    pub effets: Vec<Effet>,
    /// `None` : dure jusqu'à ce qu'on le retire.
    pub rounds_restants: Option<u32>,
}

#[cfg(test)]
mod tests {
    use crate::donnees::Referentiel;

    #[test]
    fn les_34_etats_du_core_sont_la() {
        let r = Referentiel::core();
        assert_eq!(r.etats.len(), 34);
        for e in &r.etats {
            assert!(!e.nom.is_empty(), "état sans nom français : {}", e.id);
            assert!(!e.texte.is_empty(), "état sans texte français : {}", e.id);
            assert_eq!(e.source.page, Some(565));
        }
        let secoue = r.etat("shaken").unwrap();
        assert_eq!(secoue.effets_chiffres().len(), 5);
    }
}
