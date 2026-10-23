//! Horloge de campagne : avance par pas hors combat, par round en combat.

use crate::instant::{Instant, ROUNDS_PAR_MINUTE};
use crate::ErreurTemps;

/// Pas d'avance du mode aventure.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
pub enum Pas {
    Minutes5,
    Minutes10,
    Minutes30,
    Heure1,
}

impl Pas {
    pub fn rounds(self) -> i64 {
        match self {
            Pas::Minutes5 => 5 * ROUNDS_PAR_MINUTE,
            Pas::Minutes10 => 10 * ROUNDS_PAR_MINUTE,
            Pas::Minutes30 => 30 * ROUNDS_PAR_MINUTE,
            Pas::Heure1 => 60 * ROUNDS_PAR_MINUTE,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
pub enum Mode {
    Aventure,
    /// Combat commencé à `debut` : l'horloge horaire est suspendue.
    Combat {
        debut: Instant,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Horloge {
    pub maintenant: Instant,
    pub mode: Mode,
}

impl Horloge {
    pub fn nouvelle(maintenant: Instant) -> Self {
        Horloge {
            maintenant,
            mode: Mode::Aventure,
        }
    }

    pub fn en_combat(&self) -> bool {
        matches!(self.mode, Mode::Combat { .. })
    }

    /// Avance de `rounds` (aventure seulement). Renvoie le nombre de minuits franchis,
    /// à répercuter sur les jours d'intermède.
    pub fn avancer_de(&mut self, rounds: i64) -> Result<i64, ErreurTemps> {
        if self.en_combat() {
            return Err(ErreurTemps::HorlogeSuspendue);
        }
        if rounds < 0 {
            return Err(ErreurTemps::Invalide(
                "pour reculer, utiliser le réglage de l'heure".into(),
            ));
        }
        let avant = self.maintenant;
        self.maintenant = avant.plus(rounds);
        Ok(Instant::minuits_franchis(avant, self.maintenant))
    }

    pub fn avancer(&mut self, pas: Pas) -> Result<i64, ErreurTemps> {
        self.avancer_de(pas.rounds())
    }

    pub fn entrer_en_combat(&mut self) -> Result<(), ErreurTemps> {
        if self.en_combat() {
            return Err(ErreurTemps::DejaEnCombat);
        }
        self.mode = Mode::Combat {
            debut: self.maintenant,
        };
        Ok(())
    }

    /// Un round de combat de plus. Renvoie le nombre de minuits franchis.
    pub fn round_suivant(&mut self) -> Result<i64, ErreurTemps> {
        if !self.en_combat() {
            return Err(ErreurTemps::HorsCombat);
        }
        let avant = self.maintenant;
        self.maintenant = avant.plus(1);
        Ok(Instant::minuits_franchis(avant, self.maintenant))
    }

    /// Un round de combat de moins (annulation d'un « tour suivant »).
    pub fn round_precedent(&mut self) -> Result<(), ErreurTemps> {
        match self.mode {
            Mode::Combat { debut } if self.maintenant > debut => {
                self.maintenant = self.maintenant.plus(-1);
                Ok(())
            }
            Mode::Combat { .. } => Err(ErreurTemps::Invalide("on est au premier round".into())),
            Mode::Aventure => Err(ErreurTemps::HorsCombat),
        }
    }

    /// Sortie de combat : si `valider`, l'horloge reste après les rounds joués ;
    /// sinon (combat abandonné) elle revient au début du combat.
    pub fn sortir_du_combat(&mut self, valider: bool) -> Result<(), ErreurTemps> {
        let Mode::Combat { debut } = self.mode else {
            return Err(ErreurTemps::HorsCombat);
        };
        if !valider {
            self.maintenant = debut;
        }
        self.mode = Mode::Aventure;
        Ok(())
    }

    /// Réglage manuel de l'heure (avance ou recul), aventure seulement. Les effets doivent être
    /// réglés par l'appelant : `Ensemble::apercu_recul` / `appliquer_recul` pour un recul.
    pub fn regler(&mut self, nouvel: Instant) -> Result<(), ErreurTemps> {
        if self.en_combat() {
            return Err(ErreurTemps::HorlogeSuspendue);
        }
        if nouvel < Instant::DEBUT {
            return Err(ErreurTemps::Invalide(
                "avant le début de la campagne".into(),
            ));
        }
        self.maintenant = nouvel;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pas_et_minuit() {
        let mut h = Horloge::nouvelle(Instant::depuis(1, 23, 30).unwrap());
        assert_eq!(h.avancer(Pas::Minutes10), Ok(0));
        assert_eq!(h.avancer(Pas::Minutes30), Ok(1));
        assert_eq!(h.maintenant, Instant::depuis(2, 0, 10).unwrap());
    }

    #[test]
    fn combat_suspend_les_pas() {
        let mut h = Horloge::nouvelle(Instant(100));
        h.entrer_en_combat().unwrap();
        assert_eq!(h.avancer(Pas::Heure1), Err(ErreurTemps::HorlogeSuspendue));
        assert_eq!(h.regler(Instant(0)), Err(ErreurTemps::HorlogeSuspendue));
        h.round_suivant().unwrap();
        h.round_suivant().unwrap();
        assert_eq!(h.maintenant, Instant(102));
        h.round_precedent().unwrap();
        h.sortir_du_combat(true).unwrap();
        assert_eq!(h.maintenant, Instant(101));
        assert_eq!(h.round_suivant(), Err(ErreurTemps::HorsCombat));
    }

    #[test]
    fn combat_abandonne_revient_au_debut() {
        let mut h = Horloge::nouvelle(Instant(100));
        h.entrer_en_combat().unwrap();
        for _ in 0..5 {
            h.round_suivant().unwrap();
        }
        h.sortir_du_combat(false).unwrap();
        assert_eq!(h.maintenant, Instant(100));
        assert!(!h.en_combat());
    }

    #[test]
    fn reglage_manuel() {
        let mut h = Horloge::nouvelle(Instant(100));
        h.regler(Instant(50)).unwrap();
        assert_eq!(h.maintenant, Instant(50));
        assert!(h.regler(Instant(-1)).is_err());
        assert!(h.avancer_de(-1).is_err());
    }
}
