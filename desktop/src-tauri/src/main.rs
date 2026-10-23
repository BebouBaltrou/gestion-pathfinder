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
