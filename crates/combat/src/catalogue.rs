//! Catalogue des actions de combat (`data/combat/actions.json`, Core p. 181-186).
//! Ajouter une action = ajouter une ligne de données.

use crate::economie::TypeAction;

const JSON: &str = include_str!("../../../data/combat/actions.json");

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct ActionDef {
    pub id: String,
    pub nom_en: String,
    /// Nom français (traduction non officielle).
    pub nom: String,
    #[serde(rename = "type")]
    pub type_: String,
    pub deplacement: bool,
}

impl ActionDef {
    /// `None` pour les actions dont le type varie (manœuvre offensive, don) : le joueur le précise.
    pub fn type_action(&self) -> Option<TypeAction> {
        Some(match self.type_.as_str() {
            "simple" => TypeAction::Simple,
            "mouvement" => TypeAction::Mouvement,
            "complexe" => TypeAction::Complexe,
            "rapide" => TypeAction::Rapide,
            "immediate" => TypeAction::Immediate,
            "libre" => TypeAction::Libre,
            "aucune" => TypeAction::Aucune,
            _ => return None,
        })
    }
}

#[derive(serde::Deserialize)]
struct Fichier {
    actions: Vec<ActionDef>,
}

pub fn catalogue() -> Vec<ActionDef> {
    serde_json::from_str::<Fichier>(JSON)
        .expect("data/combat/actions.json est valide")
        .actions
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    #[test]
    fn le_catalogue_est_complet_et_coherent() {
        let c = catalogue();
        assert_eq!(c.len(), 58);
        let ids: BTreeSet<_> = c.iter().map(|a| &a.id).collect();
        assert_eq!(ids.len(), c.len(), "identifiants en double");
        for a in &c {
            assert!(!a.nom.is_empty() && !a.nom_en.is_empty());
            assert!(
                a.type_action().is_some() || a.type_ == "variable",
                "type inconnu pour {} : {}",
                a.id,
                a.type_
            );
        }
        let n = |t: &str| c.iter().filter(|a| a.type_ == t).count();
        assert_eq!((n("simple"), n("mouvement"), n("complexe")), (21, 13, 13));
        assert_eq!(
            (
                n("rapide"),
                n("immediate"),
                n("libre"),
                n("aucune"),
                n("variable")
            ),
            (1, 1, 5, 2, 2)
        );
        // les actions qui déplacent réellement
        let depl: Vec<_> = c
            .iter()
            .filter(|a| a.deplacement)
            .map(|a| a.id.as_str())
            .collect();
        assert_eq!(depl, vec!["move", "charge", "run", "withdraw"]);
    }
}
