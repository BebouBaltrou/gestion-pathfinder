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
        v.sort_by(|a, b| b.0.cmp(&a.0));
        v.into_iter().map(|(_, f)| f).collect()
    }

    pub fn ouvrir(&mut self, chemin: &Path) -> Retour {
        match campagne::fichier::charger(chemin) {
            Ok(c) => {
                let m = format!("Campagne « {} » ouverte", c.nom);
                self.simulation = None;
                self.campagne = Some(c);
                self.chemin = Some(chemin.to_path_buf());
                Retour::ok(m)
            }
            Err(e) => Retour::erreur(format!("Impossible d'ouvrir {} : {e}", chemin.display())),
        }
    }

    pub fn creer(&mut self, nom: &str) -> Retour {
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
        self.simulation = None;
        self.campagne = Some(Campagne::nouvelle(nom));
        self.chemin = Some(chemin);
        if let Err(e) = self.enregistrer() {
            return Retour::erreur(e);
        }
        Retour::ok(format!("Campagne « {nom} » créée"))
    }

    fn enregistrer(&mut self) -> Result<(), String> {
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
        let Some(c) = self.campagne.as_mut() else {
            return Retour::erreur("Aucune campagne ouverte");
        };
        match c.executer(ev, &self.r) {
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
    pub fn regler_options(
        &mut self,
        o: regles::options::OptionsRegles,
        succes: &str,
    ) -> Retour {
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
