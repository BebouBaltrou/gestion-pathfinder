//! Faim et soif (Core Rulebook p. 444, lectures à valider).
//!
//! - Soif : on peut passer 1 jour + (score de Constitution) heures sans eau ; ensuite un test de
//!   Constitution par heure (DD 10, +1 par test précédent), sinon 1d6 dégâts non létaux.
//! - Faim : on peut passer 3 jours sans nourriture ; ensuite un test par jour (DD 10, +1 par test
//!   précédent), sinon 1d6 dégâts non létaux.
//! - Ces dégâts non létaux ne se récupèrent qu'après avoir bu ou mangé, et rendent fatigué.
//!
//! Le premier test tombe **une période complète après la fin du délai de grâce** (lecture à valider).
//! Hypothèses non modélisées : personnage de taille Petite (besoins divisés par deux) et climat chaud
//! (besoins en eau × 2 ou × 3), que le texte prévoit.

use temps::{Instant, ROUNDS_PAR_HEURE, ROUNDS_PAR_JOUR};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
pub enum Besoin {
    Eau,
    Nourriture,
}

impl Besoin {
    pub fn nom(self) -> &'static str {
        match self {
            Besoin::Eau => "soif",
            Besoin::Nourriture => "faim",
        }
    }
}

/// Un test de Constitution qui est dû.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TestDu {
    pub besoin: Besoin,
    /// Instant où le test devient dû.
    pub du_a: Instant,
    pub dd: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Survie {
    pub derniere_boisson: Instant,
    pub dernier_repas: Instant,
    /// Tests déjà passés depuis la dernière boisson / le dernier repas.
    pub tests_soif: u32,
    pub tests_faim: u32,
}

impl Survie {
    pub fn nouvelle(maintenant: Instant) -> Self {
        Survie {
            derniere_boisson: maintenant,
            dernier_repas: maintenant,
            tests_soif: 0,
            tests_faim: 0,
        }
    }

    /// Fin du délai de grâce.
    pub fn fin_de_grace(&self, besoin: Besoin, con_score: i32) -> Instant {
        match besoin {
            Besoin::Eau => self
                .derniere_boisson
                .plus(ROUNDS_PAR_JOUR + con_score.max(0) as i64 * ROUNDS_PAR_HEURE),
            Besoin::Nourriture => self.dernier_repas.plus(3 * ROUNDS_PAR_JOUR),
        }
    }

    fn periode(besoin: Besoin) -> i64 {
        match besoin {
            Besoin::Eau => ROUNDS_PAR_HEURE,
            Besoin::Nourriture => ROUNDS_PAR_JOUR,
        }
    }

    fn deja_faits(&self, besoin: Besoin) -> u32 {
        match besoin {
            Besoin::Eau => self.tests_soif,
            Besoin::Nourriture => self.tests_faim,
        }
    }

    /// Tests dus à `maintenant` et pas encore passés, du plus ancien au plus récent.
    pub fn tests_dus(&self, besoin: Besoin, maintenant: Instant, con_score: i32) -> Vec<TestDu> {
        let grace = self.fin_de_grace(besoin, con_score);
        let p = Self::periode(besoin);
        let faits = self.deja_faits(besoin);
        let mut v = Vec::new();
        for k in faits..faits.saturating_add(10_000) {
            let du_a = grace.plus(p * (k as i64 + 1));
            if du_a > maintenant {
                break;
            }
            v.push(TestDu {
                besoin,
                du_a,
                dd: 10 + k as i32,
            });
        }
        v
    }

    pub fn test_passe(&mut self, besoin: Besoin) {
        match besoin {
            Besoin::Eau => self.tests_soif += 1,
            Besoin::Nourriture => self.tests_faim += 1,
        }
    }

    pub fn boit(&mut self, maintenant: Instant) {
        self.derniere_boisson = maintenant;
        self.tests_soif = 0;
    }

    pub fn mange(&mut self, maintenant: Instant) {
        self.dernier_repas = maintenant;
        self.tests_faim = 0;
    }

    /// Recale les repères si l'horloge recule avant eux.
    pub fn recaler(&mut self, maintenant: Instant) {
        self.derniere_boisson = self.derniere_boisson.min(maintenant);
        self.dernier_repas = self.dernier_repas.min(maintenant);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn soif_apres_un_jour_plus_con_heures() {
        let s = Survie::nouvelle(Instant::DEBUT);
        // CON 12 : grâce = 24 h + 12 h = 36 h ; premier test à 37 h
        let h = |n: i64| Instant(n * ROUNDS_PAR_HEURE);
        assert!(s.tests_dus(Besoin::Eau, h(36), 12).is_empty());
        assert!(s
            .tests_dus(Besoin::Eau, h(36).plus(ROUNDS_PAR_HEURE - 1), 12)
            .is_empty());
        let dus = s.tests_dus(Besoin::Eau, h(39), 12);
        assert_eq!(dus.len(), 3);
        assert_eq!(
            dus.iter().map(|t| t.dd).collect::<Vec<_>>(),
            vec![10, 11, 12]
        );
        assert_eq!(dus[0].du_a, h(37));
    }

    #[test]
    fn faim_apres_trois_jours() {
        let mut s = Survie::nouvelle(Instant::DEBUT);
        let j = |n: i64| Instant(n * ROUNDS_PAR_JOUR);
        assert!(s.tests_dus(Besoin::Nourriture, j(3), 10).is_empty());
        assert_eq!(s.tests_dus(Besoin::Nourriture, j(4), 10).len(), 1);
        s.test_passe(Besoin::Nourriture);
        assert!(s.tests_dus(Besoin::Nourriture, j(4), 10).is_empty());
        let dus = s.tests_dus(Besoin::Nourriture, j(5), 10);
        assert_eq!((dus.len(), dus[0].dd), (1, 11));
    }

    #[test]
    fn manger_et_boire_remettent_a_zero() {
        let mut s = Survie::nouvelle(Instant::DEBUT);
        let j = |n: i64| Instant(n * ROUNDS_PAR_JOUR);
        s.test_passe(Besoin::Eau);
        s.boit(j(2));
        assert_eq!(s.tests_soif, 0);
        s.mange(j(2));
        assert!(s.tests_dus(Besoin::Nourriture, j(5), 10).is_empty());
        assert_eq!(s.tests_dus(Besoin::Nourriture, j(6), 10).len(), 1);
    }
}
