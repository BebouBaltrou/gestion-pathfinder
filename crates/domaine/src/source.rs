use std::fmt;

/// Référence consultable d'une donnée officielle.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Source {
    /// Code du livre (voir la liste `sources` du référentiel), ex. « GC ».
    pub livre: String,
    /// Page imprimée du livre ; `None` si la donnée vient d'une page web seulement.
    pub page: Option<u32>,
    pub section: String,
}

impl fmt::Display for Source {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.page {
            Some(p) => write!(f, "{} p. {} — {}", self.livre, p, self.section),
            None => write!(f, "{} — {}", self.livre, self.section),
        }
    }
}

/// Statut d'une donnée ou d'une règle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Statut {
    /// Écrite dans la source citée.
    Officiel,
    /// Définie par l'utilisateur ; ne remplace jamais silencieusement l'officiel.
    Personnalise,
    /// Source ambiguë : appliquée avec un avertissement, liée à un point à confirmer.
    AConfirmer,
}
