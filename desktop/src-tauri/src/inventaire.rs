//! Commandes de l'inventaire et de l'équipement : ce fichier ne fait que relier l'interface au
//! cœur (`inventaire`, `campagne::equipement_fiche`, `vues::inventaire`).

use crate::session::Retour;
use crate::{session, S};
use campagne::catalogue_inventaire::{catalogue_complet, emplacements_par_defaut};
use campagne::equipement_fiche::{ref_personnage, PROPRIETAIRE_PERSONNAGE};
use campagne::modele::IdPersonnage;
use domaine::Montant;
use inventaire::{Commande, Conteneur, Id, Modele, Poids, Proprietaire};
use vues::inventaire as vi;

#[tauri::command]
pub fn page_inventaire(s: S) -> Option<vi::PageInventaire> {
    session(&s).avec_etat(vi::page_inventaire)
}

#[tauri::command]
pub fn catalogue_objets(
    requete: String,
    type_objet: Option<String>,
    limite: Option<usize>,
) -> Vec<vi::ModeleVue> {
    vi::rechercher_catalogue(&requete, type_objet.as_deref(), limite.unwrap_or(60))
}

#[tauri::command]
pub fn types_objets() -> Vec<String> {
    vi::types_du_catalogue()
}

/// Une commande du moteur, telle que l'interface la décrit (déplacer, jeter, équiper…).
#[tauri::command]
pub fn inventaire_commande(s: S, commande: Commande, message: String) -> Retour {
    session(&s).executer_inventaires(vec![commande], &message)
}

fn conteneur_illimite() -> Conteneur {
    Conteneur {
        capacite: None,
        types_autorises: None,
        contenu_compte: true,
    }
}

/// Crée l'inventaire d'un personnage (et définit les emplacements du corps la première fois).
#[tauri::command]
pub fn creer_inventaire_personnage(s: S, pj: u32) -> Retour {
    let mut x = session(&s);
    let Some((nom, emplacements_definis)) = x.avec_etat(|e| {
        (
            e.pj(IdPersonnage(pj)).map(|p| p.nom.clone()),
            !e.inventaire.regles_emplacements().emplacements.is_empty(),
        )
    }) else {
        return Retour::erreur("Aucune campagne ouverte");
    };
    let Ok(nom) = nom else {
        return Retour::erreur("Personnage inconnu");
    };
    let mut commandes = Vec::new();
    if !emplacements_definis {
        commandes.push(Commande::DefinirEmplacements(emplacements_par_defaut()));
    }
    commandes.push(Commande::CreerInventaire {
        proprietaire: Proprietaire {
            type_id: PROPRIETAIRE_PERSONNAGE.to_string(),
            ref_id: ref_personnage(IdPersonnage(pj)),
        },
        conteneur: conteneur_illimite(),
    });
    x.executer_inventaires(commandes, &format!("Inventaire de {nom} créé"))
}

/// Commandes pour que le modèle du catalogue existe dans la campagne, puis pour l'utiliser.
fn avec_modele(x: &mut session::Session, modele: &Modele, suite: Commande) -> Vec<Commande> {
    let connu = x
        .avec_etat(|e| e.inventaire.modele(&modele.id).is_some())
        .unwrap_or(false);
    let mut commandes = Vec::new();
    if !connu {
        commandes.push(Commande::AjouterModele(modele.clone()));
    }
    commandes.push(suite);
    commandes
}

fn modele_du_catalogue(id: &str) -> Option<&'static Modele> {
    catalogue_complet().iter().find(|m| m.id == id)
}

/// Ajoute `quantite` exemplaires d'un objet du catalogue dans le conteneur `dans`.
#[tauri::command]
pub fn ajouter_objet_catalogue(s: S, dans: Id, modele: String, quantite: u32) -> Retour {
    let Some(m) = modele_du_catalogue(&modele) else {
        return Retour::erreur("Objet inconnu du catalogue");
    };
    let mut x = session(&s);
    let commandes = avec_modele(
        &mut x,
        m,
        Commande::Ajouter {
            modele: m.id.clone(),
            quantite: quantite.max(1),
            dans,
        },
    );
    let message = format!("{} × {} ajouté", quantite.max(1), m.nom);
    x.executer_inventaires(commandes, &message)
}

/// Ajoute un objet du catalogue qui sert de conteneur (sac, coffre…), avec sa capacité éventuelle.
#[tauri::command]
pub fn ajouter_conteneur(
    s: S,
    dans: Id,
    modele: String,
    capacite_grammes: Option<i64>,
    extradimensionnel: bool,
) -> Retour {
    let Some(m) = modele_du_catalogue(&modele) else {
        return Retour::erreur("Objet inconnu du catalogue");
    };
    let mut x = session(&s);
    let commandes = avec_modele(
        &mut x,
        m,
        Commande::AjouterConteneur {
            modele: m.id.clone(),
            conteneur: Conteneur {
                capacite: capacite_grammes.filter(|g| *g > 0).map(Poids),
                types_autorises: None,
                contenu_compte: !extradimensionnel,
            },
            dans,
        },
    );
    let message = format!("{} ajouté comme conteneur", m.nom);
    x.executer_inventaires(commandes, &message)
}

/// Saisie libre : un objet qui n'est pas dans le catalogue.
#[tauri::command]
pub fn ajouter_objet_libre(
    s: S,
    dans: Id,
    nom: String,
    type_objet: String,
    valeur: String,
    poids_grammes: i64,
    quantite: u32,
) -> Retour {
    let nom = nom.trim().to_string();
    if nom.is_empty() {
        return Retour::erreur("Donnez un nom à l'objet");
    }
    let valeur = if valeur.trim().is_empty() {
        None
    } else {
        match Montant::depuis_texte(&valeur) {
            Some(m) => Some(m),
            None => return Retour::erreur("Valeur illisible (exemples : 12 po, 4 po 7 pa, 5 pc)"),
        }
    };
    let mut x = session(&s);
    let mut n = 1;
    let id = loop {
        let id = format!("libre:{n}");
        if !x
            .avec_etat(|e| e.inventaire.modele(&id).is_some())
            .unwrap_or(false)
        {
            break id;
        }
        n += 1;
    };
    let modele = Modele {
        id: id.clone(),
        nom: nom.clone(),
        type_objet: if type_objet.trim().is_empty() {
            "divers".to_string()
        } else {
            type_objet.trim().to_string()
        },
        valeur,
        poids: Some(Poids(poids_grammes.max(0))),
        source: "saisie libre".to_string(),
        proprietes: Default::default(),
    };
    let commandes = vec![
        Commande::AjouterModele(modele),
        Commande::Ajouter {
            modele: id,
            quantite: quantite.max(1),
            dans,
        },
    ];
    x.executer_inventaires(commandes, &format!("{nom} ajouté"))
}
