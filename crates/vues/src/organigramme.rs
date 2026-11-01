//! Organigramme : le personnage du joueur au sommet, puis chaque localité. Un gestionnaire est au-dessus
//! de ce qu'il gère : bâtiments, organisations, et autres gestionnaires qui lui sont affectés.

use campagne::etat::Etat;
use campagne::modele::*;
use std::collections::BTreeSet;

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct NoeudOrga {
    /// « joueur », « localite », « batiment », « organisation » ou « gestionnaire ».
    pub genre: String,
    pub id: u32,
    pub nom: String,
    pub enfants: Vec<NoeudOrga>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct Organigramme {
    /// Le personnage du joueur, au sommet ; ses enfants sont les localités.
    pub global: NoeudOrga,
    /// Un organigramme par localité.
    pub localites: Vec<NoeudOrga>,
}

fn noeud(genre: &str, id: u32, nom: &str, enfants: Vec<NoeudOrga>) -> NoeudOrga {
    NoeudOrga {
        genre: genre.to_string(),
        id,
        nom: nom.to_string(),
        enfants,
    }
}

/// Un gestionnaire, au-dessus de ce qui lui est affecté. Un autre gestionnaire affecté à lui
/// est dessous, et ainsi de suite. `vus` évite de boucler si deux gestionnaires se gèrent mutuellement.
fn gestionnaire_noeud(etat: &Etat, g: &GestionnaireEmploye, vus: &mut BTreeSet<IdGestionnaire>) -> NoeudOrga {
    let mut enfants = Vec::new();
    for a in &g.affectations {
        match a {
            Affectation::Batiment(b) => {
                let nom = etat.batiments.get(b).map(|x| x.nom.clone()).unwrap_or_default();
                enfants.push(noeud("batiment", b.0, &nom, Vec::new()));
            }
            Affectation::Organisation(o) => {
                let nom = etat.organisations.get(o).map(|x| x.nom.clone()).unwrap_or_default();
                enfants.push(noeud("organisation", o.0, &nom, Vec::new()));
            }
            Affectation::Gestionnaire(s) => {
                if let Some(sous) = etat.gestionnaires.get(s) {
                    if vus.insert(*s) {
                        enfants.push(gestionnaire_noeud(etat, sous, vus));
                    }
                }
            }
        }
    }
    noeud("gestionnaire", g.id.0, &g.nom, enfants)
}

fn organigramme_localite(etat: &Etat, l: &Localite) -> NoeudOrga {
    let mut vus = BTreeSet::new();
    // Gestionnaires gérés par un autre : ils ne sont pas au sommet de la localité.
    let geres: BTreeSet<IdGestionnaire> = etat
        .gestionnaires
        .values()
        .flat_map(|g| g.affectations.iter())
        .filter_map(|a| match a {
            Affectation::Gestionnaire(s) => Some(*s),
            _ => None,
        })
        .collect();
    let mut enfants = Vec::new();
    for g in etat.gestionnaires.values() {
        if g.localite == l.id && !geres.contains(&g.id) && vus.insert(g.id) {
            enfants.push(gestionnaire_noeud(etat, g, &mut vus));
        }
    }
    // Bâtiments et organisations que personne ne gère, directement sous la localité.
    let gere = |a: Affectation| etat.gestionnaires.values().any(|g| g.affectations.contains(&a));
    for b in etat.batiments_localite(l.id) {
        if !gere(Affectation::Batiment(b.id)) {
            enfants.push(noeud("batiment", b.id.0, &b.nom, Vec::new()));
        }
    }
    for o in etat.organisations_localite(l.id) {
        if !gere(Affectation::Organisation(o.id)) {
            enfants.push(noeud("organisation", o.id.0, &o.nom, Vec::new()));
        }
    }
    noeud("localite", l.id.0, &l.nom, enfants)
}

/// Construit l'organigramme de la campagne. `nom_joueur` : le nom affiché au sommet.
pub fn organigramme(etat: &Etat, nom_joueur: &str) -> Organigramme {
    let localites: Vec<NoeudOrga> = etat
        .localites
        .values()
        .map(|l| organigramme_localite(etat, l))
        .collect();
    Organigramme {
        global: noeud("joueur", 0, nom_joueur, localites.clone()),
        localites,
    }
}
