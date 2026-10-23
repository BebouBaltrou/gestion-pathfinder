//! Instant de campagne : un nombre de rounds depuis le jour 1 à 00:00.
//!
//! 1 round = 6 secondes, 1 minute = 10 rounds, 1 heure = 600 rounds, 1 jour = 14 400 rounds.
//! La correspondance avec le calendrier de Golarion (année, mois, jour) est faite par la
//! campagne ; ici on ne connaît que le « jour n° N de la campagne ».

use std::fmt;

pub const SECONDES_PAR_ROUND: i64 = 6;
pub const ROUNDS_PAR_MINUTE: i64 = 10;
pub const ROUNDS_PAR_HEURE: i64 = 600;
pub const ROUNDS_PAR_JOUR: i64 = 14_400;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(transparent))]
pub struct Instant(pub i64);

impl Instant {
    /// Jour 1 à 00:00.
    pub const DEBUT: Instant = Instant(0);

    pub fn plus(self, rounds: i64) -> Instant {
        Instant(self.0.saturating_add(rounds))
    }

    /// Nombre de rounds entre `avant` et `self` (négatif si `self` est plus tôt).
    pub fn ecart(self, avant: Instant) -> i64 {
        self.0.saturating_sub(avant.0)
    }

    /// Numéro du jour de campagne (le premier jour est le jour 1).
    pub fn jour(self) -> i64 {
        self.0.div_euclid(ROUNDS_PAR_JOUR) + 1
    }

    fn dans_le_jour(self) -> i64 {
        self.0.rem_euclid(ROUNDS_PAR_JOUR)
    }

    pub fn heure(self) -> u32 {
        (self.dans_le_jour() / ROUNDS_PAR_HEURE) as u32
    }

    pub fn minute(self) -> u32 {
        (self.dans_le_jour() % ROUNDS_PAR_HEURE / ROUNDS_PAR_MINUTE) as u32
    }

    pub fn seconde(self) -> u32 {
        (self.dans_le_jour() % ROUNDS_PAR_MINUTE * SECONDES_PAR_ROUND) as u32
    }

    /// Jour (à partir de 1), heure (0-23), minute (0-59) → instant, ou `None` si hors limites.
    pub fn depuis(jour: i64, heure: u32, minute: u32) -> Option<Instant> {
        if jour < 1 || heure > 23 || minute > 59 {
            return None;
        }
        let rounds = (jour - 1)
            .checked_mul(ROUNDS_PAR_JOUR)?
            .checked_add(heure as i64 * ROUNDS_PAR_HEURE + minute as i64 * ROUNDS_PAR_MINUTE)?;
        Some(Instant(rounds))
    }

    /// Nombre de minuits franchis en passant de `avant` à `apres` (0 si on recule).
    pub fn minuits_franchis(avant: Instant, apres: Instant) -> i64 {
        (apres.jour() - avant.jour()).max(0)
    }
}

impl fmt::Display for Instant {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "jour {}, {:02}:{:02}:{:02}",
            self.jour(),
            self.heure(),
            self.minute(),
            self.seconde()
        )
    }
}

/// « 2 h 05 min 30 s » pour une durée en rounds (arrondie à l'affichage seulement).
pub fn texte_rounds(rounds: i64) -> String {
    let neg = rounds < 0;
    let r = rounds.abs();
    let (j, h, m, s) = (
        r / ROUNDS_PAR_JOUR,
        r % ROUNDS_PAR_JOUR / ROUNDS_PAR_HEURE,
        r % ROUNDS_PAR_HEURE / ROUNDS_PAR_MINUTE,
        r % ROUNDS_PAR_MINUTE * SECONDES_PAR_ROUND,
    );
    let mut parts = Vec::new();
    if j > 0 {
        parts.push(format!("{j} j"));
    }
    if h > 0 {
        parts.push(format!("{h} h"));
    }
    if m > 0 {
        parts.push(format!("{m} min"));
    }
    if s > 0 || parts.is_empty() {
        parts.push(format!("{s} s"));
    }
    format!("{}{}", if neg { "-" } else { "" }, parts.join(" "))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constantes_coherentes() {
        assert_eq!(ROUNDS_PAR_MINUTE * 6, 60);
        assert_eq!(ROUNDS_PAR_HEURE, 60 * ROUNDS_PAR_MINUTE);
        assert_eq!(ROUNDS_PAR_JOUR, 24 * ROUNDS_PAR_HEURE);
    }

    #[test]
    fn jour_heure_minute() {
        let i = Instant::depuis(3, 14, 5).unwrap();
        assert_eq!(
            (i.jour(), i.heure(), i.minute(), i.seconde()),
            (3, 14, 5, 0)
        );
        assert_eq!(i.plus(1).seconde(), 6);
        assert_eq!(i.to_string(), "jour 3, 14:05:00");
        assert!(Instant::depuis(0, 0, 0).is_none());
        assert!(Instant::depuis(1, 24, 0).is_none());
    }

    #[test]
    fn minuit() {
        let soir = Instant::depuis(1, 23, 50).unwrap();
        assert_eq!(Instant::minuits_franchis(soir, soir.plus(100)), 1);
        assert_eq!(Instant::minuits_franchis(soir, soir.plus(10)), 0);
        assert_eq!(Instant::minuits_franchis(soir.plus(100), soir), 0);
        assert_eq!(
            Instant::minuits_franchis(soir, soir.plus(ROUNDS_PAR_JOUR * 2)),
            2
        );
    }

    #[test]
    fn texte() {
        assert_eq!(texte_rounds(74), "7 min 24 s");
        assert_eq!(texte_rounds(0), "0 s");
        assert_eq!(texte_rounds(ROUNDS_PAR_JOUR + ROUNDS_PAR_HEURE), "1 j 1 h");
    }
}
