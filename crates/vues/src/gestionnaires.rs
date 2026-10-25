//! Page Gestionnaires (Guide de campagne p. 89-91) : embauche, salaire quotidien, affectations.

use crate::localites::Choix;
use crate::Somme;
use campagne::etat::Etat;
use campagne::evenements::Evenement;
use campagne::modele::*;
use domaine::Montant;
use regles::Referentiel;

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct PageGestionnaires {
    pub salaires: Vec<CarteSalaires>,
    pub exemples: Vec<Exemple>,
    pub localites: Vec<Choix>,
    pub gestionnaires: Vec<FicheGestionnaire>,
    /// Un gestionnaire payé une fois par jour, ou une fois par affectation (réglage).
    pub note_salaire: String,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct CarteSalaires {
    pub localite: String,
    pub salaires_jour: Somme,
    pub tresorerie: Somme,
    pub negative: bool,
    pub jours_d_avance: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct Exemple {
    pub id: String,
    pub nom: String,
    /// « Ameiko — 3 po/jour (p. 90) »
    pub libelle: String,
    /// Salaire en po, à préremplir.
    pub salaire: String,
    pub competences: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CibleAffectation {
    /// « batiment » ou « organisation ».
    pub genre: GenreAffectation,
    pub id: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub enum GenreAffectation {
    Batiment,
    Organisation,
}

impl From<Affectation> for CibleAffectation {
    fn from(a: Affectation) -> Self {
        match a {
            Affectation::Batiment(b) => CibleAffectation {
                genre: GenreAffectation::Batiment,
                id: b.0,
            },
            Affectation::Organisation(o) => CibleAffectation {
                genre: GenreAffectation::Organisation,
                id: o.0,
            },
        }
    }
}

impl From<&CibleAffectation> for Affectation {
    fn from(c: &CibleAffectation) -> Self {
        match c.genre {
            GenreAffectation::Batiment => Affectation::Batiment(IdBatiment(c.id)),
            GenreAffectation::Organisation => Affectation::Organisation(IdOrganisation(c.id)),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ChoixAffectation {
    pub cible: CibleAffectation,
    pub libelle: String,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct FicheGestionnaire {
    pub id: u32,
    pub nom: String,
    /// « Ameiko (p. 90) » si tiré d'un exemple du livre.
    pub exemple: Option<String>,
    pub salaire_jour: Somme,
    pub actif: bool,
    pub localite: u32,
    pub affectations: Vec<ChoixAffectation>,
    /// Ce qu'on peut encore lui affecter.
    pub possibles: Vec<ChoixAffectation>,
}

fn libelle_affectation(a: &Affectation, etat: &Etat) -> String {
    match a {
        Affectation::Batiment(b) => etat
            .batiments
            .get(b)
            .map(|x| format!("Bâtiment : {}", x.nom))
            .unwrap_or(b.to_string()),
        Affectation::Organisation(o) => etat
            .organisations
            .get(o)
            .map(|x| format!("Organisation : {}", x.nom))
            .unwrap_or(o.to_string()),
    }
}

pub fn page_gestionnaires(c: &mut campagne::Campagne, r: &Referentiel) -> PageGestionnaires {
    let etat = c.etat(r).clone();
    let mut payeuses: Vec<&Localite> = etat
        .localites
        .values()
        .filter(|l| etat.salaires_localite(l.id).pa > 0)
        .collect();
    payeuses.sort_by_key(|l| l.nom.clone());
    let tous: Vec<Affectation> = etat
        .batiments
        .values()
        .map(|b| Affectation::Batiment(b.id))
        .chain(
            etat.organisations
                .values()
                .map(|o| Affectation::Organisation(o.id)),
        )
        .collect();
    PageGestionnaires {
        salaires: payeuses
            .into_iter()
            .map(|l| {
                let jour = etat.salaires_localite(l.id);
                CarteSalaires {
                    localite: l.nom.clone(),
                    salaires_jour: jour.into(),
                    tresorerie: l.tresorerie.into(),
                    negative: l.tresorerie.est_negatif(),
                    jours_d_avance: (!l.tresorerie.est_negatif())
                        .then(|| l.tresorerie.pa / jour.pa),
                }
            })
            .collect(),
        exemples: r
            .gestionnaires
            .iter()
            .map(|g| Exemple {
                id: g.id.clone(),
                nom: g.nom.clone(),
                libelle: format!(
                    "{} — {} po/jour (p. {})",
                    g.nom,
                    g.salaire_po_jour,
                    g.source.page.unwrap_or(0)
                ),
                salaire: g.salaire_po_jour.to_string(),
                competences: g.competences.clone(),
            })
            .collect(),
        localites: etat
            .localites
            .values()
            .map(|l| Choix {
                id: l.id.0,
                libelle: l.nom.clone(),
            })
            .collect(),
        gestionnaires: etat
            .gestionnaires
            .values()
            .map(|g| FicheGestionnaire {
                id: g.id.0,
                nom: g.nom.clone(),
                exemple: g
                    .reference
                    .as_ref()
                    .and_then(|x| r.gestionnaire(x))
                    .map(|x| format!("{} (p. {})", x.nom, x.source.page.unwrap_or(0))),
                salaire_jour: etat.salaire_quotidien(g).into(),
                actif: g.actif,
                localite: g.localite.0,
                affectations: g
                    .affectations
                    .iter()
                    .map(|a| ChoixAffectation {
                        cible: (*a).into(),
                        libelle: libelle_affectation(a, &etat),
                    })
                    .collect(),
                possibles: tous
                    .iter()
                    .filter(|a| !g.affectations.contains(a))
                    .map(|a| ChoixAffectation {
                        cible: (*a).into(),
                        libelle: libelle_affectation(a, &etat),
                    })
                    .collect(),
            })
            .collect(),
        note_salaire: if etat.salaire_par_affectation {
            "Réglage actif : un gestionnaire est payé une fois par affectation (modifiable dans les Paramètres).".into()
        } else {
            "Un gestionnaire affecté à plusieurs établissements n'est payé qu'une fois par jour (PC-06 ; modifiable dans les Paramètres).".into()
        },
    }
}

/// Action demandée par l'interface sur un gestionnaire.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(tag = "type", rename_all = "camelCase"))]
pub enum ActionGestionnaire {
    #[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
    Embaucher {
        reference: Option<String>,
        nom: String,
        salaire: String,
        localite: u32,
    },
    #[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
    Regler { id: u32, actif: bool, localite: u32 },
    #[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
    Renvoyer { id: u32 },
    #[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
    Affecter { id: u32, cible: CibleAffectation },
    #[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
    Desaffecter { id: u32, cible: CibleAffectation },
}

pub fn action(
    c: &mut campagne::Campagne,
    r: &Referentiel,
    a: ActionGestionnaire,
) -> Result<(Evenement, String), String> {
    use ActionGestionnaire as A;
    let etat = c.etat(r).clone();
    let gest = |id: u32| {
        etat.gestionnaires
            .get(&IdGestionnaire(id))
            .cloned()
            .ok_or_else(|| "Gestionnaire introuvable".to_string())
    };
    let modifie = |g: &GestionnaireEmploye| Evenement::GestionnaireModifie {
        id: g.id,
        nom: g.nom.clone(),
        salaire: g.salaire,
        localite: g.localite,
        affectations: g.affectations.clone(),
        actif: g.actif,
    };
    Ok(match a {
        A::Embaucher {
            reference,
            nom,
            salaire,
            localite,
        } => {
            let nom = nom.trim().to_string();
            if nom.is_empty() {
                return Err("Donnez un nom au gestionnaire".into());
            }
            let salaire = Montant::depuis_texte(&salaire)
                .filter(|m| !m.est_negatif())
                .ok_or("Salaire illisible : écrivez par exemple 3 ou 2 po 5 pa")?;
            if let Some(x) = &reference {
                r.gestionnaire(x).ok_or("Exemple du livre introuvable")?;
            }
            (
                Evenement::GestionnaireEmbauche {
                    id: c.nouvel_id_gestionnaire(),
                    reference,
                    nom: nom.clone(),
                    salaire,
                    localite: IdLocalite(localite),
                },
                format!("{nom} embauché(e)"),
            )
        }
        A::Regler {
            id,
            actif,
            localite,
        } => {
            let mut g = gest(id)?;
            g.actif = actif;
            g.localite = IdLocalite(localite);
            (modifie(&g), format!("{} modifié(e)", g.nom))
        }
        A::Renvoyer { id } => {
            let g = gest(id)?;
            (
                Evenement::GestionnaireRenvoye { id: g.id },
                format!("{} renvoyé(e)", g.nom),
            )
        }
        A::Affecter { id, cible } => {
            let mut g = gest(id)?;
            let a: Affectation = (&cible).into();
            if !g.affectations.contains(&a) {
                g.affectations.push(a);
            }
            (modifie(&g), format!("{} affecté(e)", g.nom))
        }
        A::Desaffecter { id, cible } => {
            let mut g = gest(id)?;
            let a: Affectation = (&cible).into();
            g.affectations.retain(|x| *x != a);
            (modifie(&g), format!("{} modifié(e)", g.nom))
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use campagne::Campagne;

    #[test]
    fn embauche_affectation_et_salaires() {
        let r = regles::referentiel_officiel();
        let mut c = Campagne::nouvelle("Test");
        let l = c.nouvel_id_localite();
        let s = crate::localites::SaisieLocalite {
            nom: "Sandpoint".into(),
            officielle: false,
            categorie_perso_limite: "10".into(),
            ..Default::default()
        };
        c.executer(s.creation(l, &r).unwrap(), &r).unwrap();
        let o = c.nouvel_id_organisation();
        c.executer(
            Evenement::OrganisationCreee {
                id: o,
                localite: l,
                nom: "Guilde".into(),
                classement: String::new(),
                notes: String::new(),
                modele: None,
            },
            &r,
        )
        .unwrap();
        let exemple = r.gestionnaires[0].clone();
        let (ev, msg) = action(
            &mut c,
            &r,
            ActionGestionnaire::Embaucher {
                reference: Some(exemple.id.clone()),
                nom: "Ameiko".into(),
                salaire: exemple.salaire_po_jour.to_string(),
                localite: l.0,
            },
        )
        .unwrap();
        assert_eq!(msg, "Ameiko embauché(e)");
        c.executer(ev, &r).unwrap();
        let (ev, _) = action(
            &mut c,
            &r,
            ActionGestionnaire::Affecter {
                id: 1,
                cible: CibleAffectation {
                    genre: GenreAffectation::Organisation,
                    id: o.0,
                },
            },
        )
        .unwrap();
        c.executer(ev, &r).unwrap();
        let p = page_gestionnaires(&mut c, &r);
        assert_eq!(p.gestionnaires.len(), 1);
        assert_eq!(p.gestionnaires[0].affectations.len(), 1);
        assert!(p.gestionnaires[0].possibles.is_empty());
        assert_eq!(p.salaires.len(), 1);
        assert_eq!(p.exemples.len(), r.gestionnaires.len());
        // inactif : plus payé
        let (ev, _) = action(
            &mut c,
            &r,
            ActionGestionnaire::Regler {
                id: 1,
                actif: false,
                localite: l.0,
            },
        )
        .unwrap();
        c.executer(ev, &r).unwrap();
        let p = page_gestionnaires(&mut c, &r);
        assert!(!p.gestionnaires[0].actif);
        assert!(p.salaires.is_empty());
        let (ev, _) = action(&mut c, &r, ActionGestionnaire::Renvoyer { id: 1 }).unwrap();
        c.executer(ev, &r).unwrap();
        assert!(page_gestionnaires(&mut c, &r).gestionnaires.is_empty());
    }

    #[test]
    fn saisies_refusees() {
        let r = regles::referentiel_officiel();
        let mut c = Campagne::nouvelle("Test");
        let embauche = |nom: &str, salaire: &str| ActionGestionnaire::Embaucher {
            reference: None,
            nom: nom.into(),
            salaire: salaire.into(),
            localite: 1,
        };
        assert!(action(&mut c, &r, embauche(" ", "3")).is_err());
        assert!(action(&mut c, &r, embauche("A", "beaucoup")).is_err());
        assert!(action(&mut c, &r, embauche("A", "-3")).is_err());
        assert!(action(&mut c, &r, ActionGestionnaire::Renvoyer { id: 9 }).is_err());
    }
}

#[cfg(all(test, feature = "serde"))]
mod tests_json {
    use super::*;

    #[test]
    fn actions_lues_comme_l_interface_les_envoie() {
        let a: ActionGestionnaire = serde_json::from_str(
            r#"{"type":"affecter","id":1,"cible":{"genre":"batiment","id":2}}"#,
        )
        .unwrap();
        assert!(matches!(a, ActionGestionnaire::Affecter { id: 1, .. }));
        let a: ActionGestionnaire = serde_json::from_str(
            r#"{"type":"embaucher","reference":null,"nom":"A","salaire":"3","localite":1}"#,
        )
        .unwrap();
        assert!(matches!(a, ActionGestionnaire::Embaucher { .. }));
    }
}
