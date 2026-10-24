//! Le don Prestige à l'écran : score, tableau, modificateurs, acolyte et partisans.
//! Tout est calculé par le cœur (`campagne::prestige`).

use campagne::etat::Etat;
use campagne::modele::IdPersonnage;
use campagne::prestige::{ligne_du_tableau, Portee, MODIFICATEURS, NIVEAU_MINIMAL};

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct ModificateurVue {
    pub id: String,
    pub libelle: String,
    pub valeur: i32,
    /// « tous », « acolyte » ou « partisans ».
    pub portee: String,
    pub cumulable: bool,
    /// Nombre de fois coché (0 : non coché).
    pub compte: u32,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct AcolyteVue {
    pub id: u32,
    pub nom: String,
    pub niveau: Option<u32>,
    pub xp: Option<u32>,
    pub a_une_fiche: bool,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct PartisanVue {
    pub index: usize,
    pub niveau: u32,
    pub nom: String,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct PlacesVue {
    pub niveau: u32,
    pub permis: u32,
    pub pris: u32,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct PrestigeDonVue {
    /// Le personnage a le don.
    pub actif: bool,
    pub niveau_minimal: u32,
    pub score_acolyte: Option<i32>,
    pub score_partisans: Option<i32>,
    /// Niveau maximal de l'acolyte d'après le tableau, puis avec la limite « deux niveaux sous le maître ».
    pub niveau_tableau: u32,
    pub niveau_max_acolyte: u32,
    pub modificateurs: Vec<ModificateurVue>,
    pub acolyte: Option<AcolyteVue>,
    /// Personnages qui peuvent devenir son acolyte.
    pub candidats: Vec<(u32, String)>,
    pub partisans: Vec<PartisanVue>,
    pub places: Vec<PlacesVue>,
}

/// Le don Prestige d'un personnage (`actif: false` s'il ne l'a pas).
pub fn prestige_don(etat: &Etat, pj: IdPersonnage) -> PrestigeDonVue {
    let actif = etat.a_le_don_prestige(pj);
    let don = etat.prestige_don.get(&pj);
    let score_acolyte = etat.score_prestige_don(pj, true);
    let score_partisans = etat.score_prestige_don(pj, false);
    let niveau_tableau = score_acolyte
        .map(|s| ligne_du_tableau(s).niveau_acolyte)
        .unwrap_or(0);
    let permis = etat.places_de_partisans(pj);
    let partisans: Vec<PartisanVue> = don
        .map(|d| {
            d.partisans
                .iter()
                .enumerate()
                .map(|(index, p)| PartisanVue {
                    index,
                    niveau: p.niveau,
                    nom: p.nom.clone(),
                })
                .collect()
        })
        .unwrap_or_default();
    let acolyte = etat.acolyte_de(pj).and_then(|a| {
        let p = etat.personnages.get(&a)?;
        let f = etat.fiche_de(a);
        Some(AcolyteVue {
            id: a.0,
            nom: p.nom.clone(),
            niveau: f.as_ref().map(|e| e.fiche.niveau_de_personnage()),
            xp: f.as_ref().map(|e| e.xp),
            a_une_fiche: f.is_some(),
        })
    });
    PrestigeDonVue {
        actif,
        niveau_minimal: NIVEAU_MINIMAL,
        acolyte: acolyte.clone(),
        score_acolyte,
        score_partisans,
        niveau_tableau,
        niveau_max_acolyte: etat.plafond_acolyte(pj).unwrap_or(0),
        modificateurs: MODIFICATEURS
            .iter()
            .map(|m| ModificateurVue {
                id: m.id.into(),
                libelle: m.libelle.into(),
                valeur: m.valeur,
                portee: match m.portee {
                    Portee::Tous => "tous",
                    Portee::Acolyte => "acolyte",
                    Portee::Partisans => "partisans",
                }
                .into(),
                cumulable: m.cumulable,
                compte: don
                    .and_then(|d| d.modificateurs.get(m.id))
                    .copied()
                    .unwrap_or(0),
            })
            .collect(),
        candidats: if actif && acolyte.is_none() {
            etat.personnages
                .values()
                .filter(|p| {
                    p.id != pj
                        && !etat.acolytes.contains_key(&p.id)
                        && !etat.a_le_don_prestige(p.id)
                })
                .map(|p| (p.id.0, p.nom.clone()))
                .collect()
        } else {
            Vec::new()
        },
        places: (1..=6)
            .map(|n| PlacesVue {
                niveau: n,
                permis: permis[(n - 1) as usize],
                pris: partisans.iter().filter(|p| p.niveau == n).count() as u32,
            })
            .collect(),
        partisans,
    }
}
