//! Application de bureau (Windows, Linux) : fenêtre native Tauri, interface web locale.
//! Les commandes ci-dessous sont la seule porte entre l'interface et le cœur de règles.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod session;

use campagne::evenements::Evenement;
use campagne::modele::IdLocalite;
use serde::Serialize;
use session::{FichierCampagne, Retour, Session};
use std::path::Path;
use std::sync::Mutex;
use tauri::State;

struct Etat(Mutex<Session>);

type S<'a> = State<'a, Etat>;

/// Ce que la coquille de l'interface affiche en permanence.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct EtatApp {
    entete: Option<vues::Entete>,
    campagnes: Vec<FichierCampagne>,
    dossier: String,
    /// (actions, jours) d'avance de la simulation sur la campagne réelle.
    simulation: Option<(i64, i64)>,
}

fn session<'a>(s: &'a State<'_, Etat>) -> std::sync::MutexGuard<'a, Session> {
    // une commande qui a paniqué ne doit pas bloquer les suivantes
    s.0.lock().unwrap_or_else(|e| e.into_inner())
}

#[tauri::command]
fn etat_app(s: S) -> EtatApp {
    let mut x = session(&s);
    let simulation = x.ecart_simulation();
    let r = x.r.clone();
    EtatApp {
        entete: x.campagne.as_mut().map(|c| vues::entete(c, &r)),
        campagnes: x.lister(),
        dossier: x.dossier.to_string_lossy().to_string(),
        simulation,
    }
}

#[tauri::command]
fn ouvrir_campagne(s: S, chemin: String) -> Retour {
    session(&s).ouvrir(Path::new(&chemin))
}

#[tauri::command]
fn creer_campagne(s: S, nom: String) -> Retour {
    session(&s).creer(&nom)
}

#[tauri::command]
fn tableau_de_bord(s: S) -> Option<vues::tableau::TableauDeBord> {
    let mut x = session(&s);
    let r = x.r.clone();
    x.campagne
        .as_mut()
        .map(|c| vues::tableau::tableau_de_bord(c, &r))
}

// ---------------------------------------------------------------- localités

#[tauri::command]
fn page_localites(s: S) -> Option<vues::localites::PageLocalites> {
    let mut x = session(&s);
    let r = x.r.clone();
    x.campagne
        .as_mut()
        .map(|c| vues::localites::page_localites(c, &r))
}

#[tauri::command]
fn categorie_suggeree(s: S, population: String) -> Option<vues::localites::ChoixCategorie> {
    vues::localites::categorie_suggeree(&population, &session(&s).r)
}

#[tauri::command]
fn lire_montant(texte: String) -> Option<vues::Somme> {
    vues::localites::lire_montant(&texte)
}

/// Exécute l'événement construit à partir d'une saisie, ou signale la saisie refusée.
fn executer_saisie(x: &mut Session, ev: Result<Evenement, String>, succes: &str) -> Retour {
    match ev {
        Ok(ev) => x.executer(ev, succes),
        Err(e) => Retour::erreur(e),
    }
}

#[tauri::command]
fn creer_localite(s: S, saisie: vues::localites::SaisieLocalite) -> Retour {
    let mut x = session(&s);
    let Some(id) = x.campagne.as_ref().map(|c| c.nouvel_id_localite()) else {
        return Retour::erreur("Aucune campagne ouverte");
    };
    let ev = saisie.creation(id, &x.r);
    executer_saisie(
        &mut x,
        ev,
        &format!("Localité « {} » créée", saisie.nom.trim()),
    )
}

#[tauri::command]
fn modifier_localite(s: S, id: u32, saisie: vues::localites::SaisieLocalite) -> Retour {
    let mut x = session(&s);
    let ev = saisie.modification(IdLocalite(id), &x.r);
    executer_saisie(&mut x, ev, "Localité modifiée")
}

#[tauri::command]
fn mouvement_po(s: S, injection: bool, pj: u32, localite: u32, montant: String) -> Retour {
    let mut x = session(&s);
    let ev = vues::localites::mouvement_po(injection, pj, localite, &montant);
    let succes = match (&ev, injection) {
        (Ok(Evenement::InjectionPo { montant, .. }), true) => format!("{montant} injectés"),
        (Ok(Evenement::RetraitPo { montant, .. }), false) => format!("{montant} retirés"),
        _ => String::new(),
    };
    executer_saisie(&mut x, ev, &succes)
}

#[tauri::command]
fn transfert(
    s: S,
    de: u32,
    vers: u32,
    ressource: String,
    quantite: String,
    duree: String,
) -> Retour {
    let mut x = session(&s);
    let Some(id) = x.campagne.as_ref().map(|c| c.nouvel_id_transfert()) else {
        return Retour::erreur("Aucune campagne ouverte");
    };
    let ev = vues::localites::transfert(id, de, vers, &ressource, &quantite, &duree);
    let succes = match &ev {
        Ok(Evenement::TransfertEnvoye { lot, .. }) => format!("Transfert de {lot} envoyé"),
        _ => String::new(),
    };
    executer_saisie(&mut x, ev, &succes)
}

#[tauri::command]
fn ajustement(
    s: S,
    localite: u32,
    delta_po: String,
    ajout: vues::Capitaux,
    retrait: vues::Capitaux,
    motif: String,
) -> Retour {
    let mut x = session(&s);
    let ev = vues::localites::ajustement(localite, &delta_po, ajout, retrait, &motif);
    executer_saisie(&mut x, ev, "Ajustement enregistré")
}

// ---------------------------------------------------------------- personnages

#[tauri::command]
fn page_personnages(s: S) -> Option<vues::personnages::PagePersonnages> {
    let mut x = session(&s);
    let r = x.r.clone();
    x.campagne
        .as_mut()
        .map(|c| vues::personnages::page_personnages(c, &r))
}

#[tauri::command]
fn creer_personnage(s: S, nom: String, compte: String, prestige: String) -> Retour {
    let mut x = session(&s);
    let Some(id) = x.campagne.as_ref().map(|c| c.nouvel_id_personnage()) else {
        return Retour::erreur("Aucune campagne ouverte");
    };
    let ev = vues::personnages::creation(id, &nom, &compte, &prestige);
    executer_saisie(&mut x, ev, &format!("Personnage « {} » créé", nom.trim()))
}

#[tauri::command]
fn modifier_personnage(s: S, id: u32, nom: String, prestige: String) -> Retour {
    let mut x = session(&s);
    let ev = vues::personnages::modification(id, &nom, &prestige);
    executer_saisie(&mut x, ev, "Personnage modifié")
}

#[tauri::command]
fn deplacer_personnage(s: S, id: u32, vers: Option<u32>) -> Retour {
    session(&s).executer(
        vues::personnages::deplacement(id, vers),
        "Déplacement enregistré",
    )
}

#[tauri::command]
fn ajuster_compte(s: S, pj: u32, delta: String, motif: String) -> Retour {
    let mut x = session(&s);
    let ev = vues::personnages::ajustement_compte(pj, &delta, &motif);
    executer_saisie(&mut x, ev, "Compte mis à jour")
}

// ---------------------------------------------------------------- temps

#[tauri::command]
fn avancer(s: S, jours: u32) -> Retour {
    let succes = if jours == 1 {
        "Jour suivant".to_string()
    } else {
        format!("{jours} jours écoulés")
    };
    session(&s).executer(Evenement::JoursAvances { nombre: jours }, &succes)
}

#[tauri::command]
fn annuler(s: S) -> Retour {
    session(&s).annuler()
}

#[tauri::command]
fn retablir(s: S) -> Retour {
    session(&s).retablir()
}

#[tauri::command]
fn resoudre_prestige(s: S, localite: u32, reussi: bool) -> Retour {
    session(&s).executer(
        Evenement::TestPrestigeResolu {
            localite: IdLocalite(localite),
            reussi,
        },
        if reussi {
            "Test de prestige réussi"
        } else {
            "Test de prestige raté : voyez avec le MJ ce qu'il advient des entreprises"
        },
    )
}

#[tauri::command]
fn simulation(s: S, action: String) -> Retour {
    let mut x = session(&s);
    match action.as_str() {
        "demarrer" => x.demarrer_simulation(),
        "abandonner" => x.abandonner_simulation(),
        "garder" => x.garder_simulation(),
        _ => Retour::erreur("action de simulation inconnue"),
    }
}

/// Essai automatique de bout en bout (compilation sur GitHub) : l'interface enchaîne
/// quelques actions puis écrit son rapport dans le fichier indiqué et ferme l'application.
fn fichier_autotest() -> Option<String> {
    std::env::var("GESTION_PATHFINDER_AUTOTEST").ok()
}

#[tauri::command]
fn autotest_actif() -> bool {
    fichier_autotest().is_some()
}

#[tauri::command]
fn autotest_fin(app: tauri::AppHandle, rapport: String) {
    if let Some(f) = fichier_autotest() {
        let _ = std::fs::write(f, rapport);
        app.exit(0);
    }
}

fn main() {
    tauri::Builder::default()
        .manage(Etat(Mutex::new(Session::demarrer())))
        .invoke_handler(tauri::generate_handler![
            etat_app,
            ouvrir_campagne,
            creer_campagne,
            tableau_de_bord,
            page_localites,
            categorie_suggeree,
            lire_montant,
            creer_localite,
            modifier_localite,
            mouvement_po,
            transfert,
            ajustement,
            page_personnages,
            creer_personnage,
            modifier_personnage,
            deplacer_personnage,
            ajuster_compte,
            avancer,
            annuler,
            retablir,
            resoudre_prestige,
            simulation,
            autotest_actif,
            autotest_fin
        ])
        .run(tauri::generate_context!())
        .expect("impossible de lancer l'application");
}
