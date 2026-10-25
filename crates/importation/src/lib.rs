//! Importation d'une fiche de personnage PDF : lecture des champs, lecture du gabarit de la
//! feuille, contrôle de cohérence avec le moteur de règles (`fiches`).
//!
//! Rien n'est modifié ici : le résultat est un [`Rapport`] que l'interface présentera ligne par
//! ligne (corriger / garder).

pub mod controle;
pub mod dons;
pub mod feuille;
pub mod lecteur;

pub use controle::{
    appliquer_gardes, controler, proposer, Garde, Gravite, Ligne, Proposition, Rapport,
};
pub use feuille::{lire_feuille, Feuille};
pub use lecteur::{lire_champs, lire_champs_octets, Champs};
