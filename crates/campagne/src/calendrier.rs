//! Calendrier : compteur de jours de campagne et date de Golarion (Comput d'Absalom, AR).
//!
//! Hors règles d'intermède : données de cadre de jeu.
//! Mois et jours de la semaine (VF) : <https://www.pathfinder-fr.org/Wiki/Golarion.Calendrier.ashx>.
//! Le jour bissextile s'ajoute à la fin de Calistril ; sa fréquence diffère selon les éditions
//! (tous les 8 ans en PF1, tous les 4 ans depuis PF2) : point à confirmer PC-13.

pub const MOIS: [(&str, u8); 12] = [
    ("Abadius", 31),
    ("Calistril", 28),
    ("Pharast", 31),
    ("Gozran", 30),
    ("Desnus", 31),
    ("Sarénith", 30),
    ("Érastus", 31),
    ("Arodus", 31),
    ("Rova", 30),
    ("Lamashan", 31),
    ("Neth", 30),
    ("Kuthona", 31),
];

pub const JOURS_SEMAINE: [&str; 7] = [
    "lunedi",
    "marnedi",
    "mercantdi",
    "juredi",
    "ventdi",
    "astredi",
    "solardi",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum RegleBissextile {
    Aucune,
    /// Années divisibles par 4 (PF2 ; 4724 AR cité comme bissextile).
    TousLes4Ans,
    /// Années divisibles par 8 (PF1 ; ancrage non publié).
    TousLes8Ans,
}

impl RegleBissextile {
    pub fn est_bissextile(self, annee: i32) -> bool {
        match self {
            RegleBissextile::Aucune => false,
            RegleBissextile::TousLes4Ans => annee.rem_euclid(4) == 0,
            RegleBissextile::TousLes8Ans => annee.rem_euclid(8) == 0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DateGolarion {
    pub annee: i32,
    /// 1 à 12
    pub mois: u8,
    /// 1 à 31
    pub jour: u8,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Calendrier {
    /// Date de Golarion du jour 1 de la campagne.
    pub origine: DateGolarion,
    /// Jour de la semaine du jour 1 (0 = lunedi). Aucune source ne fixe la correspondance.
    pub jour_semaine_origine: u8,
    pub bissextile: RegleBissextile,
}

impl Default for Calendrier {
    fn default() -> Self {
        Calendrier {
            origine: DateGolarion {
                annee: 4712,
                mois: 1,
                jour: 1,
            },
            jour_semaine_origine: 0,
            bissextile: RegleBissextile::TousLes4Ans,
        }
    }
}

impl Calendrier {
    pub fn jours_du_mois(&self, annee: i32, mois: u8) -> u8 {
        let base = MOIS[(mois - 1) as usize].1;
        if mois == 2 && self.bissextile.est_bissextile(annee) {
            base + 1
        } else {
            base
        }
    }

    /// Date de Golarion du jour `n` de la campagne (n ≥ 1).
    pub fn date(&self, n: u32) -> DateGolarion {
        let mut d = self.origine;
        let mut reste = n.saturating_sub(1);
        while reste > 0 {
            let fin_mois = self.jours_du_mois(d.annee, d.mois);
            let dispo = (fin_mois - d.jour) as u32;
            if reste <= dispo {
                d.jour += reste as u8;
                break;
            }
            reste -= dispo + 1;
            d.jour = 1;
            if d.mois == 12 {
                d.mois = 1;
                d.annee += 1;
            } else {
                d.mois += 1;
            }
        }
        d
    }

    pub fn jour_semaine(&self, n: u32) -> &'static str {
        let i = (self.jour_semaine_origine as u32 + n.saturating_sub(1)) % 7;
        JOURS_SEMAINE[i as usize]
    }

    /// « juredi 12 Pharast 4712 AR »
    pub fn libelle(&self, n: u32) -> String {
        let d = self.date(n);
        format!(
            "{} {} {} {} AR",
            self.jour_semaine(n),
            d.jour,
            MOIS[(d.mois - 1) as usize].0,
            d.annee
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn annee_de_365_jours() {
        assert_eq!(MOIS.iter().map(|m| m.1 as u32).sum::<u32>(), 365);
    }

    #[test]
    fn progression_des_dates() {
        let c = Calendrier {
            bissextile: RegleBissextile::Aucune,
            ..Calendrier::default()
        };
        assert_eq!(
            c.date(1),
            DateGolarion {
                annee: 4712,
                mois: 1,
                jour: 1
            }
        );
        assert_eq!(
            c.date(32),
            DateGolarion {
                annee: 4712,
                mois: 2,
                jour: 1
            }
        );
        assert_eq!(
            c.date(365),
            DateGolarion {
                annee: 4712,
                mois: 12,
                jour: 31
            }
        );
        assert_eq!(
            c.date(366),
            DateGolarion {
                annee: 4713,
                mois: 1,
                jour: 1
            }
        );
        assert_eq!(c.libelle(1), "lunedi 1 Abadius 4712 AR");
        assert_eq!(c.jour_semaine(8), "lunedi");
    }

    #[test]
    fn jour_bissextile_fin_calistril() {
        let c = Calendrier {
            origine: DateGolarion {
                annee: 4724,
                mois: 2,
                jour: 28,
            },
            ..Calendrier::default()
        };
        assert_eq!(
            c.date(2),
            DateGolarion {
                annee: 4724,
                mois: 2,
                jour: 29
            }
        );
        assert_eq!(
            c.date(3),
            DateGolarion {
                annee: 4724,
                mois: 3,
                jour: 1
            }
        );
        let c8 = Calendrier {
            bissextile: RegleBissextile::TousLes8Ans,
            ..c
        };
        assert_eq!(
            c8.date(2),
            DateGolarion {
                annee: 4724,
                mois: 3,
                jour: 1
            }
        );
    }
}
