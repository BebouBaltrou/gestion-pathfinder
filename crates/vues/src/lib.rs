//! Modèles d'affichage : ce que l'interface montre, calculé ici à partir de l'état de la
//! campagne. L'interface (quelle qu'elle soit) ne refait aucun calcul de règles.

pub mod actions_batiments;
pub mod aventure;
pub mod batiments;
pub mod gestionnaires;
pub mod historique;
pub mod inventaire;
pub mod localites;
pub mod localites_modeles;
pub mod organisations;
pub mod parametres;
pub mod personnage;
pub mod personnages;
pub mod points;
pub mod prestige_don;
pub mod recettes;
pub mod referentiel;
pub mod tableau;
pub mod ventes;

use domaine::{CoutCapitaux, Montant};

/// Pages de l'application (identifiants partagés avec l'interface).
pub mod page {
    pub const TABLEAU: &str = "tableau";
    pub const LOCALITES: &str = "localites";
    pub const BATIMENTS: &str = "batiments";
    pub const ORGANISATIONS: &str = "organisations";
    pub const GESTIONNAIRES: &str = "gestionnaires";
    pub const RECETTES: &str = "recettes";
    pub const PERSONNAGES: &str = "personnages";
}

/// Somme en pièces de cuivre, avec son libellé (« 4 po 7 pa »).
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Somme {
    pub pc: i64,
    pub texte: String,
}

impl From<Montant> for Somme {
    fn from(m: Montant) -> Self {
        Somme {
            pc: m.pc,
            texte: m.to_string(),
        }
    }
}

/// Les quatre capitaux, dans l'ordre du livre.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Capitaux {
    pub marchandises: u32,
    pub influence: u32,
    pub travail: u32,
    pub magie: u32,
}

impl From<Capitaux> for CoutCapitaux {
    fn from(c: Capitaux) -> Self {
        CoutCapitaux {
            marchandises: c.marchandises,
            influence: c.influence,
            travail: c.travail,
            magie: c.magie,
        }
    }
}

impl From<CoutCapitaux> for Capitaux {
    fn from(c: CoutCapitaux) -> Self {
        Capitaux {
            marchandises: c.marchandises,
            influence: c.influence,
            travail: c.travail,
            magie: c.magie,
        }
    }
}

/// Barre du haut : campagne ouverte, date, annuler/rétablir.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct Entete {
    pub campagne: String,
    pub jour: u32,
    pub date: String,
    pub peut_annuler: bool,
    pub peut_retablir: bool,
    /// Libellé de l'action que « Annuler » défera.
    pub derniere_action: Option<String>,
    /// Pastilles de la navigation : (page, texte).
    pub pastilles: Vec<(String, String)>,
}

pub fn entete(c: &mut campagne::Campagne, r: &regles::Referentiel) -> Entete {
    let etat = c.etat(r).clone();
    let derniere = c.historique().next().map(|e| e.evenement.clone());
    let derniere_action = derniere.map(|e| c.decrire(&e, r));
    let mut pastilles = Vec::new();
    let attente: u32 = etat
        .localites
        .keys()
        .filter(|l| etat.a_entreprise(**l, r))
        .map(|l| etat.jours_sans_recettes(*l))
        .max()
        .unwrap_or(0);
    if attente > 0 {
        pastilles.push((page::RECETTES.to_string(), format!("{attente} j")));
    }
    Entete {
        campagne: c.nom.clone(),
        jour: etat.jour,
        date: c.calendrier.libelle(etat.jour),
        peut_annuler: c.peut_annuler(),
        peut_retablir: c.peut_retablir(),
        derniere_action,
        pastilles,
    }
}

/// Message affiché après une action : le libellé de l'action suivi de ses conséquences
/// (salle terminée, équipe recrutée…). Le booléen signale une conséquence à surveiller
/// (trésorerie négative, test de prestige).
pub fn resume_action(
    succes: &str,
    consequences: &[campagne::etat::Consequence],
    etat: &campagne::etat::Etat,
) -> (String, bool) {
    use campagne::etat::Consequence as C;
    let mut msg = succes.to_string();
    let mut alerte = false;
    for c in consequences {
        match c {
            C::TransfertArrive(t) => {
                let nom = etat
                    .localites
                    .get(&t.vers)
                    .map(|l| l.nom.clone())
                    .unwrap_or_default();
                msg.push_str(&format!(" — arrivée de {} à {nom}", t.lot));
            }
            C::SalleTerminee { nom, batiment, .. } => {
                msg.push_str(&format!(" — {nom} terminée ({batiment})"));
            }
            C::EquipeRecrutee {
                nom, organisation, ..
            } => msg.push_str(&format!(" — {nom} recrutée ({organisation})")),
            C::EquipeArrivee { nom, localite, .. } => {
                msg.push_str(&format!(" — {nom} arrivée à {localite}"))
            }
            C::VenteConclue {
                libelle,
                po,
                capitaux,
                ..
            } => {
                let cap = if capitaux.unites() > 0 {
                    format!(" + {capitaux}")
                } else {
                    String::new()
                };
                msg.push_str(&format!(" — vente conclue : {libelle} ({po}{cap})"));
            }
            C::UsureCapitaux { localite } => msg.push_str(&format!(
                " — usure des capitaux à {localite} (−1 par type, GC p. 82)"
            )),
            C::TestPrestigeRequis { localite, dd } => {
                alerte = true;
                msg.push_str(&format!(
                    " — retour à {localite} après une longue absence : test de prestige DD {dd}"
                ));
            }
            C::TresorerieNegative { localite } => {
                alerte = true;
                msg.push_str(&format!(" — trésorerie de {localite} négative (salaires)"));
            }
        }
    }
    (msg, alerte)
}
