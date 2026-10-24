//! Mode aventure : horloge de campagne, état vital, effets temporisés, repos, faim et soif.
//!
//! Bibliothèque indépendante de l'interface. Le moteur de personnage (`fiches`) reste le seul à
//! calculer les valeurs ; ce module suit le temps, la santé et les ressources, et dit à la
//! campagne quels événements appliquer aux fiches.

pub mod etat;
pub mod survie;
pub mod vital;

pub use etat::{
    nom_dans_la_fiche, Contenu, Contexte, ContextePj, Entree, ErreurAventure, EtatAventure,
    EvenementAventure, IdPj, Sortie, SuiviPj,
};
pub use survie::{Besoin, Survie, TestDu};
pub use vital::{SoinApplique, Vital};

#[cfg(test)]
mod tests;
