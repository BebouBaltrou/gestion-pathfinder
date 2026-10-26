//! Session de travail : campagne ouverte, simulation, enregistrement automatique.
//! Indépendante de l'interface : les commandes de `main.rs` ne font que l'appeler.

use campagne::evenements::Evenement;
use campagne::Campagne;
use regles::Referentiel;
use serde::Serialize;
use std::path::{Path, PathBuf};

/// Résultat d'une action, affiché dans une notification.
#[derive(Debug, Clone, Serialize)]
pub struct Retour {
    pub ok: bool,
    /// Conséquence à surveiller (trésorerie négative, test de prestige…).
    pub alerte: bool,
    pub message: String,
}

impl Retour {
    pub fn ok(message: impl Into<String>) -> Self {
        Retour {
            ok: true,
            alerte: false,
            message: message.into(),
        }
    }
    pub fn erreur(message: impl Into<String>) -> Self {
        Retour {
            ok: false,
            alerte: false,
            message: message.into(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct FichierCampagne {
    pub nom: String,
    pub chemin: String,
    pub ouverte: bool,
}

pub struct Session {
    pub r: Referentiel,
    pub dossier: PathBuf,
    pub campagne: Option<Campagne>,
    /// Mode simulation : la campagne réelle mise de côté ; `campagne` est alors une copie
    /// jetable, jamais enregistrée.
    pub simulation: Option<Campagne>,
    pub chemin: Option<PathBuf>,
    /// Modèles personnels (bâtiments, organisations), communs à toutes les campagnes.
    pub modeles: campagne::modeles_perso::ModelesPerso,
    /// Combat en cours : copie de travail séparée de la campagne, sauvegardée après chaque action
    /// dans un fichier à côté de la campagne (reprise après un plantage).
    pub combat: Option<combat::Combat>,
}

/// Dossier des campagnes : %APPDATA%\GestionPathfinder\campagnes sous Windows,
/// ~/.local/share/GestionPathfinder/campagnes sous Linux (même dossier que l'ancienne version).
pub fn dossier_campagnes() -> PathBuf {
    // dossier imposé (essais automatiques, version portable)
    if let Some(d) = std::env::var_os("GESTION_PATHFINDER_DOSSIER") {
        return PathBuf::from(d);
    }
    let base = std::env::var_os("APPDATA")
        .or_else(|| std::env::var_os("XDG_DATA_HOME"))
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".local").join("share")))
        .unwrap_or_else(|| PathBuf::from("."));
    base.join("GestionPathfinder").join("campagnes")
}

impl Session {
    pub fn demarrer() -> Self {
        let mut s = Session {
            r: regles::referentiel_officiel(),
            dossier: dossier_campagnes(),
            campagne: None,
            simulation: None,
            chemin: None,
            modeles: campagne::modeles_perso::ModelesPerso::charger(&dossier_campagnes()),
            combat: None,
        };
        // rouvre la campagne la plus récemment modifiée
        if let Some(c) = s.lister().into_iter().next() {
            let _ = s.ouvrir(Path::new(&c.chemin));
        }
        s
    }

    /// Campagnes du dossier, la plus récente d'abord.
    pub fn lister(&self) -> Vec<FichierCampagne> {
        let mut v: Vec<(std::time::SystemTime, FichierCampagne)> = std::fs::read_dir(&self.dossier)
            .into_iter()
            .flatten()
            .flatten()
            .map(|e| e.path())
            .filter(|p| p.extension().is_some_and(|x| x == "json"))
            .map(|p| {
                let t = std::fs::metadata(&p)
                    .and_then(|m| m.modified())
                    .unwrap_or(std::time::UNIX_EPOCH);
                let nom = p
                    .file_stem()
                    .map(|s| s.to_string_lossy().to_string())
                    .unwrap_or_default();
                let ouverte = self.chemin.as_deref() == Some(p.as_path());
                (
                    t,
                    FichierCampagne {
                        nom,
                        chemin: p.to_string_lossy().to_string(),
                        ouverte,
                    },
                )
            })
            .collect();
        v.sort_by_key(|x| std::cmp::Reverse(x.0));
        v.into_iter().map(|(_, f)| f).collect()
    }

    pub fn ouvrir(&mut self, chemin: &Path) -> Retour {
        match campagne::fichier::charger(chemin) {
            Ok(c) => {
                let m = format!("Campagne « {} » ouverte", c.nom);
                self.simulation = None;
                self.campagne = Some(c);
                self.chemin = Some(chemin.to_path_buf());
                self.combat = None;
                let m = match self.recharger_combat() {
                    Ok(true) => format!("{m} : un combat interrompu a été retrouvé"),
                    _ => m,
                };
                Retour::ok(m)
            }
            Err(e) => Retour::erreur(format!("Impossible d'ouvrir {} : {e}", chemin.display())),
        }
    }

    pub fn creer(
        &mut self,
        nom: &str,
        calendrier: Option<&vues::parametres::SaisieCalendrier>,
    ) -> Retour {
        let nom = nom.trim();
        if nom.is_empty() {
            return Retour::erreur("Donnez un nom à la campagne");
        }
        let fichier: String = nom
            .chars()
            .map(|c| {
                if c.is_alphanumeric() || c == '-' || c == '_' || c == ' ' {
                    c
                } else {
                    '_'
                }
            })
            .collect();
        let chemin = self.dossier.join(format!("{}.json", fichier.trim()));
        if chemin.exists() {
            return Retour::erreur(format!("Une campagne « {nom} » existe déjà"));
        }
        let mut neuve = Campagne::nouvelle(nom);
        if let Some(cal) = calendrier {
            if let Err(e) = vues::parametres::appliquer_calendrier(&mut neuve, cal) {
                return Retour::erreur(e);
            }
        }
        self.simulation = None;
        self.combat = None;
        self.campagne = Some(neuve);
        self.chemin = Some(chemin);
        if let Err(e) = self.enregistrer() {
            return Retour::erreur(e);
        }
        Retour::ok(format!("Campagne « {nom} » créée"))
    }

    /// Supprime une campagne : son fichier (et celui d'un éventuel combat interrompu) est rangé
    /// dans le sous-dossier `corbeille`, d'où on peut le récupérer à la main.
    pub fn supprimer_campagne(&mut self, chemin: &Path) -> Retour {
        let est_dans_le_dossier = chemin.parent() == Some(self.dossier.as_path())
            && chemin.extension().is_some_and(|x| x == "json")
            && chemin.is_file();
        if !est_dans_le_dossier {
            return Retour::erreur("Cette campagne n'existe plus");
        }
        let ouverte = self.chemin.as_deref() == Some(chemin);
        if ouverte && self.simulation.is_some() {
            return Retour::erreur(
                "Une simulation est en cours : gardez-la ou abandonnez-la avant de supprimer la campagne",
            );
        }
        let corbeille = self.dossier.join("corbeille");
        if let Err(e) = std::fs::create_dir_all(&corbeille) {
            return Retour::erreur(format!("Dossier « corbeille » impossible à créer : {e}"));
        }
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |d| d.as_secs());
        let nom = chemin
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default();
        let ranger = |source: &Path, extension: &str| {
            std::fs::rename(source, corbeille.join(format!("{nom}.{stamp}.{extension}")))
        };
        if let Err(e) = ranger(chemin, "json") {
            return Retour::erreur(format!("Suppression impossible : {e}"));
        }
        let combat = chemin.with_extension("combat.json");
        if combat.is_file() {
            let _ = ranger(&combat, "combat.json");
        }
        if ouverte {
            self.campagne = None;
            self.chemin = None;
            self.combat = None;
        }
        Retour::ok(format!(
            "Campagne « {nom} » supprimée (rangée dans le dossier « corbeille »)"
        ))
    }

    /// Enregistre les modèles personnels (fichier commun aux deux applications).
    pub fn enregistrer_modeles(&self) -> Result<(), String> {
        self.modeles
            .enregistrer(&self.dossier)
            .map_err(|e| format!("Enregistrement des modèles impossible : {e}"))
    }

    pub fn enregistrer(&mut self) -> Result<(), String> {
        if self.simulation.is_some() {
            return Ok(()); // une simulation n'est jamais enregistrée
        }
        if let (Some(c), Some(p)) = (&self.campagne, &self.chemin) {
            campagne::fichier::enregistrer(c, p)
                .map_err(|e| format!("Enregistrement impossible : {e}"))?;
        }
        Ok(())
    }

    /// Exécute un événement sur la campagne ouverte, puis enregistre.
    pub fn executer(&mut self, ev: Evenement, succes: &str) -> Retour {
        if self.combat.is_some() && !ev.valide_un_combat() && !ev.est_une_note() {
            return Retour::erreur(
                "Un combat est en cours : terminez-le (validation ou abandon) avant toute autre action",
            );
        }
        self.executer_avec(succes, |c, r| c.executer(ev, r))
    }

    /// Exécute des commandes d'inventaire d'un seul geste (avec la mise à jour des fiches), puis
    /// enregistre. Refusé pendant un combat.
    pub fn executer_inventaires(
        &mut self,
        commandes: Vec<inventaire::Commande>,
        succes: &str,
    ) -> Retour {
        if self.combat.is_some() {
            return Retour::erreur(
                "Un combat est en cours : terminez-le (validation ou abandon) avant toute autre action",
            );
        }
        self.executer_avec(succes, |c, r| c.executer_inventaires(commandes, r))
    }

    fn executer_avec(
        &mut self,
        succes: &str,
        action: impl FnOnce(
            &mut campagne::Campagne,
            &regles::Referentiel,
        ) -> Result<Vec<campagne::etat::Consequence>, campagne::etat::Erreur>,
    ) -> Retour {
        let Some(c) = self.campagne.as_mut() else {
            return Retour::erreur("Aucune campagne ouverte");
        };
        match action(c, &self.r) {
            Ok(consequences) => {
                let etat = c.etat(&self.r).clone();
                let (message, alerte) = vues::resume_action(succes, &consequences, &etat);
                if let Err(e) = self.enregistrer() {
                    return Retour::erreur(e);
                }
                Retour {
                    ok: true,
                    alerte,
                    message,
                }
            }
            Err(e) => Retour::erreur(format!("Action refusée : {e}")),
        }
    }

    /// Lit l'état de la campagne ouverte.
    pub fn avec_etat<T>(&mut self, f: impl FnOnce(&campagne::etat::Etat) -> T) -> Option<T> {
        let c = self.campagne.as_mut()?;
        Some(f(c.etat(&self.r)))
    }

    /// Règles optionnelles de l'intermède (hors journal), puis enregistrement.
    pub fn regler_options(&mut self, o: regles::options::OptionsRegles, succes: &str) -> Retour {
        let Some(c) = self.campagne.as_mut() else {
            return Retour::erreur("Aucune campagne ouverte");
        };
        c.options = o;
        match self.enregistrer() {
            Ok(()) => Retour::ok(succes),
            Err(e) => Retour::erreur(e),
        }
    }

    pub fn annuler(&mut self) -> Retour {
        if self.combat.is_some() {
            return Retour::erreur(
                "Un combat est en cours : annulez dans le combat (ou abandonnez-le)",
            );
        }
        let Some(c) = self.campagne.as_mut() else {
            return Retour::erreur("Aucune campagne ouverte");
        };
        let Some(e) = c.annuler().map(|e| e.evenement.clone()) else {
            return Retour::erreur("Rien à annuler");
        };
        let d = c.decrire(&e, &self.r);
        match self.enregistrer() {
            Ok(()) => Retour::ok(format!("Annulé : {d}")),
            Err(e) => Retour::erreur(e),
        }
    }

    pub fn retablir(&mut self) -> Retour {
        if self.combat.is_some() {
            return Retour::erreur("Un combat est en cours : rétablissez dans le combat");
        }
        let Some(c) = self.campagne.as_mut() else {
            return Retour::erreur("Aucune campagne ouverte");
        };
        let Some(e) = c.retablir().map(|e| e.evenement.clone()) else {
            return Retour::erreur("Rien à rétablir");
        };
        let d = c.decrire(&e, &self.r);
        match self.enregistrer() {
            Ok(()) => Retour::ok(format!("Rétabli : {d}")),
            Err(e) => Retour::erreur(e),
        }
    }

    pub fn demarrer_simulation(&mut self) -> Retour {
        match &self.campagne {
            Some(c) if self.simulation.is_none() => {
                self.simulation = Some(c.clone());
                Retour::ok(
                    "Simulation démarrée : rien ne sera enregistré tant que vous ne la gardez pas",
                )
            }
            Some(_) => Retour::erreur("Une simulation est déjà en cours"),
            None => Retour::erreur("Aucune campagne ouverte"),
        }
    }

    pub fn abandonner_simulation(&mut self) -> Retour {
        match self.simulation.take() {
            Some(reelle) => {
                self.campagne = Some(reelle);
                Retour::ok("Simulation abandonnée : la campagne est revenue à son état d'avant")
            }
            None => Retour::erreur("Aucune simulation en cours"),
        }
    }

    pub fn garder_simulation(&mut self) -> Retour {
        if self.simulation.take().is_none() {
            return Retour::erreur("Aucune simulation en cours");
        }
        match self.enregistrer() {
            Ok(()) => {
                Retour::ok("Simulation gardée : ses actions font désormais partie de la campagne")
            }
            Err(e) => Retour::erreur(e),
        }
    }

    /// Écart entre la simulation et la campagne réelle : (actions, jours).
    pub fn ecart_simulation(&mut self) -> Option<(i64, i64)> {
        let reelle = self.simulation.as_ref()?.clone();
        let mut reelle = reelle;
        let (n_reel, j_reel) = (reelle.curseur, reelle.etat(&self.r).jour);
        let c = self.campagne.as_mut()?;
        let (n_sim, j_sim) = (c.curseur, c.etat(&self.r).jour);
        Some((n_sim as i64 - n_reel as i64, j_sim as i64 - j_reel as i64))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn session_dans(dossier: &Path) -> Session {
        Session {
            r: regles::referentiel_officiel(),
            dossier: dossier.to_path_buf(),
            campagne: None,
            simulation: None,
            chemin: None,
            modeles: Default::default(),
            combat: None,
        }
    }

    #[test]
    fn supprimer_une_campagne_la_range_dans_la_corbeille() {
        let d = std::env::temp_dir().join(format!("gp-suppression-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        let mut s = session_dans(&d);
        assert!(s.creer("Ancienne", None).ok);
        assert!(s.creer("Autre", None).ok);
        let chemin = d.join("Ancienne.json");
        assert!(chemin.is_file());
        // « Autre » est ouverte ; on supprime « Ancienne » sans toucher à l'ouverte
        assert!(s.supprimer_campagne(&chemin).ok);
        assert!(!chemin.exists());
        assert_eq!(s.lister().len(), 1);
        assert!(s.campagne.is_some());
        let rangees = std::fs::read_dir(d.join("corbeille")).unwrap().count();
        assert_eq!(rangees, 1);
        // supprimer la campagne ouverte la ferme
        let ouverte = d.join("Autre.json");
        assert!(s.supprimer_campagne(&ouverte).ok);
        assert!(s.campagne.is_none() && s.chemin.is_none());
        assert!(s.lister().is_empty());
        // fichier inexistant ou hors du dossier : refusé
        assert!(!s.supprimer_campagne(&ouverte).ok);
        assert!(!s.supprimer_campagne(Path::new("C:/Windows/notepad.exe")).ok);
        let _ = std::fs::remove_dir_all(&d);
    }
}
