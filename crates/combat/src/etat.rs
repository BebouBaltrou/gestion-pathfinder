//! État d'un combat et actions qui le font évoluer.
//!
//! Un combat travaille sur une **copie de travail** de l'état d'aventure. Chaque action est
//! enregistrée avec les dés saisis : rejouer la liste d'actions redonne exactement le même état
//! (déterminisme), et annuler une action revient à rejouer jusqu'à l'action précédente.

use crate::economie::{Depense, EconomieTour, TypeAction};
use aventure::Sante;
use aventure::{ContextePj, ErreurAventure, EtatAventure, EvenementAventure, IdPj, Sortie};
use fiches::journal::EvenementFiche;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use temps::{ErreurTemps, MomentDuTour};

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ActionCombat {
    /// Résultat du d20 physique ; le modificateur d'initiative vient de la fiche.
    Initiative {
        pj: IdPj,
        d20: i32,
    },
    /// Tirage de départage entre deux personnages à égalité parfaite.
    Departage {
        pj: IdPj,
        d20: i32,
    },
    /// Début du combat. `surprise` : seuls ces personnages agissent au round de surprise.
    Demarrer {
        surprise: Option<Vec<IdPj>>,
    },
    /// Fin du tour du personnage qui joue.
    TourSuivant,
    /// Le personnage qui joue retarde son action : il passe plus bas dans l'ordre.
    Retarder {
        pj: IdPj,
        nouvelle_initiative: i32,
    },
    /// Correction d'une initiative en cours de combat.
    ModifierInitiative {
        pj: IdPj,
        total: i32,
    },
    /// Le personnage agit : l'économie d'actions du tour est contrôlée (libellé libre, souvent
    /// pris dans le catalogue `data/combat/actions.json`). `deplacement` : l'action le déplace
    /// réellement (incompatible avec un pas de 1,5 m).
    Agir {
        pj: IdPj,
        type_action: TypeAction,
        libelle: String,
        deplacement: bool,
    },
    /// Action préparée (texte libre), gardée jusqu'au prochain tour du personnage.
    Preparer {
        pj: IdPj,
        action: String,
    },
    /// Santé, effets, boire, manger… (les événements qui touchent au temps sont refusés).
    Aventure(EvenementAventure),
    Note(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErreurCombat {
    Aventure(ErreurAventure),
    Temps(ErreurTemps),
    Invalide(String),
}

impl fmt::Display for ErreurCombat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErreurCombat::Aventure(e) => write!(f, "{e}"),
            ErreurCombat::Temps(e) => write!(f, "{e}"),
            ErreurCombat::Invalide(m) => write!(f, "{m}"),
        }
    }
}

impl std::error::Error for ErreurCombat {}

impl From<ErreurAventure> for ErreurCombat {
    fn from(e: ErreurAventure) -> Self {
        ErreurCombat::Aventure(e)
    }
}

impl From<ErreurTemps> for ErreurCombat {
    fn from(e: ErreurTemps) -> Self {
        ErreurCombat::Temps(e)
    }
}

fn invalide<T>(m: &str) -> Result<T, ErreurCombat> {
    Err(ErreurCombat::Invalide(m.to_string()))
}

/// Ce que le moteur demande au joueur de saisir.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Demande {
    /// Jet de stabilisation d'un mourant : d20 + Con. − |PV négatifs| contre DD 10.
    Stabilisation { pj: IdPj, malus: i32 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
pub enum Phase {
    /// On saisit les initiatives.
    Initiative,
    EnCours,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Init {
    pub d20: i32,
    pub modificateur: i32,
    pub total: i32,
    pub departage: Option<i32>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Stats {
    pub degats_subis: i32,
    pub soins_recus: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct EntreeCombat {
    pub round: u32,
    pub texte: String,
}

/// État de travail d'un combat (toujours recalculé en rejouant les actions).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EtatCombat {
    pub aventure: EtatAventure,
    pub engages: Vec<IdPj>,
    pub phase: Phase,
    pub initiatives: BTreeMap<IdPj, Init>,
    /// Ordre de jeu complet (initiative décroissante).
    pub ordre: Vec<IdPj>,
    /// Personnages qui agissent au round de surprise (round 0), s'il y en a un.
    pub surprise: Option<Vec<IdPj>>,
    /// Round courant ; 0 = round de surprise.
    pub round: u32,
    pub actif: Option<IdPj>,
    pub preparees: BTreeMap<IdPj, String>,
    /// Actions dépensées par chacun dans son tour en cours.
    pub economie: BTreeMap<IdPj, EconomieTour>,
    /// Personnages qui ont déjà utilisé une action immédiate hors de leur tour (elle prend leur
    /// prochaine action rapide).
    pub immediates_hors_tour: BTreeSet<IdPj>,
    pub stats: BTreeMap<IdPj, Stats>,
    pub journal: Vec<EntreeCombat>,
    /// Événements à appliquer aux fiches à la validation, dans l'ordre.
    pub fiche: Vec<(IdPj, EvenementFiche)>,
    /// Minuits franchis pendant le combat (jours d'intermède).
    pub minuits: i64,
    /// Rounds entièrement joués.
    pub rounds_joues: u32,
}

impl EtatCombat {
    /// Copie de travail de l'état d'aventure, horloge en mode combat.
    pub fn initial(base: &EtatAventure, engages: Vec<IdPj>) -> Result<Self, ErreurCombat> {
        if engages.is_empty() {
            return invalide("aucun personnage engagé");
        }
        for pj in &engages {
            if !base.pjs.contains_key(pj) {
                return Err(ErreurAventure::PjInconnu(*pj).into());
            }
        }
        let mut aventure = base.clone();
        aventure.entrer_en_combat()?;
        Ok(EtatCombat {
            aventure,
            engages,
            phase: Phase::Initiative,
            initiatives: BTreeMap::new(),
            ordre: Vec::new(),
            surprise: None,
            round: 1,
            actif: None,
            preparees: BTreeMap::new(),
            economie: BTreeMap::new(),
            immediates_hors_tour: BTreeSet::new(),
            stats: BTreeMap::new(),
            journal: Vec::new(),
            fiche: Vec::new(),
            minuits: 0,
            rounds_joues: 0,
        })
    }

    fn noter(&mut self, texte: String) {
        self.journal.push(EntreeCombat {
            round: self.round,
            texte,
        });
    }

    fn nom(ctx: &dyn Fn(IdPj) -> Option<ContextePj>, pj: IdPj) -> String {
        ctx(pj).map_or_else(|| format!("PJ {pj}"), |c| c.nom)
    }

    fn verifier_engage(&self, pj: IdPj) -> Result<(), ErreurCombat> {
        if self.engages.contains(&pj) {
            Ok(())
        } else {
            invalide("ce personnage n'est pas engagé dans le combat")
        }
    }

    fn cle(&self, pj: IdPj) -> (i32, i32, i32) {
        let i = &self.initiatives[&pj];
        (i.total, i.modificateur, i.departage.unwrap_or(i32::MIN))
    }

    fn trier(&mut self) {
        let mut ordre: Vec<IdPj> = self.initiatives.keys().copied().collect();
        ordre.sort_by(|a, b| self.cle(*b).cmp(&self.cle(*a)).then(a.cmp(b)));
        self.ordre = ordre;
    }

    /// Groupes de personnages à égalité parfaite (initiative, modificateur et départage).
    pub fn egalites(&self) -> Vec<Vec<IdPj>> {
        let mut groupes: BTreeMap<(i32, i32, i32), Vec<IdPj>> = BTreeMap::new();
        for pj in self.initiatives.keys() {
            groupes.entry(self.cle(*pj)).or_default().push(*pj);
        }
        groupes.into_values().filter(|g| g.len() > 1).collect()
    }

    /// Personnages qui jouent dans le round courant (tous, sauf au round de surprise).
    pub fn ordre_du_round(&self) -> Vec<IdPj> {
        match (&self.surprise, self.round) {
            (Some(s), 0) => self
                .ordre
                .iter()
                .copied()
                .filter(|p| s.contains(p))
                .collect(),
            _ => self.ordre.clone(),
        }
    }

    fn fusionner(&mut self, s: Sortie) {
        self.fiche.extend(s.fiche);
        self.minuits += s.minuits;
    }

    fn debut_de_tour(&mut self, pj: IdPj, ctx: &dyn Fn(IdPj) -> Option<ContextePj>) {
        let s = self
            .aventure
            .terminer_effets_du_tour(pj, MomentDuTour::Debut, ctx);
        self.fusionner(s);
        self.actif = Some(pj);
        // nouveau tour : économie d'actions remise à zéro ; une action immédiate prise hors
        // tour a déjà pris l'action rapide
        let rapide_prise = self.immediates_hors_tour.remove(&pj);
        self.economie.insert(
            pj,
            EconomieTour {
                rapide: rapide_prise,
                ..EconomieTour::default()
            },
        );
        if let Some(a) = self.preparees.remove(&pj) {
            let n = Self::nom(ctx, pj);
            self.noter(format!("{n} : l'action préparée (« {a} ») est expirée."));
        }
        let n = Self::nom(ctx, pj);
        self.noter(format!("Tour de {n}."));
        let s = self.aventure.soins_de_debut_de_tour(pj, ctx);
        self.fusionner(s);
    }

    /// Ce que le moteur demande au joueur à ce moment (dé physique à saisir).
    pub fn demandes(&self, ctx: &dyn Fn(IdPj) -> Option<ContextePj>) -> Vec<Demande> {
        let mut v = Vec::new();
        if let (Phase::EnCours, Some(pj)) = (self.phase, self.actif) {
            if let (Some(c), Some(s)) = (ctx(pj), self.aventure.pjs.get(&pj)) {
                if s.vital.sante(c.pv_max, c.con_score) == Sante::Mourant {
                    v.push(Demande::Stabilisation {
                        pj,
                        malus: s.vital.pv_actuels(c.pv_max),
                    });
                }
            }
        }
        v
    }

    /// Le personnage est-il limité à une seule action simple ou de mouvement (round de
    /// surprise, chancelant, hors de combat) ?
    fn limite(&self, pj: IdPj, ctx: &dyn Fn(IdPj) -> Option<ContextePj>) -> bool {
        self.round == 0
            || match (ctx(pj), self.aventure.pjs.get(&pj)) {
                (Some(c), Some(s)) => matches!(
                    s.vital.sante(c.pv_max, c.con_score),
                    Sante::Chancelant | Sante::HorsDeCombat
                ),
                _ => false,
            }
    }

    /// Applique une action. En cas d'erreur, l'état n'a pas changé.
    pub fn appliquer(
        &mut self,
        a: &ActionCombat,
        ctx: &dyn Fn(IdPj) -> Option<ContextePj>,
    ) -> Result<(), ErreurCombat> {
        let sauvegarde = self.clone();
        let r = self.appliquer_interne(a, ctx);
        if r.is_err() {
            *self = sauvegarde;
        }
        r
    }

    fn appliquer_interne(
        &mut self,
        a: &ActionCombat,
        ctx: &dyn Fn(IdPj) -> Option<ContextePj>,
    ) -> Result<(), ErreurCombat> {
        match a {
            ActionCombat::Initiative { pj, d20 } => {
                self.verifier_engage(*pj)?;
                if self.phase != Phase::Initiative {
                    return invalide("le combat est commencé : utiliser « modifier l'initiative »");
                }
                if !(1..=20).contains(d20) {
                    return invalide("d20 : de 1 à 20");
                }
                let c = ctx(*pj).ok_or(ErreurAventure::PjInconnu(*pj))?;
                self.initiatives.insert(
                    *pj,
                    Init {
                        d20: *d20,
                        modificateur: c.initiative,
                        total: d20 + c.initiative,
                        departage: None,
                    },
                );
                self.trier();
                self.noter(format!(
                    "{} : initiative {} ({d20} + {}).",
                    c.nom,
                    d20 + c.initiative,
                    c.initiative
                ));
            }
            ActionCombat::Departage { pj, d20 } => {
                self.verifier_engage(*pj)?;
                if !(1..=20).contains(d20) {
                    return invalide("d20 : de 1 à 20");
                }
                let Some(i) = self.initiatives.get_mut(pj) else {
                    return invalide("initiative pas encore saisie");
                };
                i.departage = Some(*d20);
                self.trier();
                let n = Self::nom(ctx, *pj);
                self.noter(format!("{n} : tirage de départage {d20}."));
            }
            ActionCombat::Demarrer { surprise } => {
                if self.phase != Phase::Initiative {
                    return invalide("le combat est déjà commencé");
                }
                if let Some(pj) = self
                    .engages
                    .iter()
                    .find(|p| !self.initiatives.contains_key(p))
                {
                    let n = Self::nom(ctx, *pj);
                    return Err(ErreurCombat::Invalide(format!(
                        "il manque l'initiative de {n}"
                    )));
                }
                if let Some(g) = self.egalites().first() {
                    let noms: Vec<String> = g.iter().map(|p| Self::nom(ctx, *p)).collect();
                    return Err(ErreurCombat::Invalide(format!(
                        "égalité parfaite entre {} : saisir un tirage de départage",
                        noms.join(" et ")
                    )));
                }
                match surprise {
                    Some(s) if s.is_empty() || s.iter().any(|p| !self.engages.contains(p)) => {
                        return invalide("round de surprise : liste de personnages invalide");
                    }
                    Some(_) => self.round = 0,
                    None => self.round = 1,
                }
                self.surprise = surprise.clone();
                self.phase = Phase::EnCours;
                let premier = self.ordre_du_round()[0];
                self.noter(if self.round == 0 {
                    "Début du combat : round de surprise.".into()
                } else {
                    "Début du combat.".into()
                });
                self.debut_de_tour(premier, ctx);
            }
            ActionCombat::TourSuivant => {
                let (Phase::EnCours, Some(actif)) = (self.phase, self.actif) else {
                    return invalide("le combat n'est pas commencé");
                };
                let s = self
                    .aventure
                    .terminer_effets_du_tour(actif, MomentDuTour::Fin, ctx);
                self.fusionner(s);
                let liste = self.ordre_du_round();
                let position = liste.iter().position(|p| *p == actif).unwrap_or(0);
                if let Some(suivant) = liste.get(position + 1) {
                    self.debut_de_tour(*suivant, ctx);
                } else {
                    // le round s'achève : 6 secondes passent
                    let s = self.aventure.round_de_combat(ctx)?;
                    for id in &s.expires {
                        if let Some(t) = self
                            .aventure
                            .effets
                            .termines
                            .iter()
                            .find(|t| t.effet.id == *id)
                        {
                            let m = format!("« {} » a pris fin.", t.effet.nom);
                            self.noter(m);
                        }
                    }
                    self.fusionner(s);
                    self.rounds_joues += 1;
                    self.round += 1;
                    self.noter(format!("Round {}.", self.round));
                    let premier = self.ordre[0];
                    self.debut_de_tour(premier, ctx);
                }
            }
            ActionCombat::Retarder {
                pj,
                nouvelle_initiative,
            } => {
                if self.actif != Some(*pj) || self.phase != Phase::EnCours {
                    return invalide("seul le personnage qui joue peut retarder son action");
                }
                if *nouvelle_initiative >= self.initiatives[pj].total {
                    return invalide("on retarde en passant plus bas dans l'ordre");
                }
                let liste = self.ordre_du_round();
                let position = liste.iter().position(|p| p == pj).unwrap_or(0);
                let Some(suivant) = liste.get(position + 1).copied() else {
                    return invalide("personne ne joue après vous dans ce round");
                };
                let i = self.initiatives.get_mut(pj).expect("initiative connue");
                i.total = *nouvelle_initiative;
                i.departage = None;
                self.trier();
                let n = Self::nom(ctx, *pj);
                self.noter(format!(
                    "{n} retarde son action (initiative {nouvelle_initiative})."
                ));
                self.debut_de_tour(suivant, ctx);
            }
            ActionCombat::ModifierInitiative { pj, total } => {
                self.verifier_engage(*pj)?;
                let Some(i) = self.initiatives.get_mut(pj) else {
                    return invalide("initiative pas encore saisie");
                };
                i.total = *total;
                i.departage = None;
                self.trier();
                let n = Self::nom(ctx, *pj);
                self.noter(format!("{n} : initiative corrigée à {total}."));
            }
            ActionCombat::Agir {
                pj,
                type_action,
                libelle,
                deplacement,
            } => {
                self.verifier_engage(*pj)?;
                if self.phase != Phase::EnCours {
                    return invalide("le combat n'est pas commencé");
                }
                let c = ctx(*pj).ok_or(ErreurAventure::PjInconnu(*pj))?;
                let sante = self
                    .aventure
                    .pjs
                    .get(pj)
                    .map(|s| s.vital.sante(c.pv_max, c.con_score))
                    .ok_or(ErreurAventure::PjInconnu(*pj))?;
                if matches!(
                    sante,
                    Sante::Mourant | Sante::Stable | Sante::Mort | Sante::Inconscient
                ) {
                    return Err(ErreurCombat::Invalide(format!(
                        "{} est {} : il ne peut pas agir",
                        c.nom,
                        sante.nom()
                    )));
                }
                let limite = self.limite(*pj, ctx);
                let tour_de_pj = self.actif == Some(*pj);
                if *type_action == TypeAction::Immediate && !tour_de_pj {
                    // action immédiate hors de son tour : elle prend la prochaine action rapide
                    if !self.immediates_hors_tour.insert(*pj) {
                        return invalide("une action immédiate a déjà été utilisée avant ce tour");
                    }
                    self.noter(format!(
                        "{} : {libelle} (action immédiate hors tour).",
                        c.nom
                    ));
                } else {
                    if !tour_de_pj {
                        return invalide("ce n'est pas le tour de ce personnage");
                    }
                    let eco = self.economie.entry(*pj).or_default();
                    let depense = eco
                        .depenser(*type_action, *deplacement, limite)
                        .map_err(ErreurCombat::Invalide)?;
                    let note = match depense {
                        Depense::Normale => String::new(),
                        Depense::ConvertiEnMouvement => {
                            " (l'action simple sert de mouvement)".into()
                        }
                    };
                    self.noter(format!(
                        "{} : {libelle} ({}){note}.",
                        c.nom,
                        type_action.nom()
                    ));
                    // à 0 PV, une action simple ou complexe inflige 1 dégât (p. 189, lecture à valider)
                    if sante == Sante::HorsDeCombat
                        && matches!(type_action, TypeAction::Simple | TypeAction::Complexe)
                    {
                        let s = self.aventure.appliquer(
                            &EvenementAventure::Degats {
                                pj: *pj,
                                nombre: 1,
                                letaux: true,
                            },
                            ctx,
                        )?;
                        self.stats.entry(*pj).or_default().degats_subis += 1;
                        self.fusionner(s);
                        self.noter(format!(
                            "{} subit 1 dégât : effort alors qu'il est hors de combat.",
                            c.nom
                        ));
                    }
                }
            }
            ActionCombat::Preparer { pj, action } => {
                self.verifier_engage(*pj)?;
                if self.phase != Phase::EnCours {
                    return invalide("le combat n'est pas commencé");
                }
                self.preparees.insert(*pj, action.clone());
                let n = Self::nom(ctx, *pj);
                self.noter(format!("{n} prépare une action : {action}."));
            }
            ActionCombat::Aventure(ev) => {
                if let Some(pj) = pj_de(ev) {
                    self.verifier_engage(pj)?;
                }
                let s = self.aventure.appliquer(ev, ctx)?;
                match ev {
                    EvenementAventure::Degats { pj, nombre, .. } => {
                        self.stats.entry(*pj).or_default().degats_subis += nombre;
                    }
                    EvenementAventure::Soin { pj, nombre } => {
                        self.stats.entry(*pj).or_default().soins_recus += nombre;
                    }
                    _ => {}
                }
                self.fusionner(s);
            }
            ActionCombat::Note(t) => self.noter(t.clone()),
        }
        Ok(())
    }
}

/// Personnage visé par un événement d'aventure, s'il en vise un.
fn pj_de(ev: &EvenementAventure) -> Option<IdPj> {
    match ev {
        EvenementAventure::Degats { pj, .. }
        | EvenementAventure::Soin { pj, .. }
        | EvenementAventure::PvTemporaires { pj, .. }
        | EvenementAventure::DegatsTypes { pj, .. }
        | EvenementAventure::Stabilisation { pj, .. }
        | EvenementAventure::EffetAjoute { pj, .. }
        | EvenementAventure::Manger { pj }
        | EvenementAventure::Boire { pj }
        | EvenementAventure::TestSurvie { pj, .. } => Some(*pj),
        _ => None,
    }
}
