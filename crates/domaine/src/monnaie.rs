use std::fmt;

/// Montant en pièces de cuivre (1 pa = 10 pc, 1 po = 100 pc, 1 pp = 1 000 pc), signé pour
/// permettre une trésorerie négative.
///
/// Les gains en po d'un test se comptent à la pièce d'argent près (Guide de campagne
/// p. 79 et p. 84 : 47 → 4 po 7 pa) et le Manuel des joueurs donne des prix en pièces de
/// cuivre ; on évite donc tout calcul en flottant.
///
/// Les campagnes enregistrées avant l'ajout des pc stockaient `{"pa": n}` : la lecture
/// convertit (n × 10 pc) et l'écriture donne `{"pc": n}`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(try_from = "MontantStocke"))]
pub struct Montant {
    pub pc: i64,
}

/// Forme lue dans un fichier : ancien format en pa ou nouveau format en pc.
#[cfg(feature = "serde")]
#[derive(serde::Deserialize)]
struct MontantStocke {
    pc: Option<i64>,
    pa: Option<i64>,
}

#[cfg(feature = "serde")]
impl TryFrom<MontantStocke> for Montant {
    type Error = String;

    fn try_from(s: MontantStocke) -> Result<Self, String> {
        match (s.pc, s.pa) {
            (Some(pc), None) => Ok(Montant { pc }),
            (None, Some(pa)) => Ok(Montant::pa(pa)),
            _ => Err("montant : « pc » (ou l'ancien « pa ») attendu".to_string()),
        }
    }
}

impl Montant {
    pub const ZERO: Montant = Montant { pc: 0 };

    pub fn pc(pc: i64) -> Self {
        Montant { pc }
    }

    pub fn pa(pa: i64) -> Self {
        Montant { pc: pa * 10 }
    }

    pub fn po(po: i64) -> Self {
        Montant { pc: po * 100 }
    }

    pub fn pp(pp: i64) -> Self {
        Montant { pc: pp * 1000 }
    }

    pub fn est_negatif(&self) -> bool {
        self.pc < 0
    }

    /// Lit un montant saisi : « 12 », « -3 », « 12,5 » ou « 12,25 » (po, deux décimales
    /// au plus), « 4 po 7 pa », « 7 pa », « 5 pc », « 2 pp », « 12 po ». Renvoie `None` si la
    /// saisie est invalide.
    pub fn depuis_texte(s: &str) -> Option<Montant> {
        let t = s.trim().to_lowercase().replace('\u{a0}', " ");
        if t.is_empty() {
            return None;
        }
        let (neg, t) = match t.strip_prefix('-') {
            Some(r) => (true, r.trim().to_string()),
            None => (false, t),
        };
        const UNITES: [(&str, i64); 4] = [("pp", 1000), ("po", 100), ("pa", 10), ("pc", 1)];
        let pc = if UNITES.iter().any(|(u, _)| t.contains(u)) {
            let mut total: i64 = 0;
            let mots: Vec<&str> = t.split_whitespace().collect();
            let mut i = 0;
            let mut vu = false;
            while i < mots.len() {
                let colle = UNITES
                    .iter()
                    .find_map(|(u, _)| mots[i].strip_suffix(u))
                    .filter(|n| !n.is_empty());
                let (nombre, unite) = match colle {
                    Some(n) => (n, &mots[i][n.len()..]),
                    None => {
                        let u = *mots.get(i + 1)?;
                        i += 1;
                        (mots[i - 1], u)
                    }
                };
                let n: i64 = nombre.parse().ok()?;
                total += n * UNITES.iter().find(|(u, _)| *u == unite)?.1;
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
                None => t.parse::<i64>().ok()? * 100,
                Some((e, d)) => {
                    if d.is_empty() || d.len() > 2 || !d.chars().all(|c| c.is_ascii_digit()) {
                        return None;
                    }
                    let e: i64 = if e.is_empty() { 0 } else { e.parse().ok()? };
                    // « 12,5 » = 12 po 5 pa, « 12,25 » = 12 po 2 pa 5 pc.
                    let decimales = if d.len() == 1 {
                        d.parse::<i64>().ok()? * 10
                    } else {
                        d.parse::<i64>().ok()?
                    };
                    e * 100 + decimales
                }
            }
        };
        Some(Montant {
            pc: if neg { -pc } else { pc },
        })
    }
}

impl std::ops::Add for Montant {
    type Output = Montant;
    fn add(self, o: Montant) -> Montant {
        Montant { pc: self.pc + o.pc }
    }
}

impl std::ops::Sub for Montant {
    type Output = Montant;
    fn sub(self, o: Montant) -> Montant {
        Montant { pc: self.pc - o.pc }
    }
}

impl std::ops::Neg for Montant {
    type Output = Montant;
    fn neg(self) -> Montant {
        Montant { pc: -self.pc }
    }
}

impl std::iter::Sum for Montant {
    fn sum<I: Iterator<Item = Montant>>(it: I) -> Montant {
        it.fold(Montant::ZERO, |a, b| a + b)
    }
}

impl fmt::Display for Montant {
    /// « 4 po 7 pa », « -12 po », « 5 pa », « 4 po 7 pa 3 pc ». Les platines s'affichent en po.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let signe = if self.pc < 0 { "-" } else { "" };
        let abs = self.pc.unsigned_abs();
        let (po, pa, pc) = (abs / 100, abs % 100 / 10, abs % 10);
        let mut morceaux = Vec::new();
        if po > 0 {
            morceaux.push(format!("{po} po"));
        }
        if pa > 0 {
            morceaux.push(format!("{pa} pa"));
        }
        if pc > 0 {
            morceaux.push(format!("{pc} pc"));
        }
        if morceaux.is_empty() {
            return f.write_str("0 po");
        }
        write!(f, "{signe}{}", morceaux.join(" "))
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
        assert_eq!(Montant::pc(473).to_string(), "4 po 7 pa 3 pc");
        assert_eq!(Montant::pc(5).to_string(), "5 pc");
        assert_eq!(Montant::pp(2).to_string(), "20 po");
    }

    #[test]
    fn saisie() {
        let m = |s| Montant::depuis_texte(s);
        assert_eq!(m("12"), Some(Montant::po(12)));
        assert_eq!(m(" -3 "), Some(Montant::po(-3)));
        assert_eq!(m("12,5"), Some(Montant::pa(125)));
        assert_eq!(m("12,25"), Some(Montant::pc(1225)));
        assert_eq!(m("0.7"), Some(Montant::pa(7)));
        assert_eq!(m("4 po 7 pa"), Some(Montant::pa(47)));
        assert_eq!(m("4po 7pa"), Some(Montant::pa(47)));
        assert_eq!(m("7 pa"), Some(Montant::pa(7)));
        assert_eq!(m("- 2 po"), Some(Montant::po(-2)));
        assert_eq!(m("4 pc"), Some(Montant::pc(4)));
        assert_eq!(m("2 pp"), Some(Montant::po(20)));
        assert_eq!(m("1 po 2 pa 3 pc"), Some(Montant::pc(123)));
        for bad in ["", "abc", "1,255", "1,", "po", "3 po x"] {
            assert_eq!(m(bad), None, "{bad}");
        }
    }
}
