use std::fmt;

/// Les quatre capitaux d'intermède (Guide de campagne p. 78).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum TypeCapital {
    Marchandises,
    Influence,
    Travail,
    Magie,
}

impl TypeCapital {
    pub const TOUS: [TypeCapital; 4] = [
        TypeCapital::Marchandises,
        TypeCapital::Influence,
        TypeCapital::Travail,
        TypeCapital::Magie,
    ];
}

/// Ce qu'un test de capital peut produire : des po ou un capital.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Ressource {
    Po,
    Marchandises,
    Influence,
    Travail,
    Magie,
}

impl Ressource {
    pub const TOUTES: [Ressource; 5] = [
        Ressource::Po,
        Ressource::Marchandises,
        Ressource::Influence,
        Ressource::Travail,
        Ressource::Magie,
    ];

    pub fn capital(self) -> Option<TypeCapital> {
        match self {
            Ressource::Po => None,
            Ressource::Marchandises => Some(TypeCapital::Marchandises),
            Ressource::Influence => Some(TypeCapital::Influence),
            Ressource::Travail => Some(TypeCapital::Travail),
            Ressource::Magie => Some(TypeCapital::Magie),
        }
    }
}

impl From<TypeCapital> for Ressource {
    fn from(c: TypeCapital) -> Self {
        match c {
            TypeCapital::Marchandises => Ressource::Marchandises,
            TypeCapital::Influence => Ressource::Influence,
            TypeCapital::Travail => Ressource::Travail,
            TypeCapital::Magie => Ressource::Magie,
        }
    }
}

impl fmt::Display for Ressource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Ressource::Po => "po",
            Ressource::Marchandises => "Marchandises",
            Ressource::Influence => "Influence",
            Ressource::Travail => "Travail",
            Ressource::Magie => "Magie",
        })
    }
}

impl fmt::Display for TypeCapital {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Ressource::from(*self).fmt(f)
    }
}

/// Quantités de capitaux (coût de création d'une salle, d'une équipe, d'un modèle…).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct CoutCapitaux {
    pub marchandises: u32,
    pub influence: u32,
    pub travail: u32,
    pub magie: u32,
}

impl CoutCapitaux {
    pub fn get(&self, c: TypeCapital) -> u32 {
        match c {
            TypeCapital::Marchandises => self.marchandises,
            TypeCapital::Influence => self.influence,
            TypeCapital::Travail => self.travail,
            TypeCapital::Magie => self.magie,
        }
    }

    pub fn get_mut(&mut self, c: TypeCapital) -> &mut u32 {
        match c {
            TypeCapital::Marchandises => &mut self.marchandises,
            TypeCapital::Influence => &mut self.influence,
            TypeCapital::Travail => &mut self.travail,
            TypeCapital::Magie => &mut self.magie,
        }
    }

    /// Nombre total d'unités de capital (sert à la limite des dépenses).
    pub fn unites(&self) -> u32 {
        self.marchandises + self.influence + self.travail + self.magie
    }

    pub fn fois(self, n: u32) -> Self {
        CoutCapitaux {
            marchandises: self.marchandises * n,
            influence: self.influence * n,
            travail: self.travail * n,
            magie: self.magie * n,
        }
    }
}

impl std::ops::Add for CoutCapitaux {
    type Output = Self;
    fn add(self, o: Self) -> Self {
        CoutCapitaux {
            marchandises: self.marchandises + o.marchandises,
            influence: self.influence + o.influence,
            travail: self.travail + o.travail,
            magie: self.magie + o.magie,
        }
    }
}

impl fmt::Display for CoutCapitaux {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let parts: Vec<String> = TypeCapital::TOUS
            .iter()
            .filter(|c| self.get(**c) > 0)
            .map(|c| format!("{} {}", self.get(*c), c))
            .collect();
        if parts.is_empty() {
            f.write_str("aucun capital")
        } else {
            f.write_str(&parts.join(", "))
        }
    }
}
