//! Référentiel des règles d'intermède (Guide de campagne / Ultimate Campaign) et calculs.
//!
//! - [`referentiel`] : données officielles, recherche, catégories de communauté
//! - [`personnalisation`] : surcouche de données personnalisées, sans écraser l'officiel
//! - [`options`] : règles personnalisées activables (dont la répartition stricte des bonus)
//! - [`calculs`] : test de capital, gains, coûts de production, améliorations, accélération
//! - [`repartition`] : affectation des bonus des salles/équipes aux ressources produites
//! - [`controles`] : contrôles de cohérence du référentiel (avertissements)

pub mod calculs;
pub mod controles;
mod donnees_officielles;
pub mod niveaux_equipes;
pub mod options;
pub mod personnalisation;
pub mod referentiel;
pub mod repartition;

pub use referentiel::Referentiel;

/// Charge le référentiel officiel embarqué dans l'exécutable.
pub fn referentiel_officiel() -> Referentiel {
    donnees_officielles::referentiel_officiel()
}
