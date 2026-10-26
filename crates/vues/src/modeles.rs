//! Page « Modèles personnels » : tous les bâtiments et organisations enregistrés comme modèles
//! (toutes campagnes confondues).

use campagne::modeles_perso::{ModelesPerso, PREFIXE};

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct PageModeles {
    pub batiments: Vec<ModelePerso>,
    pub organisations: Vec<ModelePerso>,
}

/// Un modèle personnel : `id` sert à la suppression.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct ModelePerso {
    pub id: String,
    pub nom: String,
    pub detail: String,
}

pub fn page_modeles(m: &ModelesPerso) -> PageModeles {
    PageModeles {
        batiments: m
            .batiments
            .iter()
            .map(|b| {
                let salles: usize = b.etages.iter().map(|e| e.salles.len()).sum();
                ModelePerso {
                    id: format!("{PREFIXE}{}", b.nom),
                    nom: b.nom.clone(),
                    detail: format!("{} étage(s), {} salle(s)", b.etages.len(), salles),
                }
            })
            .collect(),
        organisations: m
            .organisations
            .iter()
            .map(|o| ModelePerso {
                id: format!("{PREFIXE}{}", o.nom),
                nom: o.nom.clone(),
                detail: format!("{} équipe(s)", o.equipes.len()),
            })
            .collect(),
    }
}
