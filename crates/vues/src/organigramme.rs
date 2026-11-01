//! Organigramme des gestionnaires : le personnage du joueur au sommet, puis chaque localité avec ses
//! bâtiments et organisations, sous lesquels se rangent les gestionnaires qui y sont affectés.
//! Un gestionnaire affecté à un autre gestionnaire travaille sous ses ordres.

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
    /// Un organigramme par localité, pour la vue locale.
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

/// Gestionnaires affectés à `cible`, chacun avec ceux qui sont affectés à lui.
/// `vus` évite de boucler si deux gestionnaires se désignent mutuellement.
fn sous_cible(etat: &Etat, cible: Affectation, vus: &mut BTreeSet<IdGestionnaire>) -> Vec<NoeudOrga> {
    let mut noeuds = Vec::new();
    for g in etat.gestionnaires.values() {
        if g.affectations.contains(&cible) && vus.insert(g.id) {
            let enfants = sous_cible(etat, Affectation::Gestionnaire(g.id), vus);
            noeuds.push(noeud("gestionnaire", g.id.0, &g.nom, enfants));
        }
    }
    noeuds
}

fn organigramme_localite(etat: &Etat, l: &Localite) -> NoeudOrga {
    let mut vus = BTreeSet::new();
    let mut enfants: Vec<NoeudOrga> = etat
        .batiments_localite(l.id)
        .map(|b| {
            let sous = sous_cible(etat, Affectation::Batiment(b.id), &mut vus);
            noeud("batiment", b.id.0, &b.nom, sous)
        })
        .collect();
    enfants.extend(etat.organisations_localite(l.id).map(|o| {
        let sous = sous_cible(etat, Affectation::Organisation(o.id), &mut vus);
        noeud("organisation", o.id.0, &o.nom, sous)
    }));
    // Gestionnaires de la localité qui ne sont affectés à rien : directement sous la localité.
    for g in etat.gestionnaires.values() {
        if g.localite == l.id && g.affectations.is_empty() && vus.insert(g.id) {
            let sous = sous_cible(etat, Affectation::Gestionnaire(g.id), &mut vus);
            enfants.push(noeud("gestionnaire", g.id.0, &g.nom, sous));
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
