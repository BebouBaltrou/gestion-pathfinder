//! Commandes du mode aventure et du mode combat : ce fichier ne fait que relier l'interface au
//! cœur (`aventure`, `combat`, `campagne::aventure`, `vues::aventure`).
//!
//! Le combat vit dans une copie de travail séparée de la campagne : il est sauvegardé après chaque
//! action dans `<campagne>.combat.json` (écriture atomique) et n'entre dans le journal de la
//! campagne qu'une fois validé (un seul événement `CombatValide`).

use crate::session::{Retour, Session};
use crate::{session, S};
use aventure::{EvenementAventure, IdPj};
use campagne::evenements::Evenement;
use campagne::modele::IdPersonnage;
use combat::{ActionCombat, Combat, Validation};
use serde::Deserialize;
use std::collections::BTreeMap;
use std::path::PathBuf;
use temps::{Duree, IdEffet};
use vues::aventure as va;

impl Session {
    fn chemin_combat(&self) -> Option<PathBuf> {
        self.chemin
            .as_ref()
            .map(|p| p.with_extension("combat.json"))
    }

    fn contextes(&mut self) -> BTreeMap<IdPj, aventure::ContextePj> {
        match self.campagne.as_mut() {
            Some(c) => c.etat(&self.r).contextes_aventure(),
            None => BTreeMap::new(),
        }
    }

    /// Reprend un combat interrompu (fichier à côté de la campagne), s'il y en a un.
    pub fn recharger_combat(&mut self) -> Result<bool, String> {
        let Some(chemin) = self.chemin_combat() else {
            return Ok(false);
        };
        if !chemin.exists() {
            return Ok(false);
        }
        let contextes = self.contextes();
        let ctx = |id: IdPj| contextes.get(&id).cloned();
        let c = combat::persistance::charger(&chemin, &ctx)?;
        self.combat = Some(c);
        Ok(true)
    }

    /// Sauvegarde le combat après une action (jamais pendant une simulation).
    fn sauver_combat(&self) -> Result<(), String> {
        if self.simulation.is_some() {
            return Ok(());
        }
        match (&self.combat, self.chemin_combat()) {
            (Some(c), Some(p)) => {
                if let Some(d) = p.parent() {
                    std::fs::create_dir_all(d).map_err(|e| e.to_string())?;
                }
                combat::persistance::sauvegarder(c, &p)
                    .map_err(|e| format!("Enregistrement du combat impossible : {e}"))
            }
            _ => Ok(()),
        }
    }

    fn supprimer_fichier_combat(&self) {
        if let Some(p) = self.chemin_combat() {
            let _ = std::fs::remove_file(p);
        }
    }

    /// Applique une action au combat, puis le sauvegarde.
    fn agir_en_combat(
        &mut self,
        f: impl FnOnce(
            &mut Combat,
            &dyn Fn(IdPj) -> Option<aventure::ContextePj>,
        ) -> Result<String, String>,
    ) -> Retour {
        if self.combat.is_none() {
            return Retour::erreur("Aucun combat en cours");
        }
        let contextes = self.contextes();
        let ctx = |id: IdPj| contextes.get(&id).cloned();
        let c = self.combat.as_mut().expect("vérifié ci-dessus");
        match f(c, &ctx) {
            Ok(m) => match self.sauver_combat() {
                Ok(()) => Retour::ok(m),
                Err(e) => Retour::erreur(e),
            },
            Err(e) => Retour::erreur(e),
        }
    }
}

// ------------------------------------------------------------------ aventure

#[tauri::command]
pub fn aventure_vue(s: S) -> va::AventureVue {
    session(&s)
        .avec_etat(va::aventure)
        .expect("une campagne est ouverte")
}

#[tauri::command]
pub fn aventure_demarrer(
    s: S,
    heure: u32,
    minute: u32,
    survie: bool,
    degats_massifs: bool,
) -> Retour {
    session(&s).executer(
        Evenement::AventureDemarree {
            heure,
            minute,
            survie,
            degats_massifs,
        },
        &format!("Aventure démarrée à {heure:02}:{minute:02}"),
    )
}

#[tauri::command]
pub fn aventure_terminer(s: S) -> Retour {
    let mut x = session(&s);
    if x.combat.is_some() {
        return Retour::erreur("Terminez ou abandonnez d'abord le combat en cours");
    }
    x.executer(Evenement::AventureTerminee, "Aventure terminée")
}

/// Événement d'aventure (temps, santé, effets, repos, objets, ressources…).
#[tauri::command]
pub fn aventure_evenement(s: S, evenement: EvenementAventure) -> Retour {
    let mut x = session(&s);
    if x.combat.is_some() {
        return Retour::erreur(
            "Un combat est en cours : l'aventure reprend à sa fin (validation ou abandon)",
        );
    }
    let message = evenement.description();
    x.executer(Evenement::Aventure(evenement), &message)
}

/// Ce qu'un recul de l'horloge toucherait (rien n'est modifié).
#[tauri::command]
pub fn aventure_apercu_recul(s: S, instant: i64) -> Option<va::ReculVue> {
    session(&s)
        .avec_etat(|e| va::apercu_recul(e, instant))
        .flatten()
}

/// Calcule les usages quotidiens des pouvoirs de lignage du personnage (3 + Charisme, paliers…).
#[tauri::command]
pub fn aventure_actualiser_ressources(s: S, pj: u32) -> Retour {
    let mut x = session(&s);
    let Some(ressources) = x.avec_etat(|e| {
        let etat = e.fiche_de(IdPersonnage(pj))?;
        let v = etat.fiche.calculer(fiches::Referentiel::partage()).ok()?;
        let lignage = etat.fiche.lignage.clone()?;
        let niveau = etat.fiche.niveau_de_classe("ensorceleur");
        let cha = v
            .modificateurs
            .get(&fiches::Carac::Cha)
            .copied()
            .unwrap_or(0);
        Some(aventure::ressources_de_lignage(&lignage, niveau, cha))
    }) else {
        return Retour::erreur("Aucune campagne ouverte");
    };
    let Some(ressources) = ressources else {
        return Retour::erreur("Ce personnage n'a pas de pouvoir de lignage à suivre");
    };
    if ressources.is_empty() {
        return Retour::erreur("Aucun pouvoir à utilisations quotidiennes à ce niveau");
    }
    x.executer(
        Evenement::Aventure(EvenementAventure::RessourcesDefinies { pj, ressources }),
        "Ressources quotidiennes mises à jour",
    )
}

// -------------------------------------------------------------------- combat

#[tauri::command]
pub fn combat_vue(s: S) -> Option<va::CombatVue> {
    let mut x = session(&s);
    let c = x.combat.clone()?;
    x.avec_etat(|e| va::combat(e, &c))
}

#[tauri::command]
pub fn combat_demarrer(s: S, engages: Vec<u32>) -> Retour {
    let mut x = session(&s);
    if x.combat.is_some() {
        return Retour::erreur("Un combat est déjà en cours");
    }
    let Some(base) = x.avec_etat(|e| e.aventure.clone()).flatten() else {
        return Retour::erreur("Démarrez d'abord l'aventure");
    };
    let contextes = x.contextes();
    let ctx = |id: IdPj| contextes.get(&id).cloned();
    match Combat::nouveau(&base, engages, &ctx) {
        Ok(c) => {
            x.combat = Some(c);
            match x.sauver_combat() {
                Ok(()) => Retour::ok("Combat commencé : saisissez les initiatives"),
                Err(e) => Retour::erreur(e),
            }
        }
        Err(e) => Retour::erreur(e.to_string()),
    }
}

#[tauri::command]
pub fn combat_action(s: S, action: ActionCombat) -> Retour {
    session(&s).agir_en_combat(|c, ctx| {
        c.appliquer(action, ctx).map_err(|e| e.to_string())?;
        Ok(c.etat()
            .journal
            .last()
            .map_or_else(|| "Fait".to_string(), |j| j.texte.clone()))
    })
}

#[tauri::command]
pub fn combat_annuler(s: S) -> Retour {
    session(&s).agir_en_combat(|c, ctx| match c.annuler(ctx) {
        Ok(true) => Ok("Action du combat annulée".into()),
        Ok(false) => Err("Rien à annuler dans le combat".into()),
        Err(e) => Err(e.to_string()),
    })
}

#[tauri::command]
pub fn combat_retablir(s: S) -> Retour {
    session(&s).agir_en_combat(|c, ctx| match c.retablir(ctx) {
        Ok(true) => Ok("Action du combat rétablie".into()),
        Ok(false) => Err("Rien à rétablir dans le combat".into()),
        Err(e) => Err(e.to_string()),
    })
}

/// Ce qu'il faut savoir (et lancer) avant de lancer un sort.
#[tauri::command]
pub fn combat_apercu_sort(
    s: S,
    pj: u32,
    sort: String,
    metamagies: Vec<String>,
    niveau_heighten: Option<u32>,
) -> Result<va::ApercuSortVue, String> {
    let mut x = session(&s);
    let contextes = x.contextes();
    let ctx = |id: IdPj| contextes.get(&id).cloned();
    let c = x.combat.as_ref().ok_or("Aucun combat en cours")?;
    c.etat()
        .apercu_sort(pj, &sort, &metamagies, niveau_heighten, &ctx)
        .map(|a| va::apercu_sort(&a))
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn combat_apercu_fin(s: S) -> Result<va::ApercuFinVue, String> {
    let mut x = session(&s);
    let contextes = x.contextes();
    let ctx = |id: IdPj| contextes.get(&id).cloned();
    let c = x.combat.clone().ok_or("Aucun combat en cours")?;
    let f = c.apercu_fin(&ctx).map_err(|e| e.to_string())?;
    x.avec_etat(|e| va::apercu_fin(e, &f))
        .ok_or_else(|| "Aucune campagne ouverte".to_string())
}

#[derive(Debug, Deserialize)]
#[serde(tag = "genre", rename_all = "snake_case")]
pub enum ValidationDto {
    Tout,
    Partiel { pjs: Vec<u32> },
    Abandon,
}

/// Fin du combat : on le valide (pour tous, ou certains personnages seulement) ou on l'abandonne.
#[tauri::command]
pub fn combat_terminer(
    s: S,
    validation: ValidationDto,
    contestations: Vec<(IdEffet, Duree)>,
) -> Retour {
    let mut x = session(&s);
    let Some(c) = x.combat.clone() else {
        return Retour::erreur("Aucun combat en cours");
    };
    let v = match validation {
        ValidationDto::Tout => Validation::Tout,
        ValidationDto::Partiel { pjs } => Validation::Partiel(pjs),
        ValidationDto::Abandon => Validation::Abandon,
    };
    if v == Validation::Abandon {
        x.combat = None;
        x.supprimer_fichier_combat();
        return Retour::ok("Combat abandonné : rien n'a changé dans la campagne");
    }
    let contextes = x.contextes();
    let ctx = |id: IdPj| contextes.get(&id).cloned();
    let res = match c.valider(&v, &contestations, &ctx) {
        Ok(r) => r,
        Err(e) => return Retour::erreur(e.to_string()),
    };
    let rounds = c.etat().rounds_joues;
    let sorts: u32 = c.etat().stats.values().map(|st| st.sorts_lances).sum();
    let resume = format!("{rounds} round(s), {sorts} sort(s) lancé(s)");
    let retour = x.executer(
        Evenement::CombatValide {
            aventure: Box::new(res.aventure),
            fiche: res.sortie.fiche,
            minuits: res.sortie.minuits,
            resume: resume.clone(),
        },
        &format!("Combat validé ({resume})"),
    );
    if retour.ok {
        x.combat = None;
        x.supprimer_fichier_combat();
    }
    retour
}

#[tauri::command]
pub fn catalogue_actions() -> Vec<combat::catalogue::ActionDef> {
    combat::catalogue::catalogue()
}

/// Les 34 états du Core (identifiant, nom français) pour les choisir dans l'interface.
#[tauri::command]
pub fn catalogue_etats() -> Vec<(String, String)> {
    fiches::Referentiel::partage()
        .etats
        .iter()
        .map(|e| (e.id.clone(), e.nom.clone()))
        .collect()
}
