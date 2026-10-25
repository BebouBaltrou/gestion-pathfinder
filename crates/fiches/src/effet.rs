//! Effets typés : `{ cible, type de bonus, valeur, source }`.

/// Types de bonus du Core Rulebook.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
pub enum TypeBonus {
    Alteration,
    Armure,
    Bouclier,
    Parade,
    Esquive,
    Moral,
    Chance,
    Circonstance,
    Racial,
    Resistance,
    Objet,
    Taille,
    Naturel,
    Perspicacite,
    Competence,
    Sacre,
    Profane,
    SansType,
}

impl TypeBonus {
    /// Les bonus d'esquive, de circonstance et sans type se cumulent entre eux.
    pub fn cumulable(self) -> bool {
        matches!(
            self,
            TypeBonus::Esquive | TypeBonus::Circonstance | TypeBonus::SansType
        )
    }

    pub fn nom(self) -> &'static str {
        match self {
            TypeBonus::Alteration => "altération",
            TypeBonus::Armure => "armure",
            TypeBonus::Bouclier => "bouclier",
            TypeBonus::Parade => "parade",
            TypeBonus::Esquive => "esquive",
            TypeBonus::Moral => "moral",
            TypeBonus::Chance => "chance",
            TypeBonus::Circonstance => "circonstance",
            TypeBonus::Racial => "racial",
            TypeBonus::Resistance => "de résistance",
            TypeBonus::Objet => "d'objet",
            TypeBonus::Taille => "de taille",
            TypeBonus::Naturel => "naturel",
            TypeBonus::Perspicacite => "perspicacité",
            TypeBonus::Competence => "de compétence",
            TypeBonus::Sacre => "sacré",
            TypeBonus::Profane => "profane",
            TypeBonus::SansType => "sans type",
        }
    }
}

/// Ce qu'un effet modifie, par identifiant texte (ex. `carac.dex`, `ca`, `jds.vig`,
/// `comp.perception`) : les référentiels de données les citent sans toucher au moteur.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Cible(pub String);

impl Cible {
    pub fn new(id: &str) -> Self {
        Cible(id.to_string())
    }
}

/// Un effet. Un malus est un effet de valeur négative : les malus se cumulent toujours.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Effet {
    pub cible: Cible,
    pub type_bonus: TypeBonus,
    pub valeur: i32,
    /// D'où vient l'effet, pour l'explication (« Elfe », « Cotte de mailles »…).
    pub source: String,
}

impl Effet {
    pub fn new(cible: &str, type_bonus: TypeBonus, valeur: i32, source: &str) -> Self {
        Effet {
            cible: Cible::new(cible),
            type_bonus,
            valeur,
            source: source.to_string(),
        }
    }
}
