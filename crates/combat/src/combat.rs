//! Combat transactionnel : annuler/rétablir action par action, aperçu et validation de la fin,
//! sauvegarde atomique après chaque action.

use crate::etat::{ActionCombat, ErreurCombat, EtatCombat, Stats};
use aventure::{ContextePj, EtatAventure, IdPj, Sortie};
use temps::{BilanFinDeCombat, Duree, Echeance, IdEffet};

pub const FORMAT_COMBAT: u32 = 1;

pub type Ctx<'a> = &'a dyn Fn(IdPj) -> Option<ContextePj>;

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Combat {
    pub format: u32,
    /// État d'aventure au moment où le combat a commencé (jamais modifié).
    pub base: EtatAventure,
    pub engages: Vec<IdPj>,
    pub actions: Vec<ActionCombat>,
    /// Nombre d'actions appliquées ; les suivantes (s'il y en a) peuvent être rétablies.
    pub curseur: usize,
    #[cfg_attr(feature = "serde", serde(skip))]
    courant: Option<EtatCombat>,
}

/// Ce que le joueur valide à la fin.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Validation {
    /// Tous les personnages reportent leur état de travail sur leur fiche et l'aventure.
    Tout,
    /// Seuls ces personnages reportent leur état ; les autres reviennent à leur état d'avant.
    Partiel(Vec<IdPj>),
    /// On jette le combat : rien n'est changé, l'horloge reste au début du combat.
    Abandon,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecapPj {
    pub pj: IdPj,
    pub pv_avant: i32,
    pub pv_apres: i32,
    pub stats: Stats,
}

/// Récapitulatif montré avant validation (§2.2 / §3.11).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApercuFin {
    pub rounds_joues: u32,
    pub par_pj: Vec<RecapPj>,
    /// Conversion de durées : effets dissipés (à afficher en alerte) et reliquats arrondis.
    pub bilan: BilanFinDeCombat,
    /// Effets qui continuent après le combat : (id, nom, personnage, échéance).
    pub persistants: Vec<(IdEffet, String, String, Echeance)>,
}

/// Résultat d'une validation : le nouvel état d'aventure et ce qui reste à répercuter ailleurs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Resultat {
    pub aventure: EtatAventure,
    pub sortie: Sortie,
}

impl Combat {
    pub fn nouveau(
        base: &EtatAventure,
        engages: Vec<IdPj>,
        _ctx: Ctx,
    ) -> Result<Combat, ErreurCombat> {
        let courant = EtatCombat::initial(base, engages.clone())?;
        Ok(Combat {
            format: FORMAT_COMBAT,
            base: base.clone(),
            engages,
            actions: Vec::new(),
            curseur: 0,
            courant: Some(courant),
        })
    }

    /// Recalcule l'état courant en rejouant les actions (après chargement, annulation…).
    pub fn reconstruire(&mut self, ctx: Ctx) -> Result<(), ErreurCombat> {
        let mut e = EtatCombat::initial(&self.base, self.engages.clone())?;
        for a in &self.actions[..self.curseur] {
            e.appliquer(a, ctx)?;
        }
        self.courant = Some(e);
        Ok(())
    }

    pub fn etat(&self) -> &EtatCombat {
        self.courant
            .as_ref()
            .expect("combat chargé : appeler `reconstruire` avant")
    }

    pub fn appliquer(&mut self, a: ActionCombat, ctx: Ctx) -> Result<(), ErreurCombat> {
        if self.courant.is_none() {
            self.reconstruire(ctx)?;
        }
        self.courant
            .as_mut()
            .expect("construit ci-dessus")
            .appliquer(&a, ctx)?;
        self.actions.truncate(self.curseur);
        self.actions.push(a);
        self.curseur += 1;
        Ok(())
    }

    pub fn peut_annuler(&self) -> bool {
        self.curseur > 0
    }

    pub fn peut_retablir(&self) -> bool {
        self.curseur < self.actions.len()
    }

    /// Annule la dernière action, y compris les expirations d'un « tour suivant ».
    pub fn annuler(&mut self, ctx: Ctx) -> Result<bool, ErreurCombat> {
        if !self.peut_annuler() {
            return Ok(false);
        }
        self.curseur -= 1;
        self.reconstruire(ctx)?;
        Ok(true)
    }

    pub fn retablir(&mut self, ctx: Ctx) -> Result<bool, ErreurCombat> {
        if !self.peut_retablir() {
            return Ok(false);
        }
        self.curseur += 1;
        self.reconstruire(ctx)?;
        Ok(true)
    }

    /// Récapitulatif de fin : qui a perdu ou gagné des PV, quels effets seront dissipés.
    pub fn apercu_fin(&self, ctx: Ctx) -> Result<ApercuFin, ErreurCombat> {
        let e = self.etat();
        let mut travail = e.aventure.clone();
        let (bilan, _) = travail.terminer_combat(&[])?;
        let par_pj = self
            .engages
            .iter()
            .map(|pj| {
                let pv_max = ctx(*pj).map_or(0, |c| c.pv_max);
                let pv = |a: &EtatAventure| a.pjs.get(pj).map_or(0, |s| s.vital.pv_actuels(pv_max));
                RecapPj {
                    pj: *pj,
                    pv_avant: pv(&self.base),
                    pv_apres: pv(&e.aventure),
                    stats: e.stats.get(pj).cloned().unwrap_or_default(),
                }
            })
            .collect();
        let persistants = travail
            .effets
            .actifs
            .iter()
            .map(|t| (t.id, t.nom.clone(), t.proprietaire.clone(), t.echeance))
            .collect();
        Ok(ApercuFin {
            rounds_joues: e.rounds_joues,
            par_pj,
            bilan,
            persistants,
        })
    }

    /// Valide ou abandonne le combat. `contestations` maintient des effets dissipés avec une durée
    /// saisie. Ne modifie pas le combat : l'appelant remplace son état d'aventure par le résultat.
    pub fn valider(
        &self,
        v: &Validation,
        contestations: &[(IdEffet, Duree)],
    ) -> Result<Resultat, ErreurCombat> {
        let e = self.etat();
        if let Validation::Abandon = v {
            return Ok(Resultat {
                aventure: self.base.clone(),
                sortie: Sortie::default(),
            });
        }
        let mut travail = e.aventure.clone();
        let (_, fin) = travail.terminer_combat(contestations)?;
        let mut fiche = e.fiche.clone();
        fiche.extend(fin.fiche);
        let sortie_commune = |fiche| Sortie {
            fiche,
            minuits: e.minuits,
            expires: Vec::new(),
        };
        match v {
            Validation::Tout => Ok(Resultat {
                aventure: travail,
                sortie: sortie_commune(fiche),
            }),
            Validation::Partiel(choisis) => {
                if let Some(pj) = choisis.iter().find(|p| !self.engages.contains(p)) {
                    return Err(ErreurCombat::Invalide(format!(
                        "le personnage {pj} n'était pas engagé"
                    )));
                }
                let mut a = self.base.clone();
                a.horloge = travail.horloge;
                for pj in choisis {
                    if let Some(s) = travail.pjs.get(pj) {
                        a.pjs.insert(*pj, s.clone());
                    }
                }
                let noms: Vec<String> = choisis.iter().map(|p| p.to_string()).collect();
                a.effets.remplacer_pour(&travail.effets, &noms);
                a.journal.push(aventure::Entree {
                    instant: a.horloge.maintenant,
                    texte: format!(
                        "Fin du combat ({} round(s)) : validation partielle pour {} personnage(s).",
                        e.rounds_joues,
                        choisis.len()
                    ),
                });
                let fiche = fiche
                    .into_iter()
                    .filter(|(pj, _)| choisis.contains(pj))
                    .collect();
                Ok(Resultat {
                    aventure: a,
                    sortie: sortie_commune(fiche),
                })
            }
            Validation::Abandon => unreachable!("traité plus haut"),
        }
    }
}

#[cfg(feature = "serde")]
pub mod persistance {
    use super::*;
    use std::io::Write;
    use std::path::Path;

    /// Écrit le combat de façon atomique : fichier temporaire, puis renommage.
    pub fn sauvegarder(c: &Combat, chemin: &Path) -> Result<(), String> {
        let json = serde_json::to_string(c).map_err(|e| e.to_string())?;
        let tmp = chemin.with_extension("tmp");
        {
            let mut f = std::fs::File::create(&tmp).map_err(|e| e.to_string())?;
            f.write_all(json.as_bytes()).map_err(|e| e.to_string())?;
            f.sync_all().map_err(|e| e.to_string())?;
        }
        std::fs::rename(&tmp, chemin).map_err(|e| e.to_string())
    }

    /// Relit un combat interrompu et recalcule son état.
    pub fn charger(chemin: &Path, ctx: Ctx) -> Result<Combat, String> {
        let json = std::fs::read_to_string(chemin).map_err(|e| e.to_string())?;
        let mut c: Combat = serde_json::from_str(&json).map_err(|e| e.to_string())?;
        if c.format > FORMAT_COMBAT {
            return Err(format!(
                "combat enregistré par une version plus récente (format {})",
                c.format
            ));
        }
        c.reconstruire(ctx).map_err(|e| e.to_string())?;
        Ok(c)
    }
}
