//! Moteur de fiche de personnage (Pathfinder 1, Core Rulebook).
//!
//! Principe : les règles sont des données ; tout (race, classe, don, objet, état…) produit
//! des [`Effet`] typés ; un seul moteur ([`calculer`]) les agrège selon les règles
//! d'empilement et renvoie un [`Detail`] qui explique chaque terme.

pub mod bonus;
pub mod caracteristiques;
#[cfg(feature = "serde")]
pub mod donnees;
pub mod effet;
#[cfg(feature = "serde")]
pub mod fiche;
pub mod generation;

pub use bonus::{calculer, Detail, Terme};
pub use caracteristiques::{modificateur, Carac, Caracteristique};
#[cfg(feature = "serde")]
pub use donnees::Referentiel;
pub use effet::{Cible, Effet, TypeBonus};
#[cfg(feature = "serde")]
pub use fiche::{Fiche, Probleme, Valeurs};
