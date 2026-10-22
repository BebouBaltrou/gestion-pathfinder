//! Fiches du référentiel, telles que décrites dans « Comment lire le profil d'une unité »
//! (Guide de campagne p. 94-95).

use crate::ressource::{CoutCapitaux, Ressource};
use crate::source::Source;

/// Ressources auxquelles le bonus d'une salle ou d'une équipe peut s'appliquer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Admissibles {
    /// « +10 po ou Influence » : liste explicite.
    Liste { ressources: Vec<Ressource> },
    /// « +10 Capitaux » : n'importe quelle ressource.
    Toutes,
    /// « +5 capitaux (d'un type que le bâtiment génère déjà) » : Clocher, Cour, Étalage (PC-05).
    TypesDuBatiment,
    /// Extension Ameublement : +5 au test de la salle qui la porte.
    SalleHote,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Recettes {
    pub bonus: u32,
    pub admissibles: Admissibles,
    /// Texte imprimé, pour affichage et vérification.
    pub texte: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Taille {
    /// Cases de 1,50 m.
    Cases {
        min: u32,
        max: u32,
    },
    Personnes {
        nombre: u32,
    },
    Texte {
        texte: String,
    },
}

/// Salle, extension de salle ou équipe d'employés : même profil (p. 94).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Unite {
    pub id: String,
    pub nom: String,
    pub source: Source,
    /// `None` si le livre renvoie à la description (salle piégée, piège).
    pub cout: Option<CoutCapitaux>,
    /// Prix imprimé pour acheter l'unité terminée.
    pub prix_po: Option<u32>,
    pub duree_jours: Option<u32>,
    pub recettes: Option<Recettes>,
    pub avantages: Option<String>,
    pub taille: Option<Taille>,
    pub amelioration_de: Vec<String>,
    pub amelioration_en: Vec<String>,
    pub point_a_confirmer: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Gestionnaire {
    pub id: String,
    pub nom: String,
    pub source: Source,
    pub salaire_po_jour: u32,
    pub competences: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ElementModele {
    Unite {
        id: String,
        quantite: u32,
    },
    Extension {
        id: String,
        quantite: u32,
        sur: String,
    },
}

/// Modèle de bâtiment ou d'organisation (p. 111-119).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Modele {
    pub id: String,
    pub nom: String,
    pub source: Source,
    pub cout: CoutCapitaux,
    pub prix_po: u32,
    pub composition: Vec<ElementModele>,
    /// Liste telle qu'imprimée, conservée même quand la composition est corrigée.
    pub composition_imprimee: String,
    pub point_a_confirmer: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValeurCapital {
    pub ressource: Ressource,
    pub cout_achat_po: u32,
    pub cout_produit_po: u32,
    pub source: Source,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CategorieCommunaute {
    pub id: String,
    pub nom: String,
    pub population_min: u32,
    pub population_max: Option<u32>,
    /// Unités de capital dépensables par jour (p. 81).
    pub limite_depense_jour: u32,
    pub source_population: Source,
    pub source_limite: Source,
    pub point_a_confirmer: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatutPoint {
    Ouvert,
    Valide,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PointAConfirmer {
    pub code: String,
    pub question: String,
    pub source: Source,
    pub lecture_appliquee: String,
    pub statut: StatutPoint,
    pub decision: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LivreSource {
    pub code: String,
    pub titre: String,
    pub editeur: String,
    pub url: String,
}
