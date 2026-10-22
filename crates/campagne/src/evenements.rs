//! Événements du journal. Chaque modification de la campagne est un événement immuable ;
//! l'état est recalculé en rejouant le journal (annuler/rétablir, audit).

use crate::modele::*;
use domaine::{CoutCapitaux, Montant};

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Evenement {
    PersonnageCree {
        id: IdPersonnage,
        nom: String,
        compte_initial: Montant,
        prestige: Option<i32>,
    },
    PersonnageModifie {
        id: IdPersonnage,
        nom: String,
        prestige: Option<i32>,
    },
    /// Le PJ change de localité (ou quitte toute localité : `None`).
    PersonnageDeplace {
        id: IdPersonnage,
        vers: Option<IdLocalite>,
    },
    LocaliteCreee {
        id: IdLocalite,
        nom: String,
        genre: String,
        population: Option<u32>,
        categorie: CategorieLocalite,
        notes: String,
    },
    LocaliteModifiee {
        id: IdLocalite,
        nom: String,
        genre: String,
        population: Option<u32>,
        categorie: CategorieLocalite,
        notes: String,
    },
    /// Compte du PJ → trésorerie de la localité.
    InjectionPo {
        pj: IdPersonnage,
        localite: IdLocalite,
        montant: Montant,
    },
    /// Trésorerie de la localité → compte du PJ.
    RetraitPo {
        localite: IdLocalite,
        pj: IdPersonnage,
        montant: Montant,
    },
    /// Gain ou dépense du PJ hors de l'application (butin, achats…).
    AjustementCompte {
        pj: IdPersonnage,
        delta: Montant,
        motif: String,
    },
    /// Correction manuelle des soldes d'une localité (récompense du MJ, erreur de saisie…).
    AjustementLocalite {
        localite: IdLocalite,
        delta_po: Montant,
        /// Ajouts de capitaux.
        ajout: CoutCapitaux,
        /// Retraits de capitaux.
        retrait: CoutCapitaux,
        motif: String,
    },
    TransfertEnvoye {
        id: IdTransfert,
        de: IdLocalite,
        vers: IdLocalite,
        lot: Lot,
        duree_jours: u32,
    },
    JoursAvances {
        nombre: u32,
    },
    Note {
        texte: String,
    },
}

/// Entrée du journal : numéro d'ordre et jour de jeu où l'événement a eu lieu.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Entree {
    pub numero: u64,
    pub jour: u32,
    pub evenement: Evenement,
}
