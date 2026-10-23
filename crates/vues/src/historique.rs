//! Page Historique : journal des événements, du plus récent au plus ancien.

use regles::Referentiel;

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct PageHistorique {
    /// Opérations annulées, qui peuvent être rétablies (la plus récente d'abord).
    pub annulees: Vec<LigneJournal>,
    /// Opérations actives, la plus récente d'abord.
    pub journal: Vec<LigneJournal>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct LigneJournal {
    pub numero: u64,
    pub jour: u32,
    /// « mercantdi 31 Abadius 4712 AR »
    pub date: String,
    pub texte: String,
}

pub fn page_historique(c: &mut campagne::Campagne, r: &Referentiel) -> PageHistorique {
    let annulees: Vec<_> = c.journal[c.curseur..].iter().rev().cloned().collect();
    let actives: Vec<_> = c.historique().cloned().collect();
    let cal = c.calendrier.clone();
    let mut ligne = |e: &campagne::evenements::Entree| LigneJournal {
        numero: e.numero,
        jour: e.jour,
        date: cal.libelle(e.jour),
        texte: c.decrire(&e.evenement, r),
    };
    PageHistorique {
        annulees: annulees.iter().map(&mut ligne).collect(),
        journal: actives.iter().map(&mut ligne).collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use campagne::evenements::Evenement;
    use campagne::Campagne;

    #[test]
    fn journal_et_operations_annulees() {
        let r = regles::referentiel_officiel();
        let mut c = Campagne::nouvelle("Test");
        c.executer(Evenement::JoursAvances { nombre: 2 }, &r)
            .unwrap();
        c.executer(Evenement::JoursAvances { nombre: 1 }, &r)
            .unwrap();
        let p = page_historique(&mut c, &r);
        assert_eq!(p.journal.len(), 2);
        assert!(p.annulees.is_empty());
        assert!(p.journal[0].numero > p.journal[1].numero);
        c.annuler();
        let p = page_historique(&mut c, &r);
        assert_eq!(p.journal.len(), 1);
        assert_eq!(p.annulees.len(), 1);
    }
}
