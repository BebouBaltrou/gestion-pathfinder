//! Mode combat : copie de travail transactionnelle de l'état d'aventure, initiative, tours,
//! annulation action par action, sorts, journal rejouable (dés saisis inclus).
//!
//! Bibliothèque indépendante de l'interface. Le combat ne calcule aucune règle de personnage :
//! il appelle le mode aventure (`aventure`) qui s'appuie sur le moteur de fiches.

pub mod catalogue;
pub mod combat;
pub mod economie;
pub mod etat;
pub mod objets;
pub mod sorts;
pub mod xp;

#[cfg(feature = "serde")]
pub use combat::persistance;
pub use combat::{ApercuFin, Combat, Ctx, RecapPj, Resultat, Validation, FORMAT_COMBAT};
pub use economie::{Depense, EconomieTour, TypeAction};
pub use etat::{ActionCombat, Demande, EntreeCombat, ErreurCombat, EtatCombat, Init, Phase, Stats};
pub use objets::ParamsObjet;
pub use sorts::{ApercuSort, Distraction, ParamsLancer};

#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_actions;
#[cfg(test)]
mod tests_objets;
#[cfg(test)]
mod tests_sorts;
