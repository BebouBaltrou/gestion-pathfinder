use std::fmt;

/// Montant en pièces d'argent (1 po = 10 pa), signé pour permettre une trésorerie négative.
///
/// Les gains en po d'un test se comptent à la pièce d'argent près (Guide de campagne
/// p. 79 et p. 84 : 47 → 4 po 7 pa) ; on évite donc tout calcul en flottant.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Montant {
    pub pa: i64,
}

impl Montant {
    pub const ZERO: Montant = Montant { pa: 0 };

    pub fn po(po: i64) -> Self {
        Montant { pa: po * 10 }
    }

    pub fn pa(pa: i64) -> Self {
        Montant { pa }
    }

    pub fn est_negatif(&self) -> bool {
        self.pa < 0
    }

    /// Lit un montant saisi : « 12 », « -3 », « 12,5 » ou « 12.5 » (po, une décimale = pa),
    /// « 4 po 7 pa », « 7 pa », « 12 po ». Renvoie `None` si la saisie est invalide.
    pub fn depuis_texte(s: &str) -> Option<Montant> {
        let t = s.trim().to_lowercase().replace('\u{a0}', " ");
        if t.is_empty() {
            return None;
        }
        let (neg, t) = match t.strip_prefix('-') {
            Some(r) => (true, r.trim().to_string()),
            None => (false, t),
        };
        let pa = if t.contains("po") || t.contains("pa") {
            let mut total: i64 = 0;
            let mots: Vec<&str> = t.split_whitespace().collect();
            let mut i = 0;
            let mut vu = false;
            while i < mots.len() {
                let (nombre, unite) = match mots[i]
                    .strip_suffix("po")
                    .or_else(|| mots[i].strip_suffix("pa"))
                {
                    Some(n) if !n.is_empty() => (n, &mots[i][n.len()..]),
                    _ => {
                        let u = *mots.get(i + 1)?;
                        i += 1;
                        (mots[i - 1], u)
                    }
                };
                let n: i64 = nombre.parse().ok()?;
                total += match unite {
                    "po" => n * 10,
                    "pa" => n,
                    _ => return None,
                };
                vu = true;
                i += 1;
            }
            if !vu {
                return None;
            }
            total
        } else {
            let t = t.replace(',', ".");
            match t.split_once('.') {
                None => t.parse::<i64>().ok()? * 10,
                Some((e, d)) => {
                    if d.len() != 1 || !d.chars().all(|c| c.is_ascii_digit()) {
                        return None;
                    }
                    let e: i64 = if e.is_empty() { 0 } else { e.parse().ok()? };
                    e * 10 + d.parse::<i64>().ok()?
                }
            }
        };
        Some(Montant {
            pa: if neg { -pa } else { pa },
        })
    }
}

impl std::ops::Add for Montant {
    type Output = Montant;
    fn add(self, o: Montant) -> Montant {
        Montant { pa: self.pa + o.pa }
    }
}

impl std::ops::Sub for Montant {
    type Output = Montant;
    fn sub(self, o: Montant) -> Montant {
        Montant { pa: self.pa - o.pa }
    }
}

impl std::ops::Neg for Montant {
    type Output = Montant;
    fn neg(self) -> Montant {
        Montant { pa: -self.pa }
    }
}

impl std::iter::Sum for Montant {
    fn sum<I: Iterator<Item = Montant>>(it: I) -> Montant {
        it.fold(Montant::ZERO, |a, b| a + b)
    }
}

impl fmt::Display for Montant {
    /// « 4 po 7 pa », « -12 po », « 5 pa ».
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let signe = if self.pa < 0 { "-" } else { "" };
        let abs = self.pa.unsigned_abs();
        let (po, pa) = (abs / 10, abs % 10);
        match (po, pa) {
            (0, 0) => f.write_str("0 po"),
            (0, pa) => write!(f, "{signe}{pa} pa"),
            (po, 0) => write!(f, "{signe}{po} po"),
            (po, pa) => write!(f, "{signe}{po} po {pa} pa"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn affichage() {
        assert_eq!(Montant::pa(47).to_string(), "4 po 7 pa");
        assert_eq!(Montant::po(-12).to_string(), "-12 po");
        assert_eq!(Montant::pa(5).to_string(), "5 pa");
        assert_eq!(Montant::ZERO.to_string(), "0 po");
        assert_eq!(Montant::pa(-3).to_string(), "-3 pa");
    }

    #[test]
    fn saisie() {
        let m = |s| Montant::depuis_texte(s);
        assert_eq!(m("12"), Some(Montant::po(12)));
        assert_eq!(m(" -3 "), Some(Montant::po(-3)));
        assert_eq!(m("12,5"), Some(Montant::pa(125)));
        assert_eq!(m("0.7"), Some(Montant::pa(7)));
        assert_eq!(m("4 po 7 pa"), Some(Montant::pa(47)));
        assert_eq!(m("4po 7pa"), Some(Montant::pa(47)));
        assert_eq!(m("7 pa"), Some(Montant::pa(7)));
        assert_eq!(m("- 2 po"), Some(Montant::po(-2)));
        for bad in ["", "abc", "1,25", "4 pc", "po", "3 po x"] {
            assert_eq!(m(bad), None, "{bad}");
        }
    }
}
