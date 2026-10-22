//! Entités d'une campagne : personnages, localités, transferts en cours.

use domaine::{CoutCapitaux, Montant, TypeCapital};

macro_rules! identifiant {
    ($nom:ident, $prefixe:literal) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
        pub struct $nom(pub u32);
        impl std::fmt::Display for $nom {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, concat!($prefixe, "{}"), self.0)
            }
        }
    };
}
identifiant!(IdPersonnage, "PJ-");
identifiant!(IdLocalite, "L-");
identifiant!(IdTransfert, "T-");

/// Catégorie de communauté retenue pour une localité.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum CategorieLocalite {
    /// Catégorie officielle du référentiel (id : « village », « metropole »…).
    Officielle { id: String },
    /// Lieu non couvert par une catégorie officielle : nom et limite définis par l'utilisateur.
    Personnalisee {
        nom: String,
        limite_depense_jour: u32,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Localite {
    pub id: IdLocalite,
    pub nom: String,
    /// Libellé libre : ville, village, lieu-dit, avant-poste…
    pub genre: String,
    pub population: Option<u32>,
    pub categorie: CategorieLocalite,
    /// Trésorerie en po de la localité ; peut devenir négative (affichée en rouge).
    pub tresorerie: Montant,
    /// Réserves de capitaux ; jamais négatives.
    pub capitaux: CoutCapitaux,
    pub notes: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Personnage {
    pub id: IdPersonnage,
    pub nom: String,
    /// Compte personnel en po, distinct des trésoreries des localités.
    pub compte: Montant,
    /// Localité où se trouve le PJ (usure des capitaux et des entreprises, GC p. 82).
    pub localite: Option<IdLocalite>,
    /// Valeur de Prestige, ou niveau + modificateur de Charisme (limite de recrutement, GC p. 93).
    pub prestige: Option<i32>,
}

/// Ce qui est transféré : des po ou un capital. L'Influence ne peut pas être déplacée
/// (GC p. 81 « Déplacer des capitaux » ; décision de l'utilisateur).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Lot {
    Po(Montant),
    Capital(TypeCapital, u32),
}

impl std::fmt::Display for Lot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Lot::Po(m) => write!(f, "{m}"),
            Lot::Capital(c, n) => write!(f, "{n} {c}"),
        }
    }
}

/// Transfert entre localités, en route jusqu'au jour d'arrivée.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Transfert {
    pub id: IdTransfert,
    pub de: IdLocalite,
    pub vers: IdLocalite,
    pub lot: Lot,
    pub jour_depart: u32,
    pub jour_arrivee: u32,
}
