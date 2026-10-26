//! Calculs de jeu. Chaque fonction cite sa règle (GC = Guide de campagne, page imprimée).

use crate::referentiel::Referentiel;
use domaine::fiches::Unite;
use domaine::{CoutCapitaux, Montant, Ressource, TypeCapital};

/// Jet d'un test : résultat du d20 saisi par le joueur, ou « faire 10 » (GC p. 79, p. 84).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Jet {
    De(u8),
    Prendre10,
}

impl Jet {
    pub fn valeur(self) -> i32 {
        match self {
            Jet::De(d) => d as i32,
            Jet::Prendre10 => 10,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErreurCalcul {
    DeInvalide(u8),
    FacteurInvalide(u32),
    SansCout(String),
    SansDuree(String),
}

impl std::fmt::Display for ErreurCalcul {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ErreurCalcul::DeInvalide(d) => write!(f, "résultat de d20 invalide : {d} (1 à 20)"),
            ErreurCalcul::FacteurInvalide(k) => {
                write!(f, "facteur d'accélération invalide : {k} (2, 3 ou 4)")
            }
            ErreurCalcul::SansCout(n) => {
                write!(f, "« {n} » n'a pas de coût fixe (voir sa description)")
            }
            ErreurCalcul::SansDuree(n) => write!(f, "« {n} » n'a pas de durée fixe"),
        }
    }
}

/// Résultat du test = jet + somme des bonus (GC p. 79 « Travail qualifié », p. 94 « Recettes »).
pub fn resultat_test(jet: Jet, bonus: i32) -> Result<i32, ErreurCalcul> {
    if let Jet::De(d) = jet {
        if !(1..=20).contains(&d) {
            return Err(ErreurCalcul::DeInvalide(d));
        }
    }
    Ok(jet.valeur() + bonus)
}

/// Ce que rapporte un test de capital.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Gain {
    /// Résultat ÷ 10, au pa près : 47 → 4 po 7 pa (GC p. 79, p. 84).
    Po(Montant),
    /// Résultat ÷ 10 arrondi à l'inférieur (table « Production de capitaux », GC p. 79).
    Capital(TypeCapital, u32),
}

pub fn gain(ressource: Ressource, resultat: i32) -> Gain {
    let r = resultat.max(0);
    match ressource.capital() {
        None => Gain::Po(Montant::pa(r as i64)),
        Some(c) => Gain::Capital(c, (r / 10) as u32),
    }
}

/// Coût en po pour produire `quantite` unités : coût produit × quantité (Table 2-1, GC p. 78).
pub fn cout_production(r: &Referentiel, capital: TypeCapital, quantite: u32) -> Montant {
    let v = r
        .valeur(capital.into())
        .expect("valeur de capital officielle");
    Montant::po(v.cout_produit_po as i64 * quantite as i64)
}

/// Valeur d'achat (en po) d'un ensemble de capitaux (Table 2-1, GC p. 78).
pub fn valeur_achat(r: &Referentiel, c: &CoutCapitaux) -> u32 {
    TypeCapital::TOUS
        .iter()
        .map(|t| c.get(*t) * r.valeur((*t).into()).expect("valeur").cout_achat_po)
        .sum()
}

/// Réduction pour jours d'absence du PJ : par tranche de 7 jours, −7 po sur le total en po et
/// −1 par type de capital, minimum 0 (GC p. 84, phase des recettes, 1re étape).
pub fn appliquer_absence(gain_total: Gain, jours_absence: u32) -> Gain {
    let tranches = (jours_absence / 7) as i64;
    match gain_total {
        Gain::Po(m) => Gain::Po(Montant::pc((m.pc - tranches * 700).max(0))),
        Gain::Capital(c, n) => Gain::Capital(c, (n as i64 - tranches).max(0) as u32),
    }
}

/// Coût d'une amélioration : différence positive par capital, différence de durée et de prix
/// (GC p. 95 « Amélioration »). Les différences négatives ne sont pas remboursées.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CoutAmelioration {
    pub capitaux: CoutCapitaux,
    pub prix_po: u32,
    pub duree_jours: u32,
}

pub fn cout_amelioration(depart: &Unite, cible: &Unite) -> Result<CoutAmelioration, ErreurCalcul> {
    let (a, b) = (
        depart
            .cout
            .ok_or_else(|| ErreurCalcul::SansCout(depart.nom.clone()))?,
        cible
            .cout
            .ok_or_else(|| ErreurCalcul::SansCout(cible.nom.clone()))?,
    );
    let mut d = CoutCapitaux::default();
    for t in TypeCapital::TOUS {
        *d.get_mut(t) = b.get(t).saturating_sub(a.get(t));
    }
    Ok(CoutAmelioration {
        capitaux: d,
        prix_po: cible
            .prix_po
            .unwrap_or(0)
            .saturating_sub(depart.prix_po.unwrap_or(0)),
        duree_jours: cible
            .duree_jours
            .unwrap_or(0)
            .saturating_sub(depart.duree_jours.unwrap_or(0)),
    })
}

/// Accélération : durée ÷ k en multipliant par k le Travail (salle) ou l'Influence (équipe),
/// k ∈ {2, 3, 4} (GC p. 95 « Durée »). Arrondi au jour supérieur : point à confirmer PC-10.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Acceleration {
    pub capitaux: CoutCapitaux,
    pub duree_jours: u32,
    pub point_a_confirmer: Option<&'static str>,
}

pub fn accelerer(u: &Unite, est_equipe: bool, k: u32) -> Result<Acceleration, ErreurCalcul> {
    if !(2..=4).contains(&k) {
        return Err(ErreurCalcul::FacteurInvalide(k));
    }
    let mut c = u
        .cout
        .ok_or_else(|| ErreurCalcul::SansCout(u.nom.clone()))?;
    let duree = u
        .duree_jours
        .ok_or_else(|| ErreurCalcul::SansDuree(u.nom.clone()))?;
    let cap = if est_equipe {
        TypeCapital::Influence
    } else {
        TypeCapital::Travail
    };
    *c.get_mut(cap) *= k;
    Ok(Acceleration {
        capitaux: c,
        duree_jours: duree.div_ceil(k),
        point_a_confirmer: (duree % k != 0).then_some("PC-10"),
    })
}

/// Vérifie la limite des dépenses : unités de capital dépensées dans la journée ≤ limite de la
/// communauté (GC p. 81 ; Magie comprise : PC-01 ; po exclues : PC-03).
pub fn depasse_limite(depense_du_jour: &CoutCapitaux, limite: u32) -> Option<u32> {
    let u = depense_du_jour.unites();
    (u > limite).then(|| u - limite)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::referentiel_officiel;

    #[test]
    fn gains_exemples_du_livre() {
        // GC p. 84 : un total de 47 donne 4 po 7 pa
        assert_eq!(gain(Ressource::Po, 47), Gain::Po(Montant::pa(47)));
        assert_eq!(Montant::pa(47).to_string(), "4 po 7 pa");
        // GC p. 79 : 16 → 1 po 6 pa
        assert_eq!(gain(Ressource::Po, 16).to_string_po(), "1 po 6 pa");
        // Table Production de capitaux : 10→1, 39→3, 40→4, 57→5, 9→0
        for (r, n) in [(10, 1), (39, 3), (40, 4), (57, 5), (9, 0)] {
            assert_eq!(
                gain(Ressource::Influence, r),
                Gain::Capital(TypeCapital::Influence, n)
            );
        }
    }

    #[test]
    fn exemple_laura_absence() {
        // GC p. 84 : taverne +15 en po, faire 10, 40 jours d'absence → 100 po − 5×7 po = 65 po
        let par_jour = resultat_test(Jet::Prendre10, 15).unwrap();
        assert_eq!(par_jour, 25);
        let total = Gain::Po(Montant::pa(par_jour as i64 * 40));
        assert_eq!(appliquer_absence(total, 40), Gain::Po(Montant::po(65)));
    }

    #[test]
    fn de_invalide() {
        assert_eq!(
            resultat_test(Jet::De(21), 0),
            Err(ErreurCalcul::DeInvalide(21))
        );
        assert_eq!(
            resultat_test(Jet::De(0), 0),
            Err(ErreurCalcul::DeInvalide(0))
        );
    }

    #[test]
    fn amelioration_exemples_du_livre() {
        let r = referentiel_officiel();
        // GC p. 95 : bibliothèque → bibliothèque magique = 1 M, 1 I, 1 T, 2 Mg ou 270 po, 4 jours
        let a = cout_amelioration(
            r.salle("bibliotheque").unwrap(),
            r.salle("bibliotheque-magique").unwrap(),
        )
        .unwrap();
        assert_eq!(
            a.capitaux,
            CoutCapitaux {
                marchandises: 1,
                influence: 1,
                travail: 1,
                magie: 2
            }
        );
        assert_eq!((a.prix_po, a.duree_jours), (270, 4));
        // GC p. 95 : dortoir → logement = 3 M, 3 T, 6 jours (l'Influence en moins n'est pas rendue)
        let d =
            cout_amelioration(r.salle("dortoir").unwrap(), r.salle("logement").unwrap()).unwrap();
        assert_eq!(
            d.capitaux,
            CoutCapitaux {
                marchandises: 3,
                influence: 0,
                travail: 3,
                magie: 0
            }
        );
        assert_eq!(d.duree_jours, 6);
    }

    #[test]
    fn acceleration() {
        let r = referentiel_officiel();
        let bar = r.salle("bar").unwrap(); // 6 M, 1 I, 5 T ; 16 jours
        let a = accelerer(bar, false, 2).unwrap();
        assert_eq!(
            (a.capitaux.travail, a.duree_jours, a.point_a_confirmer),
            (10, 8, None)
        );
        let a3 = accelerer(bar, false, 3).unwrap();
        assert_eq!(
            (a3.capitaux.travail, a3.duree_jours, a3.point_a_confirmer),
            (15, 6, Some("PC-10"))
        );
        let soldats = r.equipe("soldats").unwrap(); // 2 Influence, 2 jours
        let s = accelerer(soldats, true, 2).unwrap();
        assert_eq!((s.capitaux.influence, s.duree_jours), (4, 1));
        assert!(accelerer(bar, false, 5).is_err());
    }

    #[test]
    fn production_et_limite() {
        let r = referentiel_officiel();
        // GC p. 78 : 3 Influence produites = 45 po
        assert_eq!(
            cout_production(&r, TypeCapital::Influence, 3),
            Montant::po(45)
        );
        let village = r.communaute("village").unwrap();
        let depense = CoutCapitaux {
            marchandises: 6,
            influence: 1,
            travail: 5,
            magie: 0,
        };
        assert_eq!(
            depasse_limite(&depense, village.limite_depense_jour),
            Some(2)
        );
        assert_eq!(depasse_limite(&depense, 15), None);
    }

    impl Gain {
        fn to_string_po(self) -> String {
            match self {
                Gain::Po(m) => m.to_string(),
                Gain::Capital(..) => panic!("pas des po"),
            }
        }
    }
}
