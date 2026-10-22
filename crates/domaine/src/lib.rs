//! Types purs du domaine, sans dépendance à l'interface ni au stockage.
//!
//! - [`ressource`] : po et capitaux (Marchandises, Influence, Travail, Magie)
//! - [`monnaie`] : montants en pièces d'argent, sans flottant
//! - [`source`] : provenance et statut (officiel / personnalisé / à confirmer)
//! - [`fiches`] : fiches du référentiel (salles, équipes, gestionnaires, modèles…)

pub mod fiches;
pub mod monnaie;
pub mod ressource;
pub mod source;

pub use monnaie::Montant;
pub use ressource::{CoutCapitaux, Ressource, TypeCapital};
pub use source::{Source, Statut};
