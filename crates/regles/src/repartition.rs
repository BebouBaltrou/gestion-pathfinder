//! Affectation des bonus des salles et équipes aux ressources produites, puis test par ressource.
//!
//! Officiel (GC p. 94) : le bonus d'une unité peut aller entier à un type admis ou être réparti
//! entre plusieurs types admis. Personnalisé (option `repartition_stricte`) : un seul type par
//! unité et par jour. Des unités de plusieurs bâtiments/organisations de la même communauté
//! peuvent être additionnées dans un seul test par type (GC p. 94).

use crate::calculs::{gain, resultat_test, ErreurCalcul, Gain, Jet};
use crate::options::OptionsRegles;
use domaine::fiches::{Admissibles, Unite};
use domaine::Ressource;
use std::collections::{BTreeMap, BTreeSet};

/// Une unité qui contribue au test, avec son contexte.
#[derive(Debug, Clone)]
pub struct Contribution<'a> {
    /// Libellé affiché, ex. « Auberge du Pont — Bar ».
    pub libelle: String,
    pub unite: &'a Unite,
    /// Ressources admises par les *autres* unités du même bâtiment (pour « TypesDuBatiment », PC-05).
    pub types_du_batiment: BTreeSet<Ressource>,
    /// Équipe sans bâtiment pour travailler : recettes ÷ 2 (GC p. 94), sauf option contraire.
    pub sans_batiment: bool,
    /// Salle brisée : recettes ÷ 2 (GC p. 92).
    pub brisee: bool,
    /// Équipe en préparation de délocalisation : moitié des avantages (GC p. 95).
    pub delocalisation: bool,
    /// Points affectés à chaque ressource. Somme = bonus effectif.
    pub affectation: Vec<(Ressource, u32)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErreurRepartition {
    SansRecettes(String),
    RessourceNonAdmise {
        libelle: String,
        ressource: Ressource,
    },
    SommeIncorrecte {
        libelle: String,
        attendu: u32,
        affecte: u32,
    },
    PlusieursTypes(String),
    Calcul(ErreurCalcul),
}

impl std::fmt::Display for ErreurRepartition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ErreurRepartition::SansRecettes(l) => write!(f, "{l} ne produit aucune recette"),
            ErreurRepartition::RessourceNonAdmise { libelle, ressource } => {
                write!(f, "{libelle} ne peut pas produire de {ressource}")
            }
            ErreurRepartition::SommeIncorrecte {
                libelle,
                attendu,
                affecte,
            } => {
                write!(
                    f,
                    "{libelle} : {affecte} points affectés pour un bonus de {attendu}"
                )
            }
            ErreurRepartition::PlusieursTypes(l) => {
                write!(
                    f,
                    "{l} : un seul type par jour (règle personnalisée « répartition stricte »)"
                )
            }
            ErreurRepartition::Calcul(e) => e.fmt(f),
        }
    }
}

/// Ressources admises par une unité dans son contexte.
pub fn admises(u: &Unite, types_du_batiment: &BTreeSet<Ressource>) -> BTreeSet<Ressource> {
    match u.recettes.as_ref().map(|r| &r.admissibles) {
        None => BTreeSet::new(),
        Some(Admissibles::Liste { ressources }) => ressources.iter().copied().collect(),
        Some(Admissibles::Toutes) => Ressource::TOUTES.into_iter().collect(),
        Some(Admissibles::TypesDuBatiment) => types_du_batiment.clone(),
        // Ameublement : suit la salle hôte, résolu par l'appelant via `types_du_batiment`.
        Some(Admissibles::SalleHote) => types_du_batiment.clone(),
    }
}

/// Bonus après réductions. Chaque division par 2 arrondit à l'inférieur (PC-12, validé).
pub fn bonus_effectif(c: &Contribution, options: &OptionsRegles) -> u32 {
    let mut b = c.unite.recettes.as_ref().map(|r| r.bonus).unwrap_or(0);
    if c.sans_batiment && options.moitie_equipe_sans_batiment {
        b /= 2;
    }
    if c.brisee {
        b /= 2;
    }
    if c.delocalisation {
        b /= 2;
    }
    b
}

/// Détail d'un test, prêt à afficher et à historiser.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DetailTest {
    pub ressource: Ressource,
    pub jet: i32,
    pub prendre10: bool,
    pub lignes: Vec<(String, u32)>,
    pub total_bonus: u32,
    pub resultat: i32,
    pub gain: Gain,
    pub formule: String,
}

/// Valide les affectations et calcule un test par ressource ayant au moins un point affecté.
pub fn tests_du_jour(
    contributions: &[Contribution],
    jets: &BTreeMap<Ressource, Jet>,
    options: &OptionsRegles,
) -> Result<Vec<DetailTest>, ErreurRepartition> {
    let mut par_ressource: BTreeMap<Ressource, Vec<(String, u32)>> = BTreeMap::new();
    for c in contributions {
        if c.unite.recettes.is_none() {
            return Err(ErreurRepartition::SansRecettes(c.libelle.clone()));
        }
        let ok = admises(c.unite, &c.types_du_batiment);
        let attendu = bonus_effectif(c, options);
        let utiles: Vec<_> = c.affectation.iter().filter(|(_, p)| *p > 0).collect();
        if options.repartition_stricte && utiles.len() > 1 {
            return Err(ErreurRepartition::PlusieursTypes(c.libelle.clone()));
        }
        let somme: u32 = utiles.iter().map(|(_, p)| p).sum();
        if somme != attendu {
            return Err(ErreurRepartition::SommeIncorrecte {
                libelle: c.libelle.clone(),
                attendu,
                affecte: somme,
            });
        }
        for (r, p) in utiles {
            if !ok.contains(r) {
                return Err(ErreurRepartition::RessourceNonAdmise {
                    libelle: c.libelle.clone(),
                    ressource: *r,
                });
            }
            par_ressource
                .entry(*r)
                .or_default()
                .push((c.libelle.clone(), *p));
        }
    }
    par_ressource
        .into_iter()
        .map(|(ressource, lignes)| {
            let jet = jets.get(&ressource).copied().unwrap_or(Jet::Prendre10);
            let total_bonus: u32 = lignes.iter().map(|(_, p)| p).sum();
            let resultat =
                resultat_test(jet, total_bonus as i32).map_err(ErreurRepartition::Calcul)?;
            let g = gain(ressource, resultat);
            let formule = match g {
                Gain::Po(m) => format!(
                    "({} + {}) ÷ 10 = {} (au pa près)",
                    jet.valeur(),
                    total_bonus,
                    m
                ),
                Gain::Capital(c, n) => {
                    format!(
                        "({} + {}) ÷ 10 = {} {} (arrondi inférieur)",
                        jet.valeur(),
                        total_bonus,
                        n,
                        c
                    )
                }
            };
            Ok(DetailTest {
                ressource,
                jet: jet.valeur(),
                prendre10: jet == Jet::Prendre10,
                lignes,
                total_bonus,
                resultat,
                gain: g,
                formule,
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::referentiel_officiel;
    use domaine::Montant;

    fn contrib<'a>(
        r: &'a crate::Referentiel,
        id: &str,
        aff: Vec<(Ressource, u32)>,
    ) -> Contribution<'a> {
        Contribution {
            libelle: format!("Auberge — {id}"),
            unite: r.salle(id).unwrap(),
            types_du_batiment: BTreeSet::new(),
            sans_batiment: false,
            brisee: false,
            delocalisation: false,
            affectation: aff,
        }
    }

    #[test]
    fn exemple_auberge_du_livre() {
        // GC p. 94 : bar 10 + salle commune 7 + cuisine 4 + logement 12 + stabulation 8 = +41 en po
        let r = referentiel_officiel();
        let cs = vec![
            contrib(&r, "bar", vec![(Ressource::Po, 10)]),
            contrib(&r, "salle-commune", vec![(Ressource::Po, 7)]),
            contrib(&r, "cuisine", vec![(Ressource::Po, 4)]),
            contrib(&r, "logement", vec![(Ressource::Po, 12)]),
            contrib(&r, "stabulation", vec![(Ressource::Po, 8)]),
        ];
        let jets = BTreeMap::from([(Ressource::Po, Jet::Prendre10)]);
        let t = tests_du_jour(&cs, &jets, &OptionsRegles::default()).unwrap();
        assert_eq!(t.len(), 1);
        assert_eq!(t[0].total_bonus, 41);
        assert_eq!(t[0].gain, Gain::Po(Montant::pa(51)));

        // Variante du livre : bar en Influence (+10), le reste en po (+31)
        let mut cs2 = cs.clone();
        cs2[0].affectation = vec![(Ressource::Influence, 10)];
        let jets2 = BTreeMap::from([
            (Ressource::Po, Jet::De(12)),
            (Ressource::Influence, Jet::De(5)),
        ]);
        let t2 = tests_du_jour(&cs2, &jets2, &OptionsRegles::default()).unwrap();
        let po = t2.iter().find(|d| d.ressource == Ressource::Po).unwrap();
        let inf = t2
            .iter()
            .find(|d| d.ressource == Ressource::Influence)
            .unwrap();
        assert_eq!((po.total_bonus, po.resultat), (31, 43));
        assert_eq!(inf.gain, Gain::Capital(domaine::TypeCapital::Influence, 1));
    }

    #[test]
    fn repartition_stricte_et_officielle() {
        let r = referentiel_officiel();
        // GC p. 94 : laboratoire d'alchimie +10 réparti +5 po / +5 Marchandises
        let c = vec![Contribution {
            libelle: "Labo".into(),
            unite: r.salle("laboratoire-d-alchimie").unwrap(),
            types_du_batiment: BTreeSet::new(),
            sans_batiment: false,
            brisee: false,
            delocalisation: false,
            affectation: vec![(Ressource::Po, 5), (Ressource::Marchandises, 5)],
        }];
        let jets = BTreeMap::new();
        let stricte = OptionsRegles::default();
        assert_eq!(
            tests_du_jour(&c, &jets, &stricte),
            Err(ErreurRepartition::PlusieursTypes("Labo".into()))
        );
        let officielle = OptionsRegles {
            repartition_stricte: false,
            ..OptionsRegles::default()
        };
        assert_eq!(tests_du_jour(&c, &jets, &officielle).unwrap().len(), 2);
    }

    #[test]
    fn controles_d_affectation() {
        let r = referentiel_officiel();
        let mut c = contrib(&r, "bar", vec![(Ressource::Magie, 10)]);
        let opt = OptionsRegles::default();
        assert!(matches!(
            tests_du_jour(&[c.clone()], &BTreeMap::new(), &opt),
            Err(ErreurRepartition::RessourceNonAdmise { .. })
        ));
        c.affectation = vec![(Ressource::Po, 9)];
        assert!(matches!(
            tests_du_jour(&[c], &BTreeMap::new(), &opt),
            Err(ErreurRepartition::SommeIncorrecte { .. })
        ));

        // Équipe sans bâtiment : moitié du bonus (soldats +5 → 2)
        let s = Contribution {
            libelle: "Soldats".into(),
            unite: r.equipe("soldats").unwrap(),
            types_du_batiment: BTreeSet::new(),
            sans_batiment: true,
            brisee: false,
            delocalisation: false,
            affectation: vec![(Ressource::Travail, 2)],
        };
        assert_eq!(
            tests_du_jour(&[s], &BTreeMap::new(), &opt).unwrap()[0].total_bonus,
            2
        );

        // Option : malus des équipes sans bâtiment désactivé → bonus entier (+5)
        let sans_malus = OptionsRegles {
            moitie_equipe_sans_batiment: false,
            ..OptionsRegles::default()
        };
        let s2 = Contribution {
            libelle: "Soldats".into(),
            unite: r.equipe("soldats").unwrap(),
            types_du_batiment: BTreeSet::new(),
            sans_batiment: true,
            brisee: false,
            delocalisation: false,
            affectation: vec![(Ressource::Travail, 5)],
        };
        assert_eq!(
            tests_du_jour(&[s2], &BTreeMap::new(), &sans_malus).unwrap()[0].total_bonus,
            5
        );

        // Étalage : seulement un type que le bâtiment génère déjà (PC-05)
        let e = Contribution {
            libelle: "Étalage".into(),
            unite: r.salle("etalage").unwrap(),
            types_du_batiment: BTreeSet::from([Ressource::Po, Ressource::Influence]),
            sans_batiment: false,
            brisee: false,
            delocalisation: false,
            affectation: vec![(Ressource::Magie, 5)],
        };
        assert!(tests_du_jour(&[e], &BTreeMap::new(), &opt).is_err());
    }
}
