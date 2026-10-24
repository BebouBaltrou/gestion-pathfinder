//! Les six caractéristiques (Core, chap. 1).

use crate::bonus::{calculer, Detail};
use crate::effet::{Cible, Effet};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Carac {
    For,
    Dex,
    Con,
    Int,
    Sag,
    Cha,
}

impl Carac {
    pub const TOUTES: [Carac; 6] = [
        Carac::For,
        Carac::Dex,
        Carac::Con,
        Carac::Int,
        Carac::Sag,
        Carac::Cha,
    ];

    pub fn abrege(self) -> &'static str {
        match self {
            Carac::For => "FOR",
            Carac::Dex => "DEX",
            Carac::Con => "CON",
            Carac::Int => "INT",
            Carac::Sag => "SAG",
            Carac::Cha => "CHA",
        }
    }

    /// Identifiant de cible utilisé par les effets (`carac.dex`…).
    pub fn cible(self) -> Cible {
        Cible(format!("carac.{}", self.abrege().to_lowercase()))
    }
}

/// Modificateur = ⌊(score − 10) / 2⌋, arrondi vers le bas y compris pour les scores < 10.
pub fn modificateur(score: i32) -> i32 {
    (score - 10).div_euclid(2)
}

/// Une caractéristique : score de base, dommages et drain.
///
/// - le **drain** réduit le score lui-même ;
/// - les **dommages** ne changent pas le score mais réduisent le modificateur
///   (1 point de modificateur par tranche de 2 points de dommages).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Caracteristique {
    pub base: i32,
    pub dommages: i32,
    pub drain: i32,
}

impl Caracteristique {
    pub fn new(base: i32) -> Self {
        Caracteristique {
            base,
            dommages: 0,
            drain: 0,
        }
    }

    /// Score avec bonus typés et drain (les dommages ne comptent pas ici).
    pub fn score(&self, carac: Carac, effets: &[Effet]) -> Detail {
        let mut base = vec![("score de base", self.base)];
        if self.drain != 0 {
            base.push(("drain", -self.drain));
        }
        calculer(&base, effets, &carac.cible())
    }

    /// Modificateur, dommages inclus.
    pub fn modificateur(&self, carac: Carac, effets: &[Effet]) -> i32 {
        modificateur(self.score(carac, effets).total - self.dommages)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::effet::TypeBonus;

    #[test]
    fn modificateurs_arrondis_vers_le_bas() {
        let attendu = [
            (1, -5),
            (7, -2),
            (8, -1),
            (9, -1),
            (10, 0),
            (11, 0),
            (12, 1),
            (18, 4),
        ];
        for (score, m) in attendu {
            assert_eq!(modificateur(score), m, "score {score}");
        }
    }

    #[test]
    fn bonus_de_meme_type_non_cumules_sur_le_score() {
        let c = Caracteristique::new(14);
        let e = [
            Effet::new("carac.dex", TypeBonus::Racial, 2, "Elfe"),
            Effet::new("carac.dex", TypeBonus::Alteration, 2, "Gants"),
            Effet::new("carac.dex", TypeBonus::Alteration, 4, "Potion"),
        ];
        assert_eq!(c.score(Carac::Dex, &e).total, 14 + 2 + 4);
    }

    #[test]
    fn dommages_changent_le_modificateur_pas_le_score() {
        let mut c = Caracteristique::new(16);
        c.dommages = 3;
        assert_eq!(c.score(Carac::Con, &[]).total, 16);
        assert_eq!(c.modificateur(Carac::Con, &[]), 1); // (16 - 3 - 10) / 2 = 1,5 -> 1
    }

    #[test]
    fn drain_reduit_le_score() {
        let mut c = Caracteristique::new(16);
        c.drain = 2;
        assert_eq!(c.score(Carac::Con, &[]).total, 14);
        assert_eq!(c.modificateur(Carac::Con, &[]), 2);
    }
}
