//! Durées PF1 : une énumération extensible, pas un nombre.

use crate::instant::{Instant, ROUNDS_PAR_HEURE, ROUNDS_PAR_JOUR, ROUNDS_PAR_MINUTE};

/// Début ou fin du tour du propriétaire de l'effet.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
pub enum MomentDuTour {
    Debut,
    Fin,
}

/// Durée d'origine d'un effet (celle du livre ou saisie par le joueur).
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
#[non_exhaustive]
pub enum Duree {
    Instantanee,
    Permanente,
    Rounds(u32),
    Minutes(u32),
    Heures(u32),
    Jours(u32),
    /// Tant que le lanceur se concentre : l'effet se termine quand on le retire.
    Concentration,
    /// Jusqu'à ce que l'effet soit utilisé (sort « à décharge »).
    JusquaDecharge,
    /// « Jusqu'au début/à la fin de ton prochain tour » : s'appuie sur le tour du propriétaire.
    JusquauProchainTour(MomentDuTour),
    /// Lié à un événement ou à une condition décrite en texte (« jusqu'au prochain repos »…).
    LieAUnEvenement(String),
}

/// Quand l'effet prend fin.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
pub enum Echeance {
    /// À un instant précis de l'horloge.
    A(Instant),
    /// Aucun instant : l'effet se termine sur un événement (retrait, décharge, tour, repos…)
    /// ou ne se termine jamais.
    Indeterminee,
}

impl Duree {
    /// Longueur en rounds quand la durée est chiffrée.
    pub fn en_rounds(&self) -> Option<i64> {
        match self {
            Duree::Rounds(n) => Some(*n as i64),
            Duree::Minutes(n) => Some(*n as i64 * ROUNDS_PAR_MINUTE),
            Duree::Heures(n) => Some(*n as i64 * ROUNDS_PAR_HEURE),
            Duree::Jours(n) => Some(*n as i64 * ROUNDS_PAR_JOUR),
            _ => None,
        }
    }

    /// Échéance d'un effet qui commence à `debut`.
    pub fn echeance(&self, debut: Instant) -> Echeance {
        match self.en_rounds() {
            Some(r) => Echeance::A(debut.plus(r)),
            None => Echeance::Indeterminee,
        }
    }

    /// Les durées « en rounds » et les « jusqu'à… » sont dissipées à la fin d'un combat
    /// (simplification acceptée, spécification §1.2).
    pub fn dissipee_en_fin_de_combat(&self) -> bool {
        matches!(
            self,
            Duree::Rounds(_) | Duree::JusquauProchainTour(_) | Duree::LieAUnEvenement(_)
        )
    }

    pub fn texte(&self) -> String {
        match self {
            Duree::Instantanee => "instantanée".into(),
            Duree::Permanente => "permanente".into(),
            Duree::Rounds(n) => format!("{n} round(s)"),
            Duree::Minutes(n) => format!("{n} minute(s)"),
            Duree::Heures(n) => format!("{n} heure(s)"),
            Duree::Jours(n) => format!("{n} jour(s)"),
            Duree::Concentration => "concentration".into(),
            Duree::JusquaDecharge => "jusqu'à décharge".into(),
            Duree::JusquauProchainTour(MomentDuTour::Debut) => {
                "jusqu'au début de son prochain tour".into()
            }
            Duree::JusquauProchainTour(MomentDuTour::Fin) => {
                "jusqu'à la fin de son prochain tour".into()
            }
            Duree::LieAUnEvenement(t) => t.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn echeances() {
        let d = Instant(100);
        assert_eq!(Duree::Rounds(3).echeance(d), Echeance::A(Instant(103)));
        assert_eq!(Duree::Minutes(2).echeance(d), Echeance::A(Instant(120)));
        assert_eq!(Duree::Heures(1).echeance(d), Echeance::A(Instant(700)));
        assert_eq!(Duree::Jours(1).echeance(d), Echeance::A(Instant(14_500)));
        assert_eq!(Duree::Concentration.echeance(d), Echeance::Indeterminee);
        assert_eq!(Duree::Permanente.echeance(d), Echeance::Indeterminee);
    }
}
