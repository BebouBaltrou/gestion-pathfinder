//! Le don Prestige à l'écran : score, tableau, modificateurs, acolyte et partisans.
//! Tout est calculé par le cœur (`campagne::prestige`).

use campagne::etat::Etat;
use campagne::evenements::Evenement;
use campagne::modele::{IdOrganisation, IdPersonnage};
use campagne::prestige::{ligne_du_tableau, Portee, MODIFICATEURS, NIVEAU_MINIMAL};
use domaine::fiches::Taille;
use regles::Referentiel;

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
    /// Équipe qui le représente (production, lieu de travail, équipage) : numéro et nom, avec « suite de … ».
    pub equipe: Option<u32>,
    pub equipe_libelle: Option<String>,
    pub note: String,
}

/// Équipe du référentiel proposée comme équivalent d'un partisan.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct EquivalentVue {
    pub id: String,
    pub libelle: String,
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
    /// Équipes du livre qui peuvent représenter un partisan.
    pub equivalents: Vec<EquivalentVue>,
}

/// Le don Prestige d'un personnage (`actif: false` s'il ne l'a pas).
pub fn prestige_don(etat: &Etat, pj: IdPersonnage, r: &Referentiel) -> PrestigeDonVue {
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
                    equipe: p.equipe.map(|e| e.0),
                    equipe_libelle: p
                        .equipe
                        .and_then(|e| etat.equipes.get(&e))
                        .map(|e| etat.nom_equipe(e, r)),
                    note: p.note.clone(),
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
        equivalents: r
            .equipes
            .iter()
            .map(|u| EquivalentVue {
                id: u.id.clone(),
                libelle: format!(
                    "{} — {}{}",
                    u.nom,
                    match &u.taille {
                        Some(Taille::Personnes { nombre }) => format!("{nombre} personne(s)"),
                        _ => "taille inconnue".to_string(),
                    },
                    u.recettes
                        .as_ref()
                        .map_or(String::new(), |x| format!(" · {}", x.texte)),
                ),
            })
            .collect(),
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

/// Recrute un partisan. Avec une équipe équivalente du livre, il rejoint l'organisation « Suite de
/// [maître] » (créée au premier partisan) : il peut alors être placé comme une équipe et il produit
/// sa part de la taille de l'équipe. Sans équipe, c'est un simple effectif avec sa note.
pub fn recruter_partisan(
    c: &mut campagne::Campagne,
    r: &Referentiel,
    maitre: u32,
    niveau: u32,
    nom: String,
    equipe_reference: Option<String>,
    note: String,
) -> Result<(Evenement, String), String> {
    let maitre = IdPersonnage(maitre);
    let etat = c.etat(r).clone();
    let pj = etat.pj(maitre).map_err(|e| e.to_string())?.clone();
    let nom = nom.trim().to_string();
    let libelle = format!(
        "Partisan {} recruté",
        if nom.is_empty() { "(sans nom)" } else { &nom }
    );
    let Some(reference) = equipe_reference.filter(|x| !x.is_empty()) else {
        return Ok((
            Evenement::PartisanAjoute {
                maitre,
                niveau,
                nom,
                equipe: None,
                note,
            },
            libelle,
        ));
    };
    r.equipe(&reference).ok_or("Équipe du livre introuvable")?;
    let localite = pj
        .localite
        .ok_or_else(|| format!("Placez d'abord {} dans une localité", pj.nom))?;
    // l'organisation de la suite : celle des équipes de ses autres partisans, sinon une nouvelle
    let organisation: Option<IdOrganisation> = etat.prestige_don.get(&maitre).and_then(|d| {
        d.partisans.iter().find_map(|p| {
            p.equipe
                .and_then(|e| etat.equipes.get(&e))
                .map(|e| e.organisation)
        })
    });
    let mut evenements = Vec::new();
    let organisation = match organisation {
        Some(o) => o,
        None => {
            let id = c.nouvel_id_organisation();
            evenements.push(Evenement::OrganisationCreee {
                id,
                localite,
                nom: format!("Suite de {}", pj.nom),
                classement: String::new(),
                notes: String::new(),
                modele: None,
            });
            id
        }
    };
    let equipe = c.nouvel_id_equipe();
    evenements.push(Evenement::EquipeAjoutee {
        id: equipe,
        organisation,
        reference,
        nom: (!nom.is_empty()).then(|| nom.clone()),
        lieu: None,
        deja_recrutee: true,
    });
    evenements.push(Evenement::PartisanAjoute {
        maitre,
        niveau,
        nom,
        equipe: Some(equipe),
        note,
    });
    Ok((
        Evenement::Groupe {
            libelle: libelle.clone(),
            evenements,
        },
        libelle,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use campagne::etat::Etat;
    use campagne::modele::*;
    use campagne::prestige::Partisan;
    use domaine::Montant;

    fn evs(l: IdLocalite, pj: IdPersonnage) -> Vec<Evenement> {
        vec![
            Evenement::LocaliteCreee {
                id: l,
                nom: "Fort".into(),
                genre: "ville".into(),
                population: None,
                categorie: CategorieLocalite::Officielle {
                    id: "petite-bourgade".into(),
                },
                notes: String::new(),
            },
            Evenement::PersonnageCree {
                id: pj,
                nom: "Seoni".into(),
                compte_initial: Montant::po(0),
                prestige: None,
            },
        ]
    }

    #[test]
    fn recruter_sans_localite_est_refuse_et_sans_equipe_donne_un_effectif() {
        let r = regles::referentiel_officiel();
        let mut c = campagne::Campagne::nouvelle("Test");
        let pj = IdPersonnage(1);
        c.executer(evs(IdLocalite(1), pj).remove(1), &r).unwrap();
        let err = recruter_partisan(
            &mut c,
            &r,
            pj.0,
            1,
            "Ada".into(),
            Some("archers".into()),
            String::new(),
        )
        .unwrap_err();
        assert!(err.contains("Placez d'abord"), "{err}");
        let (ev, _) =
            recruter_partisan(&mut c, &r, pj.0, 1, "Bob".into(), None, "messager".into()).unwrap();
        assert!(matches!(
            ev,
            Evenement::PartisanAjoute {
                equipe: None,
                ref note,
                ..
            } if note == "messager"
        ));
    }

    #[test]
    fn la_suite_est_comptee_a_part_et_sans_valeur() {
        let r = regles::referentiel_officiel();
        let mut e = Etat::default();
        let (l, pj) = (IdLocalite(1), IdPersonnage(1));
        for ev in evs(l, pj) {
            e.appliquer(&ev, &r).unwrap();
        }
        // une organisation normale (artisans : 3 personnes) et la suite (archers : 5 personnes)
        for (org, nom, reference, equipe) in [
            (IdOrganisation(1), "Guilde", "artisans", IdEquipe(1)),
            (IdOrganisation(2), "Suite de Seoni", "archers", IdEquipe(2)),
        ] {
            e.appliquer(
                &Evenement::OrganisationCreee {
                    id: org,
                    localite: l,
                    nom: nom.into(),
                    classement: String::new(),
                    notes: String::new(),
                    modele: None,
                },
                &r,
            )
            .unwrap();
            e.appliquer(
                &Evenement::EquipeAjoutee {
                    id: equipe,
                    organisation: org,
                    reference: reference.into(),
                    nom: None,
                    lieu: None,
                    deja_recrutee: true,
                },
                &r,
            )
            .unwrap();
        }
        e.prestige_don
            .entry(pj)
            .or_default()
            .partisans
            .push(Partisan {
                niveau: 1,
                nom: "Ada".into(),
                equipe: Some(IdEquipe(2)),
                note: String::new(),
            });
        let stats = crate::statistiques::page_statistiques(&e, &r);
        assert_eq!(stats.effectif_total, 3);
        assert_eq!(stats.effectif_suite_total, 5);
        // la valeur ne compte que les artisans
        let artisans = r.equipe("artisans").unwrap().prix_po.unwrap();
        assert_eq!(stats.valeur_equipes.pc, i64::from(artisans) * 100);
        // la page du don montre l'équipe, avec l'étiquette « suite de … »
        let v = prestige_don(&e, pj, &r);
        assert_eq!(v.partisans[0].equipe, Some(2));
        assert!(v.partisans[0]
            .equipe_libelle
            .as_deref()
            .unwrap()
            .contains("suite de Seoni"));
    }
}
