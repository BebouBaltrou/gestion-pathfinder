//! Génération des caractéristiques (Core Rulebook p. 15, « Generating Ability Scores »).
//! Les ajustements raciaux s'appliquent après la génération.

/// Méthodes du livre, plus la saisie libre.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Methode {
    /// Achat de points (tableau 1-1) : 10 (faible), 15 (standard), 20 (élevée), 25 (épique).
    Achat {
        points: u32,
    },
    /// « Standard » : 4d6, on retire le plus bas, six fois, à répartir librement.
    Standard,
    /// « Classic » : 3d6 six fois.
    Classique,
    /// « Heroic » : 2d6 + 6 six fois.
    Heroique,
    /// « Dice Pool » : réserve de dés (24 par défaut, 28 pour une partie plus puissante).
    ReserveDeDes {
        des: u32,
    },
    Libre,
}

/// Coût d'achat d'un score (tableau 1-1, p. 15), de 7 à 18.
pub fn cout_achat(score: i32) -> Option<i32> {
    Some(match score {
        7 => -4,
        8 => -2,
        9 => -1,
        10 => 0,
        11 => 1,
        12 => 2,
        13 => 3,
        14 => 5,
        15 => 7,
        16 => 10,
        17 => 13,
        18 => 17,
        _ => return None,
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErreurAchat {
    /// Aucun score ne peut être sous 7 ni au-dessus de 18 avec cette méthode.
    ScoreHorsLimites {
        score: i32,
    },
    PointsDepasses {
        depenses: i32,
        disponibles: i32,
    },
}

impl std::fmt::Display for ErreurAchat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ErreurAchat::ScoreHorsLimites { score } => {
                write!(
                    f,
                    "score {score} impossible : entre 7 et 18 avec l'achat de points"
                )
            }
            ErreurAchat::PointsDepasses {
                depenses,
                disponibles,
            } => write!(
                f,
                "{depenses} points dépensés pour {disponibles} disponibles"
            ),
        }
    }
}

/// Points dépensés par ces six scores (un score bas rend des points). Vérifie la limite.
pub fn verifier_achat(scores: &[i32; 6], points: u32) -> Result<i32, ErreurAchat> {
    let mut total = 0;
    for &s in scores {
        total += cout_achat(s).ok_or(ErreurAchat::ScoreHorsLimites { score: s })?;
    }
    if total > points as i32 {
        return Err(ErreurAchat::PointsDepasses {
            depenses: total,
            disponibles: points as i32,
        });
    }
    Ok(total)
}

/// Source de dés : l'interface fournit des vrais tirages, les tests des tirages fixes.
pub trait Des {
    fn d(&mut self, faces: u32) -> u32;
}

/// Générateur simple (xorshift) pour les tirages de l'application.
pub struct Aleatoire(pub u64);

impl Des for Aleatoire {
    fn d(&mut self, faces: u32) -> u32 {
        let mut x = self.0.max(1);
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        (x % faces as u64) as u32 + 1
    }
}

fn quatre_d6_sans_le_plus_bas(des: &mut dyn Des) -> i32 {
    let mut v = [des.d(6), des.d(6), des.d(6), des.d(6)];
    v.sort_unstable();
    (v[1] + v[2] + v[3]) as i32
}

/// Six totaux à répartir, selon la méthode aléatoire choisie.
pub fn tirer(methode: Methode, des: &mut dyn Des) -> Option<[i32; 6]> {
    let mut out = [0; 6];
    for o in out.iter_mut() {
        *o = match methode {
            Methode::Standard => quatre_d6_sans_le_plus_bas(des),
            Methode::Classique => (des.d(6) + des.d(6) + des.d(6)) as i32,
            Methode::Heroique => (des.d(6) + des.d(6)) as i32 + 6,
            _ => return None,
        };
    }
    Some(out)
}

/// « Dice Pool » : chaque score reçoit au moins 3d6 ; on garde les trois meilleurs dés.
/// `repartition` donne le nombre de dés par score ; la somme doit être celle de la réserve.
pub fn tirer_reserve(
    reserve: u32,
    repartition: [u32; 6],
    des: &mut dyn Des,
) -> Result<[i32; 6], String> {
    if repartition.iter().sum::<u32>() != reserve {
        return Err(format!(
            "la répartition doit utiliser exactement {reserve} dés"
        ));
    }
    if repartition.iter().any(|&n| n < 3) {
        return Err("au moins 3 dés par caractéristique".into());
    }
    let mut out = [0; 6];
    for (o, &n) in out.iter_mut().zip(repartition.iter()) {
        let mut v: Vec<u32> = (0..n).map(|_| des.d(6)).collect();
        v.sort_unstable_by(|a, b| b.cmp(a));
        *o = v.iter().take(3).sum::<u32>() as i32;
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Dés fixes, dans l'ordre.
    struct Fixes(Vec<u32>, usize);
    impl Des for Fixes {
        fn d(&mut self, _: u32) -> u32 {
            let v = self.0[self.1];
            self.1 += 1;
            v
        }
    }

    #[test]
    fn tableau_des_couts_du_livre() {
        let attendu = [-4, -2, -1, 0, 1, 2, 3, 5, 7, 10, 13, 17];
        for (i, c) in attendu.iter().enumerate() {
            assert_eq!(cout_achat(7 + i as i32), Some(*c));
        }
        assert_eq!(cout_achat(6), None);
        assert_eq!(cout_achat(19), None);
    }

    #[test]
    fn achat_quinze_points() {
        // 8 (-2) + 14 (5) + 12 (2) + 10 + 10 + 16 (10) = 15
        assert_eq!(verifier_achat(&[8, 14, 12, 10, 10, 16], 15), Ok(15));
        assert_eq!(
            verifier_achat(&[8, 14, 12, 10, 10, 17], 15),
            Err(ErreurAchat::PointsDepasses {
                depenses: 18,
                disponibles: 15
            })
        );
        assert_eq!(
            verifier_achat(&[6, 10, 10, 10, 10, 10], 15),
            Err(ErreurAchat::ScoreHorsLimites { score: 6 })
        );
    }

    #[test]
    fn standard_retire_le_plus_bas() {
        let mut d = Fixes(
            vec![
                1, 5, 6, 4, /**/ 6, 6, 6, 6, 1, 1, 1, 1, 2, 3, 4, 5, 3, 3, 3, 1, 3, 3, 3, 1,
            ],
            0,
        );
        let t = tirer(Methode::Standard, &mut d).unwrap();
        assert_eq!(t, [15, 18, 3, 12, 9, 9]);
    }

    #[test]
    fn classique_et_heroique() {
        let mut d = Fixes(vec![3; 18], 0);
        assert_eq!(tirer(Methode::Classique, &mut d), Some([9; 6]));
        let mut d = Fixes(vec![3; 12], 0);
        assert_eq!(tirer(Methode::Heroique, &mut d), Some([12; 6]));
        assert_eq!(tirer(Methode::Libre, &mut Fixes(vec![], 0)), None);
    }

    #[test]
    fn reserve_de_des() {
        // 24 dés : 4 pour le premier score, 4 pour le second, 3 pour les autres, 4 en tout…
        let rep = [4, 4, 4, 4, 4, 4];
        let mut tirages = vec![];
        for _ in 0..6 {
            tirages.extend([6, 5, 4, 1]);
        }
        let r = tirer_reserve(24, rep, &mut Fixes(tirages, 0)).unwrap();
        assert_eq!(r, [15; 6]);
        assert!(tirer_reserve(24, [3, 3, 3, 3, 3, 3], &mut Fixes(vec![], 0)).is_err());
        assert!(tirer_reserve(24, [2, 5, 5, 4, 4, 4], &mut Fixes(vec![], 0)).is_err());
    }

    #[test]
    fn aleatoire_reste_dans_les_bornes() {
        let mut a = Aleatoire(42);
        for _ in 0..1000 {
            let v = a.d(6);
            assert!((1..=6).contains(&v));
        }
    }
}
