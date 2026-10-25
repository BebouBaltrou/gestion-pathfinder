//! Moteur de fiche de personnage (Pathfinder 1, Core Rulebook).
//!
//! Principe : les règles sont des données ; tout (race, classe, don, objet, état…) produit
//! des [`Effet`] typés ; un seul moteur ([`calculer`]) les agrège selon les règles
//! d'empilement et renvoie un [`Detail`] qui explique chaque terme.

pub mod bonus;
pub mod caracteristiques;
#[cfg(feature = "serde")]
pub mod catalogue;
#[cfg(feature = "serde")]
pub mod divin;
#[cfg(feature = "serde")]
pub mod donnees;
pub mod effet;
#[cfg(feature = "serde")]
pub mod equipement;
#[cfg(feature = "serde")]
pub mod etats;
#[cfg(feature = "serde")]
pub mod exemple;
#[cfg(feature = "serde")]
pub mod fiche;
#[cfg(feature = "serde")]
pub mod formes;
pub mod generation;
#[cfg(feature = "serde")]
pub mod journal;
#[cfg(feature = "serde")]
pub mod objets_magiques;
#[cfg(feature = "serde")]
pub mod preparation;
#[cfg(feature = "serde")]
pub mod prerequis;
#[cfg(feature = "serde")]
pub mod progression;
#[cfg(feature = "serde")]
pub mod traductions;

pub use bonus::{calculer, Detail, Terme};
pub use caracteristiques::{modificateur, Carac, Caracteristique};
#[cfg(feature = "serde")]
pub use donnees::Referentiel;
pub use effet::{Cible, Effet, TypeBonus};
#[cfg(feature = "serde")]
pub use fiche::{Fiche, Probleme, Valeurs};
#[cfg(all(test, feature = "serde"))]
mod tests_classes;
#[cfg(all(test, feature = "serde"))]
mod tests_equipement;
