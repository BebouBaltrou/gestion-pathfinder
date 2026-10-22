use std::fmt;

/// Montant en pièces d'argent (1 po = 10 pa), signé pour permettre une trésorerie négative.
///
/// Les gains en po d'un test se comptent à la pièce d'argent près (Guide de campagne
/// p. 79 et p. 84 : 47 → 4 po 7 pa) ; on évite donc tout calcul en flottant.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
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
}
