//! Surcouche de données personnalisées.
//!
//! L'officiel n'est jamais modifié : une fiche personnalisée portant l'id d'une fiche
//! officielle la *surcharge* (les deux restent consultables), une fiche à id nouveau est un
//! *ajout*. Toute consultation renvoie le statut de la valeur retenue.

use crate::referentiel::Referentiel;
use domaine::fiches::{CategorieCommunaute, Gestionnaire, Unite};
use domaine::Statut;
use std::collections::BTreeMap;

#[derive(Debug, Clone, Default)]
pub struct Personnalisations {
    pub salles: BTreeMap<String, Unite>,
    pub equipes: BTreeMap<String, Unite>,
    pub gestionnaires: BTreeMap<String, Gestionnaire>,
    pub communautes: BTreeMap<String, CategorieCommunaute>,
}

/// Résultat d'une consultation : valeur retenue, statut et valeur officielle éventuelle.
#[derive(Debug, Clone, Copy)]
pub struct Entree<'a, T> {
    pub valeur: &'a T,
    pub statut: Statut,
    /// Valeur officielle quand elle existe (y compris si elle est surchargée).
    pub officielle: Option<&'a T>,
}

impl<T> Entree<'_, T> {
    pub fn est_surcharge(&self) -> bool {
        self.statut == Statut::Personnalise && self.officielle.is_some()
    }
}

/// Vue combinée référentiel officiel + personnalisations.
pub struct Catalogue<'a> {
    pub officiel: &'a Referentiel,
    pub perso: &'a Personnalisations,
}

fn statut_officiel(pc: &Option<String>, r: &Referentiel) -> Statut {
    match pc.as_deref().and_then(|c| r.point(c)) {
        Some(p) if p.statut == domaine::fiches::StatutPoint::Ouvert => Statut::AConfirmer,
        _ => Statut::Officiel,
    }
}

fn combine<'a, T>(
    off: Option<&'a T>,
    perso: Option<&'a T>,
    statut_off: impl Fn(&T) -> Statut,
) -> Option<Entree<'a, T>> {
    match (perso, off) {
        (Some(p), o) => Some(Entree {
            valeur: p,
            statut: Statut::Personnalise,
            officielle: o,
        }),
        (None, Some(o)) => Some(Entree {
            valeur: o,
            statut: statut_off(o),
            officielle: Some(o),
        }),
        (None, None) => None,
    }
}

impl<'a> Catalogue<'a> {
    pub fn new(officiel: &'a Referentiel, perso: &'a Personnalisations) -> Self {
        Catalogue { officiel, perso }
    }

    pub fn salle(&self, id: &str) -> Option<Entree<'a, Unite>> {
        combine(self.officiel.salle(id), self.perso.salles.get(id), |u| {
            statut_officiel(&u.point_a_confirmer, self.officiel)
        })
    }

    pub fn equipe(&self, id: &str) -> Option<Entree<'a, Unite>> {
        combine(self.officiel.equipe(id), self.perso.equipes.get(id), |u| {
            statut_officiel(&u.point_a_confirmer, self.officiel)
        })
    }

    pub fn gestionnaire(&self, id: &str) -> Option<Entree<'a, Gestionnaire>> {
        combine(
            self.officiel.gestionnaire(id),
            self.perso.gestionnaires.get(id),
            |_| Statut::Officiel,
        )
    }

    pub fn communaute(&self, id: &str) -> Option<Entree<'a, CategorieCommunaute>> {
        combine(
            self.officiel.communaute(id),
            self.perso.communautes.get(id),
            |c| statut_officiel(&c.point_a_confirmer, self.officiel),
        )
    }

    /// Toutes les salles disponibles (officielles, surchargées ou ajoutées), triées par nom.
    pub fn salles(&self) -> Vec<Entree<'a, Unite>> {
        let mut ids: Vec<&str> = self.officiel.salles.iter().map(|u| u.id.as_str()).collect();
        ids.extend(
            self.perso
                .salles
                .keys()
                .map(String::as_str)
                .filter(|k| self.officiel.salle(k).is_none()),
        );
        let mut v: Vec<_> = ids.into_iter().filter_map(|id| self.salle(id)).collect();
        v.sort_by(|a, b| a.valeur.nom.cmp(&b.valeur.nom));
        v
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn surcharge_sans_ecraser_officiel() {
        let r = crate::referentiel_officiel();
        let mut p = Personnalisations::default();
        let mut bar = r.salle("bar").unwrap().clone();
        bar.prix_po = Some(999);
        p.salles.insert("bar".into(), bar);
        let mut neuve = r.salle("bar").unwrap().clone();
        neuve.id = "bar-a-vins".into();
        neuve.nom = "Bar à vins".into();
        p.salles.insert(neuve.id.clone(), neuve);

        let cat = Catalogue::new(&r, &p);
        let e = cat.salle("bar").unwrap();
        assert_eq!(e.statut, Statut::Personnalise);
        assert!(e.est_surcharge());
        assert_eq!(e.valeur.prix_po, Some(999));
        assert_eq!(e.officielle.unwrap().prix_po, Some(250));
        assert_eq!(r.salle("bar").unwrap().prix_po, Some(250)); // officiel intact

        let a = cat.salle("bar-a-vins").unwrap();
        assert_eq!(a.statut, Statut::Personnalise);
        assert!(a.officielle.is_none());
        assert_eq!(cat.salles().len(), r.salles.len() + 1);

        assert_eq!(cat.salle("cuisine").unwrap().statut, Statut::Officiel);
    }

    #[test]
    fn statut_a_confirmer_pour_point_ouvert() {
        let r = crate::referentiel_officiel();
        let p = Personnalisations::default();
        let cat = Catalogue::new(&r, &p);
        // PC-11 (population de 20) est ouvert → catégorie lieu-dit « à confirmer »
        assert_eq!(
            cat.communaute("lieu-dit").unwrap().statut,
            Statut::AConfirmer
        );
        // PC-07 (Labyrinthe) est validé → officiel
        assert_eq!(cat.salle("labyrinthe").unwrap().statut, Statut::Officiel);
    }
}
