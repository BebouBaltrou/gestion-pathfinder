//! Courbes d'évolution des patrimoines : le journal est rejoué, et le patrimoine est mesuré
//! à la fin de chaque jour où quelque chose s'est passé.

use crate::statistiques::page_statistiques;
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
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct SerieCourbe {
    pub cle: String,
    pub nom: String,
    /// « Total », « Personnages », « Localités » ou « Autres ».
    pub groupe: String,
    /// Valeur en pièces de cuivre à chaque jour de `jours` (0 avant l'apparition de l'élément).
    pub valeurs: Vec<i64>,
}

type Instantane = Vec<(String, String, String, i64)>;

fn instantane(etat: &Etat, r: &Referentiel) -> Instantane {
    let s = page_statistiques(etat, r);
    let mut v: Instantane = vec![
        (
            "total".into(),
            "Patrimoine total".into(),
            "Total".into(),
            s.patrimoine_total.pc,
        ),
        (
            "batiments".into(),
            "Bâtiments".into(),
            "Autres".into(),
            s.total_batiments.pc,
        ),
        (
            "objets".into(),
            "Objets hors PJ".into(),
            "Autres".into(),
            s.objets_autres.pc,
        ),
    ];
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
    v
}

pub fn courbes_patrimoine(c: &Campagne, r: &Referentiel) -> CourbesPatrimoine {
    let entrees = &c.journal[..c.curseur];
    let mut etat = Etat::nouveau();
    let mut points: BTreeMap<u32, Instantane> = BTreeMap::new();
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
    for inst in points.values() {
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
                .map(|inst| {
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
    CourbesPatrimoine {
        dates: jours.iter().map(|j| c.calendrier.libelle(*j)).collect(),
        jours,
        series,
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
    }
}
