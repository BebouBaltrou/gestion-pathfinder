//! Courbes d'évolution des patrimoines : le journal est rejoué, et le patrimoine est mesuré
//! à la fin de chaque jour où quelque chose s'est passé. Les catégories viennent de
//! `statistiques::CATEGORIES` : une nouvelle sorte de patrimoine apparaît donc ici sans rien changer.

use crate::statistiques::{page_statistiques, PageStatistiques};
use campagne::etat::Etat;
use campagne::Campagne;
use regles::Referentiel;
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct CourbesPatrimoine {
    pub jours: Vec<u32>,
    pub dates: Vec<String>,
    pub series: Vec<SerieCourbe>,
    /// Composition du patrimoine de chaque personnage, à chaque jour de `jours`.
    pub compositions: Vec<CompositionPersonnage>,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct CompositionPersonnage {
    pub id: u32,
    pub nom: String,
    /// Une série par catégorie, en pièces de cuivre (0 avant l'apparition du personnage).
    pub composantes: Vec<SerieCategorie>,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct SerieCategorie {
    pub cle: String,
    pub nom: String,
    pub valeurs: Vec<i64>,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct SerieCourbe {
    pub cle: String,
    pub nom: String,
    /// « Total », « Personnages », « Localités » ou « Catégories ».
    pub groupe: String,
    /// Valeur en pièces de cuivre à chaque jour de `jours` (0 avant l'apparition de l'élément).
    pub valeurs: Vec<i64>,
}

type Instantane = Vec<(String, String, String, i64)>;
/// (id, nom, [(clé de catégorie, nom, valeur)]) d'un personnage.
type Composition = (u32, String, Vec<(String, String, i64)>);

fn instantane(etat: &Etat, r: &Referentiel) -> (Instantane, Vec<Composition>) {
    let s: PageStatistiques = page_statistiques(etat, r);
    let mut v: Instantane = vec![(
        "total".into(),
        "Patrimoine total".into(),
        "Total".into(),
        s.patrimoine_total.pc,
    )];
    for p in &s.personnages {
        v.push((
            format!("pj-{}", p.id),
            p.nom.clone(),
            "Personnages".into(),
            p.total.pc,
        ));
    }
    for l in &s.localites {
        v.push((
            format!("loc-{}", l.id),
            l.nom.clone(),
            "Localités".into(),
            l.total.pc,
        ));
    }
    for part in &s.repartition {
        v.push((
            format!("cat-{}", part.cle),
            part.nom.clone(),
            "Catégories".into(),
            part.valeur.pc,
        ));
    }
    let compositions = s
        .personnages
        .iter()
        .map(|p| {
            (
                p.id,
                p.nom.clone(),
                p.composantes
                    .iter()
                    .map(|c| (c.cle.clone(), c.nom.clone(), c.valeur.pc))
                    .collect(),
            )
        })
        .collect();
    (v, compositions)
}

pub fn courbes_patrimoine(c: &Campagne, r: &Referentiel) -> CourbesPatrimoine {
    let entrees = &c.journal[..c.curseur];
    let mut etat = Etat::nouveau();
    let mut points: BTreeMap<u32, (Instantane, Vec<Composition>)> = BTreeMap::new();
    for (i, x) in entrees.iter().enumerate() {
        let _ = etat.appliquer(&x.evenement, r);
        let fin_du_jour = entrees.get(i + 1).is_none_or(|n| n.jour != x.jour);
        if fin_du_jour {
            points.insert(etat.jour, instantane(&etat, r));
        }
    }
    if entrees.is_empty() {
        points.insert(etat.jour, instantane(&etat, r));
    }

    let jours: Vec<u32> = points.keys().copied().collect();
    let mut cles: Vec<String> = Vec::new();
    let mut infos: BTreeMap<String, (String, String)> = BTreeMap::new();
    for (inst, _) in points.values() {
        for (cle, nom, groupe, _) in inst {
            if !infos.contains_key(cle) {
                cles.push(cle.clone());
            }
            infos.insert(cle.clone(), (nom.clone(), groupe.clone()));
        }
    }
    let series = cles
        .into_iter()
        .map(|cle| {
            let (nom, groupe) = infos[&cle].clone();
            let valeurs = points
                .values()
                .map(|(inst, _)| {
                    inst.iter()
                        .find(|(k, ..)| *k == cle)
                        .map_or(0, |(_, _, _, pc)| *pc)
                })
                .collect();
            SerieCourbe {
                cle,
                nom,
                groupe,
                valeurs,
            }
        })
        .collect();

    // Composition par personnage, dans l'ordre d'apparition.
    let mut ids: Vec<u32> = Vec::new();
    let mut noms: BTreeMap<u32, String> = BTreeMap::new();
    for (_, comps) in points.values() {
        for (id, nom, _) in comps {
            if !noms.contains_key(id) {
                ids.push(*id);
            }
            noms.insert(*id, nom.clone());
        }
    }
    let compositions = ids
        .into_iter()
        .map(|id| {
            // catégories (clé, nom) dans l'ordre où elles apparaissent
            let mut categories: Vec<(String, String)> = Vec::new();
            for (_, comps) in points.values() {
                if let Some((_, _, parts)) = comps.iter().find(|(i, ..)| *i == id) {
                    for (cle, nom, _) in parts {
                        if !categories.iter().any(|(k, _)| k == cle) {
                            categories.push((cle.clone(), nom.clone()));
                        }
                    }
                }
            }
            let composantes = categories
                .into_iter()
                .map(|(cle, nom)| {
                    let valeurs = points
                        .values()
                        .map(|(_, comps)| {
                            comps
                                .iter()
                                .find(|(i, ..)| *i == id)
                                .and_then(|(_, _, parts)| parts.iter().find(|(k, ..)| *k == cle))
                                .map_or(0, |(_, _, pc)| *pc)
                        })
                        .collect();
                    SerieCategorie { cle, nom, valeurs }
                })
                .collect();
            CompositionPersonnage {
                id,
                nom: noms[&id].clone(),
                composantes,
            }
        })
        .collect();
    CourbesPatrimoine {
        dates: jours.iter().map(|j| c.calendrier.libelle(*j)).collect(),
        jours,
        series,
        compositions,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use campagne::evenements::Evenement;
    use domaine::Montant;

    #[test]
    fn la_courbe_suit_le_compte_d_un_pj() {
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
        let courbes = courbes_patrimoine(&c, &r);
        assert!(!courbes.jours.is_empty());
        let total = courbes.series.iter().find(|s| s.cle == "total").unwrap();
        assert_eq!(*total.valeurs.last().unwrap(), 10_000);
        let pj = courbes
            .series
            .iter()
            .find(|s| s.groupe == "Personnages")
            .unwrap();
        assert_eq!(pj.nom, "Lia");
        assert_eq!(*pj.valeurs.last().unwrap(), 10_000);
        assert_eq!(courbes.dates.len(), courbes.jours.len());
        // une série par catégorie : une nouvelle sorte de patrimoine apparaît sans rien changer
        let cats = courbes
            .series
            .iter()
            .filter(|s| s.groupe == "Catégories")
            .count();
        assert_eq!(cats, crate::statistiques::CATEGORIES.len());
        let compo = &courbes.compositions[0];
        assert_eq!(compo.nom, "Lia");
        assert_eq!(
            compo.composantes.len(),
            crate::statistiques::CATEGORIES.len()
        );
        let comptes = compo
            .composantes
            .iter()
            .find(|x| x.cle == "comptes")
            .unwrap();
        assert_eq!(*comptes.valeurs.last().unwrap(), 10_000);
        assert_eq!(comptes.valeurs.len(), courbes.jours.len());
    }
}
