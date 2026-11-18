//! Calcul de fabrication d'objets magiques : ce fichier relie l'interface au cœur
//! (`regles::fabrication`). Aucune règle ici.

use crate::session::Retour;
use crate::{session, S};
use campagne::evenements::Evenement;
use campagne::modele::IdLocalite;
use domaine::{CoutCapitaux, Montant};
use regles::fabrication::{self as f, Issue, Paiement, Progression};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct EntreeFabrication {
    pub prix_po: i64,
    pub niveau_lanceur: u32,
    pub conditions_manquantes: u32,
    pub potion_ou_parchemin: bool,
    pub acceleree: bool,
    pub po_payes: i64,
    pub marchandises: u32,
    pub magie: u32,
    /// Jours d'intermède déjà passés sur cet objet.
    pub jours_faits: u32,
    /// Résultat du d20 + bonus, saisi une fois la fabrication terminée.
    pub jet: Option<i32>,
}

#[derive(Serialize)]
pub struct SortieFabrication {
    pub dd: i32,
    pub heures_requises: u32,
    pub jours_requis: u32,
    pub jours_restants: u32,
    pub terminee: bool,
    /// Fournitures à payer, en pa (1 po = 10 pa).
    pub fournitures_pa: i64,
    pub paiement_pa: i64,
    pub suffisant: bool,
    /// Issue du test final, si le jet est saisi.
    pub issue: Option<&'static str>,
}

#[tauri::command]
pub fn fabrication_calculer(entree: EntreeFabrication) -> SortieFabrication {
    let dd = f::dd(entree.niveau_lanceur, entree.conditions_manquantes);
    let heures = f::heures_requises(entree.prix_po, entree.potion_ou_parchemin, entree.acceleree);
    let mut progression = Progression::nouvelle(heures);
    for _ in 0..entree.jours_faits {
        progression.avancer_jour();
    }
    let fournitures = f::fournitures_pa(entree.prix_po);
    let paiement = Paiement {
        po: entree.po_payes,
        marchandises: entree.marchandises,
        magie: entree.magie,
    }
    .valeur_pa();
    let issue = entree.jet.filter(|_| progression.terminee()).map(|jet| match f::issue_test(jet, dd) {
        Issue::Reussi => "reussi",
        Issue::Echec => "echec",
        Issue::Maudit => "maudit",
    });
    SortieFabrication {
        dd,
        heures_requises: heures,
        jours_requis: f::jours_requis(heures),
        jours_restants: progression.jours_restants(),
        terminee: progression.terminee(),
        fournitures_pa: fournitures,
        paiement_pa: paiement,
        suffisant: paiement >= fournitures,
        issue,
    }
}

/// Une journée de fabrication, enregistrée dans la campagne (journal).
#[derive(Deserialize)]
pub struct EntreeJour {
    pub cle: String,
    pub nom: String,
    pub localite: u32,
    pub prix_po: i64,
    pub niveau_lanceur: u32,
    pub conditions_manquantes: u32,
    pub potion_ou_parchemin: bool,
    pub acceleree: bool,
    /// Payés le premier jour seulement, avec les Marchandises et la Magie de la localité.
    pub po_payes: i64,
    pub marchandises: u32,
    pub magie: u32,
    /// d20 + bonus, saisi une fois la fabrication terminée.
    pub jet: Option<i32>,
}

#[derive(Serialize)]
pub struct FabricationVue {
    pub cle: String,
    pub nom: String,
    pub heures_requises: u32,
    pub heures_faites: u32,
    pub terminee: bool,
    pub issue: Option<String>,
}

#[tauri::command]
pub fn fabrications_liste(s: S) -> Vec<FabricationVue> {
    session(&s)
        .avec_etat(|e| {
            e.fabrications
                .iter()
                .map(|(cle, fab)| FabricationVue {
                    cle: cle.clone(),
                    nom: fab.nom.clone(),
                    heures_requises: fab.heures_requises,
                    heures_faites: fab.heures_faites,
                    terminee: fab.heures_faites >= fab.heures_requises,
                    issue: fab.issue.clone(),
                })
                .collect()
        })
        .unwrap_or_default()
}

#[tauri::command]
pub fn fabrication_avancer(s: S, entree: EntreeJour) -> Retour {
    let existante = session(&s)
        .avec_etat(|e| e.fabrications.get(&entree.cle).map(|x| (x.heures_faites, x.issue.clone())))
        .flatten();
    let premier_jour = existante.is_none();
    let (faites_avant, deja_issue) = existante.unwrap_or((0, None));
    if deja_issue.is_some() {
        return Retour::erreur("Cette fabrication est déjà terminée");
    }
    let dd = f::dd(entree.niveau_lanceur, entree.conditions_manquantes);
    let heures = f::heures_requises(entree.prix_po, entree.potion_ou_parchemin, entree.acceleree);
    let mut progression = Progression {
        heures_requises: heures,
        heures_faites: faites_avant,
    };
    progression.avancer_jour();
    let paiement = Paiement {
        po: entree.po_payes,
        marchandises: entree.marchandises,
        magie: entree.magie,
    };
    if premier_jour && paiement.valeur_pa() < f::fournitures_pa(entree.prix_po) {
        return Retour::erreur("Paiement insuffisant pour les fournitures (voir le calcul)");
    }
    let issue = entree
        .jet
        .filter(|_| progression.terminee())
        .map(|jet| match f::issue_test(jet, dd) {
            Issue::Reussi => "reussi".to_string(),
            Issue::Echec => "echec".to_string(),
            Issue::Maudit => "maudit".to_string(),
        });
    let libelle = format!("Fabrication : {}", entree.nom);
    let mut evenements = Vec::new();
    if premier_jour {
        evenements.push(Evenement::AjustementLocalite {
            localite: IdLocalite(entree.localite),
            delta_po: Montant::po(-entree.po_payes),
            ajout: CoutCapitaux::default(),
            retrait: CoutCapitaux {
                marchandises: entree.marchandises,
                influence: 0,
                travail: 0,
                magie: entree.magie,
            },
            motif: libelle.clone(),
        });
    }
    evenements.push(Evenement::JoursAvances { nombre: 1 });
    evenements.push(Evenement::FabricationAvancee {
        cle: entree.cle.clone(),
        nom: entree.nom.clone(),
        localite: IdLocalite(entree.localite),
        prix_po: entree.prix_po,
        dd,
        heures_requises: heures,
        heures_faites: progression.heures_faites,
        issue,
    });
    session(&s).executer(
        Evenement::Groupe {
            libelle: libelle.clone(),
            evenements,
        },
        &libelle,
    )
}
