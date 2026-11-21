//! Statistiques : patrimoine des personnages et des localités, effectif et valeur des équipes des
//! organisations (prix imprimés, GC). Les capitaux sont comptés à leur valeur d'achat
//! (Table 2-1, GC p. 78).
//!
//! # Ajouter une nouvelle sorte de patrimoine (montures, véhicules…)
//!
//! Le patrimoine est une liste d'`ElementPatrimoine`, produite par des *sources*. Tout le reste
//! (total, répartition, personnages, villes, courbes dans le temps, graphiques) s'en déduit.
//! Pour ajouter une sorte de patrimoine :
//! 1. écrire une fonction `fn source_xxx(&Etat, &Referentiel) -> Vec<ElementPatrimoine>` ;
//! 2. la déclarer dans `SOURCES` et sa catégorie dans `CATEGORIES`.
//!
//! Un élément indique son propriétaire (personnage) et/ou sa ville ; il compte une seule fois dans
//! le total, et chez chacun de ses rattachements dans les détails.

use crate::inventaire::page_inventaire;
use crate::Somme;
use campagne::equipement_fiche::{personnage_de_ref, PROPRIETAIRE_PERSONNAGE};
use campagne::etat::Etat;
use campagne::modele::*;
use domaine::Montant;
use regles::calculs::valeur_achat;
use regles::Referentiel;

/// Une valeur de patrimoine, en pièces de cuivre.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ElementPatrimoine {
    /// Clé d'une catégorie de `CATEGORIES`.
    pub categorie: &'static str,
    pub proprietaire: Option<IdPersonnage>,
    pub localite: Option<IdLocalite>,
    pub pc: i64,
}

pub type SourcePatrimoine = fn(&Etat, &Referentiel) -> Vec<ElementPatrimoine>;

/// Catégories affichées, dans l'ordre : (clé, nom).
pub const CATEGORIES: &[(&str, &str)] = &[
    ("comptes", "Comptes des PJ"),
    ("objets", "Objets"),
    ("batiments", "Bâtiments"),
    ("tresoreries", "Trésoreries"),
    ("capitaux", "Capitaux"),
    ("vehicules", "Véhicules"),
    ("montures", "Montures"),
];

/// Sources du patrimoine : une fonction par sorte de patrimoine.
pub const SOURCES: &[SourcePatrimoine] = &[
    source_comptes,
    source_objets,
    source_batiments,
    source_tresoreries,
    source_capitaux,
    source_vehicules,
    source_montures,
];

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct PageStatistiques {
    /// Parts du patrimoine total, une par catégorie.
    pub repartition: Vec<PartPatrimoine>,
    /// Patrimoine total : chaque élément compte une seule fois.
    pub patrimoine_total: Somme,
    pub personnages: Vec<PatrimoinePersonnage>,
    pub total_personnages: Somme,
    pub localites: Vec<PatrimoineLocalite>,
    pub total_localites: Somme,
    /// Éléments sans propriétaire ni ville (par exemple les objets d'un inventaire sans PJ).
    pub non_attribue: Somme,
    pub batiments: Vec<PatrimoineBatiment>,
    pub total_batiments: Somme,
    pub organisations: Vec<StatOrganisation>,
    pub effectif_total: u32,
    pub effectif_a_venir_total: u32,
    /// Membres de la suite (partisans du don Prestige), comptés à part des autres effectifs.
    pub effectif_suite_total: u32,
    pub valeur_equipes: Somme,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct PartPatrimoine {
    pub cle: String,
    pub nom: String,
    pub valeur: Somme,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct PatrimoinePersonnage {
    pub id: u32,
    pub nom: String,
    /// Une part par catégorie (comptes, objets, bâtiments possédés…).
    pub composantes: Vec<PartPatrimoine>,
    pub total: Somme,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct PatrimoineLocalite {
    pub id: u32,
    pub nom: String,
    /// Une part par catégorie (trésorerie, capitaux, bâtiments de la ville…).
    pub composantes: Vec<PartPatrimoine>,
    pub total: Somme,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct PatrimoineBatiment {
    pub id: u32,
    pub nom: String,
    pub localite: String,
    pub proprietaire: Option<String>,
    /// Salles construites.
    pub salles: u32,
    /// Prix d'achat des salles et extensions construites (référentiel, GC).
    pub valeur: Somme,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct StatOrganisation {
    pub id: u32,
    pub nom: String,
    pub localite: String,
    pub equipes: u32,
    /// Personnes en place (recrutées, en amélioration ou en délocalisation).
    pub effectif: u32,
    /// Personnes à recruter ou en recrutement.
    pub effectif_a_venir: u32,
    /// Membres de la suite en place, comptés à part.
    pub effectif_suite: u32,
    pub gestionnaires: u32,
    /// Somme des prix imprimés (`prix_po`, référentiel) des équipes de l'organisation.
    pub valeur_equipes: Somme,
}

fn somme(pc: i64) -> Somme {
    Somme::from(Montant::pc(pc))
}

// ---------------------------------------------------------------- sources

fn source_comptes(etat: &Etat, _r: &Referentiel) -> Vec<ElementPatrimoine> {
    etat.personnages
        .values()
        .map(|p| ElementPatrimoine {
            categorie: "comptes",
            proprietaire: Some(p.id),
            localite: None,
            pc: p.compte.pc,
        })
        .collect()
}

fn source_objets(etat: &Etat, _r: &Referentiel) -> Vec<ElementPatrimoine> {
    page_inventaire(etat)
        .inventaires
        .iter()
        .map(|i| ElementPatrimoine {
            categorie: "objets",
            proprietaire: if i.type_proprietaire == PROPRIETAIRE_PERSONNAGE {
                personnage_de_ref(&i.ref_proprietaire).filter(|p| etat.personnages.contains_key(p))
            } else {
                None
            },
            localite: None,
            pc: i.valeur.pc,
        })
        .collect()
}

/// Prix d'achat des salles et extensions construites d'un bâtiment, et nombre de salles construites.
fn valeur_batiment(etat: &Etat, r: &Referentiel, b: IdBatiment) -> (i64, u32) {
    let mut valeur = 0i64;
    let mut salles = 0u32;
    for s in etat.salles_batiment(b) {
        if matches!(s.statut, StatutSalle::Construite) {
            salles += 1;
            valeur += r
                .salle(&s.reference)
                .and_then(|u| u.prix_po)
                .map_or(0, |po| i64::from(po) * 100);
        }
        for x in &s.extensions {
            if matches!(x.statut, StatutExtension::Construite) {
                valeur += i64::from(x.prix_po) * 100;
            }
        }
    }
    (valeur, salles)
}

fn source_batiments(etat: &Etat, r: &Referentiel) -> Vec<ElementPatrimoine> {
    etat.batiments
        .values()
        .map(|b| ElementPatrimoine {
            categorie: "batiments",
            // un propriétaire disparu ne compte plus
            proprietaire: etat
                .proprietaires_batiments
                .get(&b.id)
                .copied()
                .filter(|p| etat.personnages.contains_key(p)),
            localite: Some(b.localite),
            pc: valeur_batiment(etat, r, b.id).0,
        })
        .collect()
}

fn source_tresoreries(etat: &Etat, _r: &Referentiel) -> Vec<ElementPatrimoine> {
    etat.localites
        .values()
        .map(|l| ElementPatrimoine {
            categorie: "tresoreries",
            proprietaire: None,
            localite: Some(l.id),
            pc: l.tresorerie.pc,
        })
        .collect()
}

fn source_capitaux(etat: &Etat, r: &Referentiel) -> Vec<ElementPatrimoine> {
    etat.localites
        .values()
        .map(|l| ElementPatrimoine {
            categorie: "capitaux",
            proprietaire: None,
            localite: Some(l.id),
            pc: i64::from(valeur_achat(r, &l.capitaux)) * 100,
        })
        .collect()
}

/// Véhicules, à leur prix d'achat : chez le PJ propriétaire, ou dans la ville de la localité
/// ou du siège de l'organisation propriétaire.
fn source_vehicules(etat: &Etat, _r: &Referentiel) -> Vec<ElementPatrimoine> {
    use campagne::biens::ProprietaireBien;
    etat.vehicules
        .values()
        .map(|v| {
            let (proprietaire, localite) = match v.proprietaire {
                ProprietaireBien::Localite(l) => (None, Some(l)),
                ProprietaireBien::Organisation(o) => {
                    (None, etat.organisations.get(&o).map(|o| o.localite))
                }
                ProprietaireBien::Personnage(p) => (Some(p), None),
            };
            ElementPatrimoine {
                categorie: "vehicules",
                proprietaire,
                localite,
                pc: v.prix.pc,
            }
        })
        .collect()
}

/// Montures, à leur prix d'achat : chez leur propriétaire (PJ, ou ville de la localité ou du siège
/// de l'organisation), même quand elles sont confiées à quelqu'un d'autre.
fn source_montures(etat: &Etat, _r: &Referentiel) -> Vec<ElementPatrimoine> {
    use campagne::biens::ProprietaireBien;
    etat.montures
        .values()
        .map(|m| {
            let (proprietaire, localite) = match m.proprietaire {
                ProprietaireBien::Localite(l) => (None, Some(l)),
                ProprietaireBien::Organisation(o) => {
                    (None, etat.organisations.get(&o).map(|o| o.localite))
                }
                ProprietaireBien::Personnage(p) => (Some(p), None),
            };
            ElementPatrimoine {
                categorie: "montures",
                proprietaire,
                localite,
                pc: m.prix.pc,
            }
        })
        .collect()
}

// ---------------------------------------------------------------- agrégation

/// Une part par catégorie, dans l'ordre de `CATEGORIES`.
fn parts<'a>(elements: impl Iterator<Item = &'a ElementPatrimoine>) -> Vec<PartPatrimoine> {
    let elements: Vec<&ElementPatrimoine> = elements.collect();
    CATEGORIES
        .iter()
        .map(|(cle, nom)| PartPatrimoine {
            cle: (*cle).into(),
            nom: (*nom).into(),
            valeur: somme(
                elements
                    .iter()
                    .filter(|e| e.categorie == *cle)
                    .map(|e| e.pc)
                    .sum(),
            ),
        })
        .collect()
}

fn total_parts(parts: &[PartPatrimoine]) -> i64 {
    parts.iter().map(|p| p.valeur.pc).sum()
}

pub fn page_statistiques(etat: &Etat, r: &Referentiel) -> PageStatistiques {
    page_statistiques_avec(etat, r, SOURCES)
}

/// Comme `page_statistiques`, avec une liste de sources donnée (utile pour les tests).
pub fn page_statistiques_avec(
    etat: &Etat,
    r: &Referentiel,
    sources: &[SourcePatrimoine],
) -> PageStatistiques {
    let elements: Vec<ElementPatrimoine> = sources.iter().flat_map(|s| s(etat, r)).collect();

    let personnages: Vec<PatrimoinePersonnage> = etat
        .personnages
        .values()
        .map(|p| {
            let composantes = parts(elements.iter().filter(|e| e.proprietaire == Some(p.id)));
            PatrimoinePersonnage {
                id: p.id.0,
                nom: p.nom.clone(),
                total: somme(total_parts(&composantes)),
                composantes,
            }
        })
        .collect();
    let total_personnages: i64 = personnages.iter().map(|p| p.total.pc).sum();

    let localites: Vec<PatrimoineLocalite> = etat
        .localites
        .values()
        .map(|l| {
            let composantes = parts(elements.iter().filter(|e| e.localite == Some(l.id)));
            PatrimoineLocalite {
                id: l.id.0,
                nom: l.nom.clone(),
                total: somme(total_parts(&composantes)),
                composantes,
            }
        })
        .collect();
    let total_localites: i64 = localites.iter().map(|l| l.total.pc).sum();

    let non_attribue: i64 = elements
        .iter()
        .filter(|e| e.proprietaire.is_none() && e.localite.is_none())
        .map(|e| e.pc)
        .sum();

    let batiments: Vec<PatrimoineBatiment> = etat
        .batiments
        .values()
        .map(|b| {
            let (valeur, salles) = valeur_batiment(etat, r, b.id);
            PatrimoineBatiment {
                id: b.id.0,
                nom: b.nom.clone(),
                localite: etat
                    .localites
                    .get(&b.localite)
                    .map(|l| l.nom.clone())
                    .unwrap_or_default(),
                proprietaire: etat
                    .proprietaires_batiments
                    .get(&b.id)
                    .and_then(|p| etat.personnages.get(p))
                    .map(|p| p.nom.clone()),
                salles,
                valeur: somme(valeur),
            }
        })
        .collect();
    let total_batiments: i64 = batiments.iter().map(|b| b.valeur.pc).sum();

    let organisations: Vec<StatOrganisation> = etat
        .organisations
        .values()
        .map(|o| {
            // Les membres de la suite (don Prestige) sont comptés à part et sans valeur d'achat.
            let (suite, equipes): (Vec<&Equipe>, Vec<&Equipe>) = etat
                .equipes_organisation(o.id)
                .partition(|e| etat.suite_de(e.id).is_some());
            let (effectif, effectif_a_venir) = etat.effectif(equipes.iter().copied(), r);
            let effectif_suite = etat.effectif(suite.iter().copied(), r).0;
            let gestionnaires = etat
                .gestionnaires_de(Affectation::Organisation(o.id))
                .filter(|g| g.actif)
                .count();
            let valeur: i64 = equipes
                .iter()
                .map(|e| {
                    r.equipe(&e.reference)
                        .and_then(|u| u.prix_po)
                        .map_or(0, |po| i64::from(po) * 100)
                })
                .sum();
            StatOrganisation {
                id: o.id.0,
                nom: o.nom.clone(),
                localite: etat
                    .localites
                    .get(&o.localite)
                    .map(|l| l.nom.clone())
                    .unwrap_or_default(),
                equipes: equipes.len() as u32,
                effectif,
                effectif_a_venir,
                effectif_suite,
                gestionnaires: gestionnaires as u32,
                valeur_equipes: somme(valeur),
            }
        })
        .collect();
    let valeur_totale: i64 = organisations.iter().map(|o| o.valeur_equipes.pc).sum();

    let repartition = parts(elements.iter());
    PageStatistiques {
        patrimoine_total: somme(total_parts(&repartition)),
        repartition,
        personnages,
        total_personnages: somme(total_personnages),
        localites,
        total_localites: somme(total_localites),
        non_attribue: somme(non_attribue),
        batiments,
        total_batiments: somme(total_batiments),
        effectif_total: organisations.iter().map(|o| o.effectif).sum(),
        effectif_a_venir_total: organisations.iter().map(|o| o.effectif_a_venir).sum(),
        effectif_suite_total: organisations.iter().map(|o| o.effectif_suite).sum(),
        organisations,
        valeur_equipes: somme(valeur_totale),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use campagne::evenements::Evenement;
    use campagne::Campagne;

    fn part<'a>(parts: &'a [PartPatrimoine], cle: &str) -> &'a PartPatrimoine {
        parts.iter().find(|p| p.cle == cle).unwrap()
    }

    #[test]
    fn patrimoine_d_un_pj_et_total() {
        let r = regles::referentiel_officiel();
        let mut c = Campagne::nouvelle("Test");
        let id = c.nouvel_id_personnage();
        c.executer(
            Evenement::PersonnageCree {
                id,
                nom: "Lia".into(),
                compte_initial: Montant::po(100),
                prestige: None,
            },
            &r,
        )
        .unwrap();
        let etat = c.etat(&r);
        let p = page_statistiques(etat, &r);
        assert_eq!(p.personnages.len(), 1);
        assert_eq!(
            part(&p.personnages[0].composantes, "comptes").valeur.pc,
            10_000
        );
        assert_eq!(p.personnages[0].total.pc, 10_000);
        assert_eq!(p.total_personnages.pc, 10_000);
        assert_eq!(p.patrimoine_total.pc, 10_000);
        assert!(p.organisations.is_empty());
        // une part par catégorie, même vide
        assert_eq!(p.repartition.len(), CATEGORIES.len());
    }

    #[test]
    fn batiment_compte_chez_son_proprietaire_et_dans_sa_ville_sans_double_compte() {
        let r = regles::referentiel_officiel();
        let prix = i64::from(r.salle("bar").unwrap().prix_po.unwrap()) * 100;
        assert!(prix > 0);
        let mut c = Campagne::nouvelle("Test");
        let pj = c.nouvel_id_personnage();
        let l = c.nouvel_id_localite();
        let b = c.nouvel_id_batiment();
        let e = c.nouvel_id_etage();
        let s = c.nouvel_id_salle();
        for ev in [
            Evenement::PersonnageCree {
                id: pj,
                nom: "Seoni".into(),
                compte_initial: Montant::po(50),
                prestige: None,
            },
            Evenement::LocaliteCreee {
                id: l,
                nom: "Sandpoint".into(),
                genre: "ville".into(),
                population: None,
                categorie: CategorieLocalite::Officielle {
                    id: "petite-bourgade".into(),
                },
                notes: String::new(),
            },
            Evenement::BatimentCree {
                id: b,
                localite: l,
                nom: "Le Dragon vert".into(),
                classement: "taverne".into(),
                notes: String::new(),
                etages_successifs: false,
                modele: None,
            },
            Evenement::EtageAjoute {
                batiment: b,
                id: e,
                nom: "RDC".into(),
                mode: ModeChantier::Parallele,
            },
            Evenement::SalleAjoutee {
                id: s,
                batiment: b,
                etage: e,
                reference: "bar".into(),
                nom: None,
                deja_construite: true,
            },
        ] {
            c.executer(ev, &r).unwrap();
        }
        // Sans propriétaire : le bâtiment ne compte que dans sa ville.
        let p = page_statistiques(c.etat(&r), &r);
        assert_eq!(
            part(&p.personnages[0].composantes, "batiments").valeur.pc,
            0
        );
        assert_eq!(
            part(&p.localites[0].composantes, "batiments").valeur.pc,
            prix
        );
        assert_eq!(p.patrimoine_total.pc, 5_000 + prix);

        c.executer(
            Evenement::BatimentProprietaire {
                id: b,
                proprietaire: Some(pj),
            },
            &r,
        )
        .unwrap();
        let p = page_statistiques(c.etat(&r), &r);
        assert_eq!(
            part(&p.personnages[0].composantes, "batiments").valeur.pc,
            prix
        );
        assert_eq!(p.personnages[0].total.pc, 5_000 + prix);
        assert_eq!(
            part(&p.localites[0].composantes, "batiments").valeur.pc,
            prix
        );
        // Le total ne compte le bâtiment qu'une fois.
        assert_eq!(p.patrimoine_total.pc, 5_000 + prix);
        assert_eq!(p.batiments[0].proprietaire.as_deref(), Some("Seoni"));
        // La répartition recompose exactement le total.
        let parts: i64 = p.repartition.iter().map(|x| x.valeur.pc).sum();
        assert_eq!(parts, p.patrimoine_total.pc);

        // Un propriétaire inconnu est refusé.
        assert!(c
            .executer(
                Evenement::BatimentProprietaire {
                    id: b,
                    proprietaire: Some(IdPersonnage(99)),
                },
                &r,
            )
            .is_err());
    }

    /// Une nouvelle source (montures, véhicules…) est prise en compte partout sans autre changement.
    #[test]
    fn une_source_ajoutee_entre_dans_le_total_et_chez_son_proprietaire() {
        fn source_test(_: &Etat, _: &Referentiel) -> Vec<ElementPatrimoine> {
            vec![ElementPatrimoine {
                categorie: "comptes",
                proprietaire: Some(IdPersonnage(1)),
                localite: None,
                pc: 700,
            }]
        }
        let r = regles::referentiel_officiel();
        let mut c = Campagne::nouvelle("Test");
        let id = c.nouvel_id_personnage();
        c.executer(
            Evenement::PersonnageCree {
                id,
                nom: "Lia".into(),
                compte_initial: Montant::po(1),
                prestige: None,
            },
            &r,
        )
        .unwrap();
        let etat = c.etat(&r).clone();
        let avant = page_statistiques(&etat, &r);
        let mut sources = SOURCES.to_vec();
        sources.push(source_test);
        let apres = page_statistiques_avec(&etat, &r, &sources);
        assert_eq!(apres.patrimoine_total.pc, avant.patrimoine_total.pc + 700);
        assert_eq!(
            apres.personnages[0].total.pc,
            avant.personnages[0].total.pc + 700
        );
        assert_eq!(apres.total_personnages.pc, avant.total_personnages.pc + 700);
    }
}
