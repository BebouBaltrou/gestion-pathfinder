//! Application de bureau (Windows, Linux) : fenêtre native Tauri, interface web locale.
//! Les commandes ci-dessous sont la seule porte entre l'interface et le cœur de règles.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod aventure;
mod export;
mod import_fiche;
mod inventaire;
mod notes;
mod personnages;
mod session;

use campagne::evenements::Evenement;
use campagne::modele::IdLocalite;
use serde::Serialize;
use session::{FichierCampagne, Retour, Session};
use std::path::Path;
use std::sync::Mutex;
use tauri::State;

pub(crate) struct Etat(Mutex<Session>);

pub(crate) type S<'a> = State<'a, Etat>;

/// Ce que la coquille de l'interface affiche en permanence.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct EtatApp {
    entete: Option<vues::Entete>,
    campagnes: Vec<FichierCampagne>,
    dossier: String,
    /// (actions, jours) d'avance de la simulation sur la campagne réelle.
    simulation: Option<(i64, i64)>,
    /// Un combat est en cours (copie de travail) : le reste de la campagne est verrouillé.
    combat_en_cours: bool,
}

pub(crate) fn session<'a>(s: &'a State<'_, Etat>) -> std::sync::MutexGuard<'a, Session> {
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
        combat_en_cours: x.combat.is_some(),
    }
}

#[tauri::command]
fn ouvrir_campagne(s: S, chemin: String) -> Retour {
    session(&s).ouvrir(Path::new(&chemin))
}

#[tauri::command]
fn creer_campagne(
    s: S,
    nom: String,
    calendrier: Option<vues::parametres::SaisieCalendrier>,
) -> Retour {
    session(&s).creer(&nom, calendrier.as_ref())
}

#[tauri::command]
fn calendrier_par_defaut() -> vues::parametres::VueCalendrier {
    vues::parametres::calendrier_par_defaut()
}

#[tauri::command]
fn exporter_campagne(s: S) -> Retour {
    export::exporter(&mut session(&s), export::Export::Campagne)
}

#[tauri::command]
fn exporter_recapitulatif(s: S) -> Retour {
    export::exporter(&mut session(&s), export::Export::Recapitulatif)
}

#[tauri::command]
fn exporter_historique(s: S) -> Retour {
    export::exporter(&mut session(&s), export::Export::Historique)
}

#[tauri::command]
fn importer_campagne(s: S) -> Retour {
    export::importer(&mut session(&s))
}

#[tauri::command]
fn ouvrir_dossier(s: S) -> Retour {
    export::ouvrir_dossier(&session(&s))
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
fn avertissement_suppression_localite(s: S, id: u32) -> String {
    let mut x = session(&s);
    let r = x.r.clone();
    match x.campagne.as_mut() {
        Some(c) => vues::localites::avertissement_suppression(c, &r, id),
        None => "Aucune campagne ouverte".into(),
    }
}

#[tauri::command]
fn supprimer_localite(s: S, id: u32) -> Retour {
    session(&s).executer(
        Evenement::LocaliteSupprimee { id: IdLocalite(id) },
        "Localité supprimée (« Annuler » la ramène)",
    )
}

#[tauri::command]
fn supprimer_personnage(s: S, id: u32) -> Retour {
    session(&s).executer(
        Evenement::PersonnageSupprime {
            id: campagne::modele::IdPersonnage(id),
        },
        "Personnage supprimé (« Annuler » le ramène)",
    )
}

#[tauri::command]
fn supprimer_campagne(s: S, chemin: String) -> Retour {
    session(&s).supprimer_campagne(Path::new(&chemin))
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

// ---------------------------------------------------------------- bâtiments

#[tauri::command]
fn page_batiments(s: S) -> Option<vues::batiments::PageBatiments> {
    let mut x = session(&s);
    let (r, m) = (x.r.clone(), x.modeles.clone());
    x.campagne
        .as_mut()
        .map(|c| vues::batiments::page_batiments(c, &r, &m))
}

#[tauri::command]
fn apercu_creation_batiment(
    s: S,
    saisie: vues::batiments::SaisieBatiment,
) -> Option<vues::batiments::ApercuCreation> {
    let mut x = session(&s);
    let (r, m) = (x.r.clone(), x.modeles.clone());
    x.campagne
        .as_mut()
        .and_then(|c| vues::batiments::apercu_creation(c, &r, &m, &saisie))
}

/// Exécute un événement construit par le cœur (message de succès compris).
fn executer_construit(x: &mut Session, ev: Result<(Evenement, String), String>) -> Retour {
    match ev {
        Ok((ev, msg)) => x.executer(ev, &msg),
        Err(e) => Retour::erreur(e),
    }
}

#[tauri::command]
fn creer_batiment(s: S, saisie: vues::batiments::SaisieBatiment) -> Retour {
    let mut x = session(&s);
    let (r, m) = (x.r.clone(), x.modeles.clone());
    let Some(c) = x.campagne.as_mut() else {
        return Retour::erreur("Aucune campagne ouverte");
    };
    let ev = vues::batiments::creation(c, &r, &m, &saisie);
    executer_construit(&mut x, ev)
}

#[tauri::command]
fn fiche_batiment(s: S, id: u32) -> Option<vues::batiments::FicheBatiment> {
    let mut x = session(&s);
    let r = x.r.clone();
    x.campagne
        .as_mut()
        .and_then(|c| vues::batiments::fiche_batiment(c, &r, id))
}

#[tauri::command]
fn action_batiment(s: S, action: vues::actions_batiments::ActionBatiment) -> Retour {
    let mut x = session(&s);
    let r = x.r.clone();
    let Some(c) = x.campagne.as_mut() else {
        return Retour::erreur("Aucune campagne ouverte");
    };
    let ev = vues::actions_batiments::action(c, &r, action);
    executer_construit(&mut x, ev)
}

#[tauri::command]
fn mise_en_vente(
    s: S,
    cible: vues::ventes::Cible,
    jours: String,
    influence: u32,
    po: String,
) -> Retour {
    let mut x = session(&s);
    let r = x.r.clone();
    let Some(c) = x.campagne.as_mut() else {
        return Retour::erreur("Aucune campagne ouverte");
    };
    let etat = c.etat(&r).clone();
    let ev = vues::ventes::mise_en_vente(&etat, &r, cible, &jours, influence, &po);
    executer_construit(&mut x, ev)
}

/// Enregistre un bâtiment comme modèle personnel (un modèle du même nom est remplacé).
#[tauri::command]
fn enregistrer_modele_batiment(s: S, id: u32) -> Retour {
    let mut x = session(&s);
    let r = x.r.clone();
    let Some(c) = x.campagne.as_mut() else {
        return Retour::erreur("Aucune campagne ouverte");
    };
    let etat = c.etat(&r).clone();
    let Some(b) = etat.batiments.get(&campagne::modele::IdBatiment(id)) else {
        return Retour::erreur("Bâtiment introuvable");
    };
    let m = campagne::modeles_perso::BatimentPerso::depuis(&etat, b, &b.nom);
    let nom = b.nom.clone();
    x.modeles.ajouter_batiment(m);
    match x.enregistrer_modeles() {
        Ok(()) => Retour::ok(format!("Modèle personnel « {nom} » enregistré")),
        Err(e) => Retour::erreur(e),
    }
}

#[tauri::command]
fn supprimer_modele_batiment(s: S, id: String) -> Retour {
    let mut x = session(&s);
    let Some(nom) = id.strip_prefix(campagne::modeles_perso::PREFIXE) else {
        return Retour::erreur("Ce modèle est officiel : il ne peut pas être supprimé");
    };
    let nom = nom.to_string();
    x.modeles.batiments.retain(|m| m.nom != nom);
    match x.enregistrer_modeles() {
        Ok(()) => Retour::ok(format!("Modèle personnel « {nom} » supprimé")),
        Err(e) => Retour::erreur(e),
    }
}

// ---------------------------------------------------------------- organisations

#[tauri::command]
fn page_organisations(s: S) -> Option<vues::organisations::PageOrganisations> {
    let mut x = session(&s);
    let (r, m) = (x.r.clone(), x.modeles.clone());
    x.campagne
        .as_mut()
        .map(|c| vues::organisations::page_organisations(c, &r, &m))
}

#[tauri::command]
fn apercu_creation_organisation(
    s: S,
    saisie: vues::batiments::SaisieBatiment,
) -> Option<vues::batiments::ApercuCreation> {
    let mut x = session(&s);
    let (r, m) = (x.r.clone(), x.modeles.clone());
    x.campagne
        .as_mut()
        .and_then(|c| vues::organisations::apercu_creation(c, &r, &m, &saisie))
}

#[tauri::command]
fn creer_organisation(s: S, saisie: vues::batiments::SaisieBatiment) -> Retour {
    let mut x = session(&s);
    let (r, m) = (x.r.clone(), x.modeles.clone());
    let Some(c) = x.campagne.as_mut() else {
        return Retour::erreur("Aucune campagne ouverte");
    };
    let ev = vues::organisations::creation(c, &r, &m, &saisie);
    executer_construit(&mut x, ev)
}

#[tauri::command]
fn fiche_organisation(s: S, id: u32) -> Option<vues::organisations::FicheOrganisation> {
    let mut x = session(&s);
    let r = x.r.clone();
    x.campagne
        .as_mut()
        .and_then(|c| vues::organisations::fiche_organisation(c, &r, id))
}

#[tauri::command]
fn action_organisation(s: S, action: vues::organisations::ActionOrganisation) -> Retour {
    let mut x = session(&s);
    let r = x.r.clone();
    let Some(c) = x.campagne.as_mut() else {
        return Retour::erreur("Aucune campagne ouverte");
    };
    let ev = vues::organisations::action(c, &r, action);
    executer_construit(&mut x, ev)
}

#[tauri::command]
fn enregistrer_modele_organisation(s: S, id: u32) -> Retour {
    let mut x = session(&s);
    let r = x.r.clone();
    let Some(c) = x.campagne.as_mut() else {
        return Retour::erreur("Aucune campagne ouverte");
    };
    let etat = c.etat(&r).clone();
    let Some(o) = etat
        .organisations
        .get(&campagne::modele::IdOrganisation(id))
    else {
        return Retour::erreur("Organisation introuvable");
    };
    let m = campagne::modeles_perso::OrganisationPerso::depuis(&etat, o, &o.nom);
    let nom = o.nom.clone();
    x.modeles.ajouter_organisation(m);
    match x.enregistrer_modeles() {
        Ok(()) => Retour::ok(format!("Modèle personnel « {nom} » enregistré")),
        Err(e) => Retour::erreur(e),
    }
}

#[tauri::command]
fn supprimer_modele_organisation(s: S, id: String) -> Retour {
    let mut x = session(&s);
    let Some(nom) = id.strip_prefix(campagne::modeles_perso::PREFIXE) else {
        return Retour::erreur("Ce modèle est officiel : il ne peut pas être supprimé");
    };
    let nom = nom.to_string();
    x.modeles.organisations.retain(|m| m.nom != nom);
    match x.enregistrer_modeles() {
        Ok(()) => Retour::ok(format!("Modèle personnel « {nom} » supprimé")),
        Err(e) => Retour::erreur(e),
    }
}

// ---------------------------------------------------------------- gestionnaires

#[tauri::command]
fn page_gestionnaires(s: S) -> Option<vues::gestionnaires::PageGestionnaires> {
    let mut x = session(&s);
    let r = x.r.clone();
    x.campagne
        .as_mut()
        .map(|c| vues::gestionnaires::page_gestionnaires(c, &r))
}

#[tauri::command]
fn action_gestionnaire(s: S, action: vues::gestionnaires::ActionGestionnaire) -> Retour {
    let mut x = session(&s);
    let r = x.r.clone();
    let Some(c) = x.campagne.as_mut() else {
        return Retour::erreur("Aucune campagne ouverte");
    };
    let ev = vues::gestionnaires::action(c, &r, action);
    executer_construit(&mut x, ev)
}

// ---------------------------------------------------------------- historique, points, paramètres

#[tauri::command]
fn page_historique(s: S) -> Option<vues::historique::PageHistorique> {
    let mut x = session(&s);
    let r = x.r.clone();
    x.campagne
        .as_mut()
        .map(|c| vues::historique::page_historique(c, &r))
}

#[tauri::command]
fn page_points(s: S) -> vues::points::PagePoints {
    let x = session(&s);
    let afficher = x
        .campagne
        .as_ref()
        .is_none_or(|c| c.options.afficher_avertissements);
    vues::points::page_points(&x.r, afficher)
}

#[tauri::command]
fn page_referentiel(s: S) -> vues::referentiel::PageReferentiel {
    vues::referentiel::page_referentiel(&session(&s).r)
}

#[tauri::command]
fn page_modeles(s: S) -> vues::modeles::PageModeles {
    vues::modeles::page_modeles(&session(&s).modeles)
}

#[tauri::command]
fn page_parametres(s: S) -> Option<vues::parametres::PageParametres> {
    let mut x = session(&s);
    let r = x.r.clone();
    x.campagne
        .as_mut()
        .map(|c| vues::parametres::page_parametres(c, &r))
}

#[tauri::command]
fn regler(s: S, id: String, actif: bool) -> Retour {
    use vues::parametres::ResultatReglage;
    let mut x = session(&s);
    let Some(c) = x.campagne.as_mut() else {
        return Retour::erreur("Aucune campagne ouverte");
    };
    match vues::parametres::regler(c, &id, actif) {
        Ok(ResultatReglage::Evenement(ev, msg)) => x.executer(*ev, &msg),
        Ok(ResultatReglage::Enregistre(msg)) => match x.enregistrer() {
            Ok(()) => Retour::ok(msg),
            Err(e) => Retour::erreur(e),
        },
        Err(e) => Retour::erreur(e),
    }
}

#[tauri::command]
fn regler_calendrier(
    s: S,
    jour_semaine: u8,
    jour: u8,
    mois: u8,
    annee: i32,
    bissextile: String,
) -> Retour {
    let mut x = session(&s);
    let Some(c) = x.campagne.as_mut() else {
        return Retour::erreur("Aucune campagne ouverte");
    };
    match vues::parametres::regler_calendrier(c, jour_semaine, jour, mois, annee, &bissextile) {
        Ok(()) => match x.enregistrer() {
            Ok(()) => Retour::ok("Calendrier enregistré"),
            Err(e) => Retour::erreur(e),
        },
        Err(e) => Retour::erreur(e),
    }
}

// ---------------------------------------------------------------- recettes

#[tauri::command]
fn vue_recettes(
    s: S,
    entree: vues::recettes::EntreeRecettes,
) -> Option<vues::recettes::VueRecettes> {
    let mut x = session(&s);
    let r = x.r.clone();
    x.campagne
        .as_mut()
        .map(|c| vues::recettes::vue_recettes(c, &r, &entree))
}

#[tauri::command]
fn encaisser_recettes(s: S, entree: vues::recettes::EntreeRecettes) -> Retour {
    let mut x = session(&s);
    let r = x.r.clone();
    let Some(c) = x.campagne.as_mut() else {
        return Retour::erreur("Aucune campagne ouverte");
    };
    let ev = vues::recettes::encaisser(c, &r, &entree);
    executer_construit(&mut x, ev)
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
            calendrier_par_defaut,
            exporter_campagne,
            exporter_recapitulatif,
            exporter_historique,
            importer_campagne,
            ouvrir_dossier,
            tableau_de_bord,
            page_localites,
            categorie_suggeree,
            lire_montant,
            creer_localite,
            modifier_localite,
            avertissement_suppression_localite,
            supprimer_localite,
            supprimer_personnage,
            supprimer_campagne,
            mouvement_po,
            transfert,
            ajustement,
            page_personnages,
            modifier_personnage,
            deplacer_personnage,
            ajuster_compte,
            page_batiments,
            apercu_creation_batiment,
            creer_batiment,
            fiche_batiment,
            action_batiment,
            mise_en_vente,
            enregistrer_modele_batiment,
            supprimer_modele_batiment,
            page_organisations,
            apercu_creation_organisation,
            creer_organisation,
            fiche_organisation,
            action_organisation,
            enregistrer_modele_organisation,
            supprimer_modele_organisation,
            page_gestionnaires,
            action_gestionnaire,
            page_historique,
            page_points,
            page_referentiel,
            page_parametres,
            page_modeles,
            regler,
            regler_calendrier,
            vue_recettes,
            encaisser_recettes,
            avancer,
            annuler,
            retablir,
            resoudre_prestige,
            simulation,
            aventure::aventure_vue,
            aventure::aventure_demarrer,
            aventure::aventure_terminer,
            aventure::aventure_evenement,
            aventure::aventure_actualiser_ressources,
            aventure::aventure_apercu_recul,
            aventure::combat_vue,
            aventure::combat_demarrer,
            aventure::combat_action,
            aventure::combat_annuler,
            aventure::combat_retablir,
            aventure::combat_apercu_sort,
            aventure::combat_apercu_fin,
            aventure::combat_terminer,
            aventure::catalogue_actions,
            aventure::catalogue_fp,
            aventure::catalogue_regles,
            aventure::combat_xp,
            aventure::combat_historique,
            aventure::combat_exporter,
            aventure::catalogue_etats,
            notes::page_notes,
            notes::liste_notes,
            notes::chronologie,
            notes::creer_note,
            notes::modifier_note,
            notes::epingler_note,
            notes::supprimer_note,
            notes::noter_objet,
            notes::exporter_notes,
            inventaire::page_inventaire,
            inventaire::catalogue_objets,
            inventaire::types_objets,
            inventaire::inventaire_commande,
            inventaire::creer_inventaire_personnage,
            inventaire::ajouter_objet_catalogue,
            inventaire::ajouter_conteneur,
            inventaire::ajouter_objet_libre,
            personnages::liste_personnages,
            personnages::fiche_personnage,
            personnages::catalogue_creation,
            personnages::apercu_fiche,
            personnages::creer_personnage,
            personnages::creer_fiche,
            import_fiche::choisir_fiche_pdf,
            import_fiche::importer_fiche_pdf,
            personnages::accorder_niveau_de_depart,
            personnages::prestige_don,
            personnages::prestige_evenement,
            personnages::recruter_acolyte,
            personnages::evenement_fiche,
            personnages::apercu_montee,
            personnages::essai_montee,
            personnages::regles_campagne,
            personnages::changer_regle,
            autotest_actif,
            autotest_fin
        ])
        .run(tauri::generate_context!())
        .expect("impossible de lancer l'application");
}
