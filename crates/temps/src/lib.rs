//! Temps de campagne : horloge en rounds, durées PF1, effets temporisés.
//!
//! Bibliothèque indépendante de l'interface et des fiches : le contenu d'un effet est un type
//! générique, les bonus typés restent dans la crate `fiches`.

pub mod duree;
pub mod effets;
pub mod horloge;
pub mod instant;

pub use duree::{Duree, Echeance, MomentDuTour};
pub use effets::{
    minutes_arrondies, ApercuRecul, Arrondi, BilanFinDeCombat, Dissipation, Ensemble, IdEffet,
    RaisonFin, Temporise, Termine,
};
pub use horloge::{Horloge, Mode, Pas};
pub use instant::{
    texte_rounds, Instant, ROUNDS_PAR_HEURE, ROUNDS_PAR_JOUR, ROUNDS_PAR_MINUTE, SECONDES_PAR_ROUND,
};

use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErreurTemps {
    /// L'horloge horaire est suspendue pendant un combat.
    HorlogeSuspendue,
    HorsCombat,
    DejaEnCombat,
    DureeInstantanee,
    EffetInconnu(IdEffet),
    /// Une opération touche des effets créés dans l'intervalle : le joueur doit confirmer.
    ConfirmationRequise,
    Invalide(String),
}

impl fmt::Display for ErreurTemps {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErreurTemps::HorlogeSuspendue => {
                write!(f, "l'horloge horaire est suspendue pendant le combat")
            }
            ErreurTemps::HorsCombat => write!(f, "pas de combat en cours"),
            ErreurTemps::DejaEnCombat => write!(f, "un combat est déjà en cours"),
            ErreurTemps::DureeInstantanee => {
                write!(f, "un effet instantané ne se suit pas dans le temps")
            }
            ErreurTemps::EffetInconnu(id) => write!(f, "effet inconnu ou non applicable : {id}"),
            ErreurTemps::ConfirmationRequise => write!(f, "confirmation requise"),
            ErreurTemps::Invalide(m) => write!(f, "{m}"),
        }
    }
}

impl std::error::Error for ErreurTemps {}

#[cfg(test)]
mod proprietes;
