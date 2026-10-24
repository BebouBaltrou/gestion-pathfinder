//! État du mode aventure et événements qui le font évoluer.
//!
//! L'état se **recalcule en rejouant les événements** (comme le reste de la campagne) : annuler,
//! rétablir et la simulation fonctionnent sans code supplémentaire. Chaque événement renvoie une
//! `Sortie` : les événements à répercuter sur les fiches (repos, effets en cours, dommages de
//! caractéristique…) et le nombre de minuits franchis (jours d'intermède à avancer).
//! Le moteur ne recalcule aucun bonus : les effets gardent leurs valeurs typées et c'est la fiche
//! qui les empile.

use crate::survie::{Besoin, Survie, TestDu};
use crate::vital::Vital;
use fiches::etats::EffetTemporaire;
use fiches::journal::EvenementFiche;
use fiches::{Carac, Effet};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use temps::{
    ApercuRecul, Duree, Ensemble, ErreurTemps, Horloge, IdEffet, Instant, Pas, ROUNDS_PAR_HEURE,
};

/// Identifiant d'un personnage (celui de la campagne).
pub type IdPj = u32;

/// Valeurs de la fiche dont le mode aventure a besoin, fournies par la campagne au moment de
/// l'événement (la fiche reste la seule à calculer).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextePj {
    pub nom: String,
    pub niveau: u32,
    pub pv_max: i32,
    pub con_score: i32,
    pub con_mod: i32,
    /// Dommages de caractéristique en cours (pour la guérison au repos).
    pub dommages: Vec<(Carac, i32)>,
}

pub type Contexte<'a> = &'a dyn Fn(IdPj) -> Option<ContextePj>;

/// Ce que porte un effet temporisé : des effets typés (le temps n'en connaît pas le détail).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Contenu {
    pub effets: Vec<Effet>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum EvenementAventure {
    /// Le personnage rejoint l'aventure : ses repères de faim et de soif partent de maintenant.
    Entree {
        pj: IdPj,
    },
    /// Le temps passe d'un pas (aventure seulement).
    Pas {
        pas: Pas,
    },
    /// Le temps passe de `rounds` rounds (aventure seulement).
    AvancerTemps {
        rounds: i64,
    },
    /// Réglage manuel de l'heure. En cas de recul : `reactiver` (effets expirés à remettre) et
    /// `retirer_crees` (confirmation de retirer les effets créés dans l'intervalle).
    RegleHeure {
        instant: Instant,
        reactiver: Vec<IdEffet>,
        retirer_crees: bool,
    },
    Degats {
        pj: IdPj,
        nombre: i32,
        letaux: bool,
    },
    Soin {
        pj: IdPj,
        nombre: i32,
    },
    PvTemporaires {
        pj: IdPj,
        nombre: i32,
    },
    EffetAjoute {
        pj: IdPj,
        nom: String,
        source: String,
        duree: Duree,
        effets: Vec<Effet>,
    },
    EffetRetire {
        id: IdEffet,
    },
    /// Repousse une expiration ou conteste une dissipation, avec une durée saisie.
    EffetRetabli {
        id: IdEffet,
        duree: Duree,
    },
    AlerteConfirmee {
        id: IdEffet,
    },
    Manger {
        pj: IdPj,
    },
    Boire {
        pj: IdPj,
    },
    /// Résultat d'un test de Constitution de faim ou de soif (d20 physique, sans le modificateur,
    /// que la fiche fournit) et, en cas d'échec, le 1d6 de dégâts non létaux.
    TestSurvie {
        pj: IdPj,
        besoin: Besoin,
        d20: i32,
        degats: Option<i32>,
    },
    /// Repos de `heures` heures pour un groupe. Une nuit complète (≥ 8 h, sans interruption)
    /// rend les emplacements de sorts et guérit ; l'alitement complet demande 24 h par jour.
    Repos {
        pjs: Vec<IdPj>,
        heures: u32,
        alitement_complet: bool,
        interrompu: bool,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErreurAventure {
    Temps(ErreurTemps),
    PjInconnu(IdPj),
    Invalide(String),
}

impl fmt::Display for ErreurAventure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErreurAventure::Temps(e) => write!(f, "{e}"),
            ErreurAventure::PjInconnu(id) => {
                write!(f, "le personnage {id} ne participe pas à l'aventure")
            }
            ErreurAventure::Invalide(m) => write!(f, "{m}"),
        }
    }
}

impl std::error::Error for ErreurAventure {}

impl From<ErreurTemps> for ErreurAventure {
    fn from(e: ErreurTemps) -> Self {
        ErreurAventure::Temps(e)
    }
}

/// Ce que la campagne doit répercuter après un événement.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Sortie {
    /// Événements à appliquer au journal de la fiche du personnage indiqué, dans l'ordre.
    pub fiche: Vec<(IdPj, EvenementFiche)>,
    /// Minuits franchis : jours d'intermède à avancer.
    pub minuits: i64,
    /// Effets arrivés à échéance pendant cet événement (alertes d'expiration).
    pub expires: Vec<IdEffet>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Entree {
    pub instant: Instant,
    pub texte: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SuiviPj {
    pub vital: Vital,
    pub survie: Survie,
    /// L'état « fatigué » a été donné par la faim ou la soif (et sera retiré avec elles).
    pub fatigue_de_survie: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct EtatAventure {
    pub horloge: Horloge,
    /// Règle de campagne : suivre la faim et la soif (désactivable).
    pub survie_active: bool,
    pub pjs: BTreeMap<IdPj, SuiviPj>,
    pub effets: Ensemble<Contenu>,
    /// Journal d'aventure, distinct du journal d'intermède et de celui des combats.
    pub journal: Vec<Entree>,
}

/// Nom de l'effet dans la fiche : unique, pour qu'un même sort lancé deux fois reste deux effets.
pub fn nom_dans_la_fiche(nom: &str, id: IdEffet) -> String {
    format!("{nom} #{id}")
}

impl EtatAventure {
    pub fn nouveau(debut: Instant, survie_active: bool) -> Self {
        EtatAventure {
            horloge: Horloge::nouvelle(debut),
            survie_active,
            pjs: BTreeMap::new(),
            effets: Ensemble::nouveau(),
            journal: Vec::new(),
        }
    }

    fn noter(&mut self, texte: String) {
        self.journal.push(Entree {
            instant: self.horloge.maintenant,
            texte,
        });
    }

    fn nom(ctx: Contexte, pj: IdPj) -> String {
        ctx(pj).map_or_else(|| format!("PJ {pj}"), |c| c.nom)
    }

    fn contexte(ctx: Contexte, pj: IdPj) -> Result<ContextePj, ErreurAventure> {
        ctx(pj).ok_or(ErreurAventure::PjInconnu(pj))
    }

    fn suivi(&mut self, pj: IdPj) -> Result<&mut SuiviPj, ErreurAventure> {
        self.pjs.get_mut(&pj).ok_or(ErreurAventure::PjInconnu(pj))
    }

    /// Tests de faim et de soif dus maintenant (tous personnages), à demander au joueur.
    pub fn tests_survie_dus(&self, ctx: Contexte) -> Vec<(IdPj, TestDu)> {
        if !self.survie_active {
            return Vec::new();
        }
        let mut v = Vec::new();
        for (id, s) in &self.pjs {
            let Some(c) = ctx(*id) else { continue };
            for besoin in [Besoin::Eau, Besoin::Nourriture] {
                for t in s
                    .survie
                    .tests_dus(besoin, self.horloge.maintenant, c.con_score)
                {
                    v.push((*id, t));
                }
            }
        }
        v
    }

    pub fn apercu_recul(&self, nouvel: Instant) -> ApercuRecul {
        self.effets.apercu_recul(nouvel)
    }

    /// Applique un événement ; en cas d'erreur, l'état n'a pas changé.
    pub fn appliquer(
        &mut self,
        ev: &EvenementAventure,
        ctx: Contexte,
    ) -> Result<Sortie, ErreurAventure> {
        let sauvegarde = self.clone();
        let r = self.appliquer_interne(ev, ctx);
        if r.is_err() {
            *self = sauvegarde;
        }
        r
    }

    fn effet_vers_fiche(&self, id: IdEffet, s: &mut Sortie, ajouter: bool) {
        let trouve = self.effets.actif(id).cloned().or_else(|| {
            self.effets
                .termines
                .iter()
                .find(|t| t.effet.id == id)
                .map(|t| t.effet.clone())
        });
        let Some(e) = trouve else { return };
        let Ok(pj) = e.proprietaire.parse::<IdPj>() else {
            return;
        };
        let nom = nom_dans_la_fiche(&e.nom, id);
        s.fiche.push((
            pj,
            if ajouter {
                EvenementFiche::EffetTemporaireAjoute(EffetTemporaire {
                    nom,
                    effets: e.contenu.effets.clone(),
                    rounds_restants: None,
                })
            } else {
                EvenementFiche::EffetTemporaireRetire(nom)
            },
        ));
    }

    fn passer_le_temps(
        &mut self,
        rounds: i64,
        ctx: Contexte,
        s: &mut Sortie,
    ) -> Result<(), ErreurAventure> {
        s.minuits += self.horloge.avancer_de(rounds)?;
        let maintenant = self.horloge.maintenant;
        for id in self.effets.avancer_a(maintenant) {
            self.effet_vers_fiche(id, s, false);
            if let Some(t) = self.effets.termines.iter().find(|t| t.effet.id == id) {
                let texte = format!(
                    "« {} » ({}) a pris fin.",
                    t.effet.nom,
                    Self::nom(ctx, t.effet.proprietaire.parse().unwrap_or(0))
                );
                let instant = t.a;
                self.journal.push(Entree { instant, texte });
            }
            s.expires.push(id);
        }
        let ids: Vec<IdPj> = self.pjs.keys().copied().collect();
        for id in ids {
            let Some(c) = ctx(id) else { continue };
            let gueris = self.suivi(id)?.vital.laisser_passer(rounds, c.niveau);
            if gueris > 0 {
                self.noter(format!(
                    "{} récupère {gueris} point(s) de dégâts non létaux.",
                    c.nom
                ));
            }
        }
        Ok(())
    }

    fn retirer_fatigue_si_fini(&mut self, pj: IdPj, s: &mut Sortie) -> Result<(), ErreurAventure> {
        let suivi = self.suivi(pj)?;
        if suivi.fatigue_de_survie && !suivi.vital.a_des_degats_de_survie() {
            suivi.fatigue_de_survie = false;
            s.fiche
                .push((pj, EvenementFiche::EtatRetire("fatigued".into())));
        }
        Ok(())
    }

    fn appliquer_interne(
        &mut self,
        ev: &EvenementAventure,
        ctx: Contexte,
    ) -> Result<Sortie, ErreurAventure> {
        let mut s = Sortie::default();
        let maintenant = self.horloge.maintenant;
        match ev {
            EvenementAventure::Entree { pj } => {
                let c = Self::contexte(ctx, *pj)?;
                self.pjs.entry(*pj).or_insert_with(|| SuiviPj {
                    vital: Vital::default(),
                    survie: Survie::nouvelle(maintenant),
                    fatigue_de_survie: false,
                });
                self.noter(format!("{} rejoint l'aventure.", c.nom));
            }
            EvenementAventure::Pas { pas } => {
                self.passer_le_temps(pas.rounds(), ctx, &mut s)?;
            }
            EvenementAventure::AvancerTemps { rounds } => {
                if *rounds <= 0 {
                    return Err(ErreurAventure::Invalide("durée à zéro".into()));
                }
                self.passer_le_temps(*rounds, ctx, &mut s)?;
            }
            EvenementAventure::RegleHeure {
                instant,
                reactiver,
                retirer_crees,
            } => {
                let nouvel = *instant;
                if self.horloge.en_combat() {
                    return Err(ErreurTemps::HorlogeSuspendue.into());
                }
                if nouvel > maintenant {
                    self.passer_le_temps(nouvel.ecart(maintenant), ctx, &mut s)?;
                } else if nouvel < maintenant {
                    if nouvel.jour() < maintenant.jour() {
                        return Err(ErreurAventure::Invalide(
                            "reculer avant minuit : utiliser Annuler, qui défait aussi le jour d'intermède"
                                .into(),
                        ));
                    }
                    let avant: BTreeSet<IdEffet> =
                        self.effets.actifs.iter().map(|e| e.id).collect();
                    let anciens: Vec<_> = self.effets.actifs.clone();
                    let connus: Vec<_> = self
                        .effets
                        .termines
                        .iter()
                        .map(|t| t.effet.clone())
                        .collect();
                    self.effets
                        .appliquer_recul(nouvel, reactiver, *retirer_crees)?;
                    let apres: BTreeSet<IdEffet> =
                        self.effets.actifs.iter().map(|e| e.id).collect();
                    for id in avant.difference(&apres) {
                        // retiré : on retrouve l'effet dans l'état d'avant
                        if let Some(e) = anciens.iter().find(|e| e.id == *id) {
                            if let Ok(pj) = e.proprietaire.parse::<IdPj>() {
                                s.fiche.push((
                                    pj,
                                    EvenementFiche::EffetTemporaireRetire(nom_dans_la_fiche(
                                        &e.nom, *id,
                                    )),
                                ));
                            }
                        }
                    }
                    for id in apres.difference(&avant) {
                        if let Some(e) = connus.iter().find(|e| e.id == *id) {
                            if let Ok(pj) = e.proprietaire.parse::<IdPj>() {
                                s.fiche.push((
                                    pj,
                                    EvenementFiche::EffetTemporaireAjoute(EffetTemporaire {
                                        nom: nom_dans_la_fiche(&e.nom, *id),
                                        effets: e.contenu.effets.clone(),
                                        rounds_restants: None,
                                    }),
                                ));
                            }
                        }
                    }
                    self.horloge.regler(nouvel)?;
                    for suivi in self.pjs.values_mut() {
                        suivi.survie.recaler(nouvel);
                    }
                    self.noter(format!("Heure reculée : {nouvel}."));
                }
            }
            EvenementAventure::Degats { pj, nombre, letaux } => {
                let c = Self::contexte(ctx, *pj)?;
                if *nombre < 0 {
                    return Err(ErreurAventure::Invalide("dégâts négatifs".into()));
                }
                let v = &mut self.suivi(*pj)?.vital;
                v.prendre_degats(*nombre, *letaux);
                let (pv, nl) = (v.pv_actuels(c.pv_max), v.non_letaux);
                self.noter(format!(
                    "{} subit {nombre} dégât(s) {} : {pv}/{} PV, {nl} non létaux.",
                    c.nom,
                    if *letaux { "létaux" } else { "non létaux" },
                    c.pv_max
                ));
            }
            EvenementAventure::Soin { pj, nombre } => {
                let c = Self::contexte(ctx, *pj)?;
                let r = self.suivi(*pj)?.vital.soigner(*nombre);
                self.noter(format!(
                    "{} reçoit {nombre} point(s) de soin : {} PV rendus, {} non létaux retirés.",
                    c.nom, r.pv_rendus, r.non_letaux_retires
                ));
            }
            EvenementAventure::PvTemporaires { pj, nombre } => {
                let c = Self::contexte(ctx, *pj)?;
                self.suivi(*pj)?.vital.donner_temporaires(*nombre);
                self.noter(format!("{} reçoit {nombre} PV temporaires.", c.nom));
            }
            EvenementAventure::EffetAjoute {
                pj,
                nom,
                source,
                duree,
                effets,
            } => {
                let c = Self::contexte(ctx, *pj)?;
                self.suivi(*pj)?;
                if duree.en_rounds() == Some(0) {
                    return Err(ErreurAventure::Invalide("durée à zéro".into()));
                }
                let id = self.effets.ajouter(
                    nom,
                    source,
                    &pj.to_string(),
                    Contenu {
                        effets: effets.clone(),
                    },
                    duree.clone(),
                    maintenant,
                )?;
                self.effet_vers_fiche(id, &mut s, true);
                self.noter(format!(
                    "{} : « {nom} » ({source}), durée {}.",
                    c.nom,
                    duree.texte()
                ));
            }
            EvenementAventure::EffetRetire { id } => {
                self.effet_vers_fiche(*id, &mut s, false);
                self.effets.retirer(*id, maintenant)?;
                self.noter(format!("Effet n° {id} retiré."));
            }
            EvenementAventure::EffetRetabli { id, duree } => {
                if duree.en_rounds() == Some(0) {
                    return Err(ErreurAventure::Invalide("durée à zéro".into()));
                }
                self.effets.retablir(*id, duree.clone(), maintenant)?;
                self.effet_vers_fiche(*id, &mut s, true);
                self.noter(format!("Effet n° {id} maintenu, durée {}.", duree.texte()));
            }
            EvenementAventure::AlerteConfirmee { id } => {
                self.effets.confirmer(*id)?;
            }
            EvenementAventure::Manger { pj } | EvenementAventure::Boire { pj } => {
                let c = Self::contexte(ctx, *pj)?;
                let boit = matches!(ev, EvenementAventure::Boire { .. });
                let suivi = self.suivi(*pj)?;
                if boit {
                    suivi.survie.boit(maintenant);
                } else {
                    suivi.survie.mange(maintenant);
                }
                suivi.vital.debloquer(boit);
                self.retirer_fatigue_si_fini(*pj, &mut s)?;
                self.noter(format!(
                    "{} {}.",
                    c.nom,
                    if boit { "boit" } else { "mange" }
                ));
            }
            EvenementAventure::TestSurvie {
                pj,
                besoin,
                d20,
                degats,
            } => {
                if !self.survie_active {
                    return Err(ErreurAventure::Invalide(
                        "faim et soif ne sont pas suivies dans cette campagne".into(),
                    ));
                }
                let c = Self::contexte(ctx, *pj)?;
                let suivi = self.suivi(*pj)?;
                let dus = suivi.survie.tests_dus(*besoin, maintenant, c.con_score);
                let Some(test) = dus.first() else {
                    return Err(ErreurAventure::Invalide(format!(
                        "aucun test de {} n'est dû",
                        besoin.nom()
                    )));
                };
                let total = *d20 + c.con_mod;
                let reussi = total >= test.dd;
                match (reussi, degats) {
                    (true, None) => {}
                    (true, Some(_)) => {
                        return Err(ErreurAventure::Invalide(
                            "test réussi : pas de dégâts à saisir".into(),
                        ))
                    }
                    (false, None) => {
                        return Err(ErreurAventure::Invalide(
                            "test raté : saisir le résultat du 1d6 de dégâts non létaux".into(),
                        ))
                    }
                    (false, Some(d)) if !(1..=6).contains(d) => {
                        return Err(ErreurAventure::Invalide("1d6 : de 1 à 6".into()))
                    }
                    (false, Some(d)) => {
                        suivi.vital.non_letaux += d;
                        match besoin {
                            Besoin::Eau => suivi.vital.bloques_soif += d,
                            Besoin::Nourriture => suivi.vital.bloques_faim += d,
                        }
                        if !suivi.fatigue_de_survie {
                            suivi.fatigue_de_survie = true;
                            s.fiche
                                .push((*pj, EvenementFiche::EtatAjoute("fatigued".into())));
                        }
                    }
                }
                suivi.survie.test_passe(*besoin);
                let (dd, nom_besoin) = (test.dd, besoin.nom());
                self.noter(format!(
                    "{} : test de Constitution ({nom_besoin}) {total} contre DD {dd} : {}.",
                    c.nom,
                    match degats {
                        None => "réussi".to_string(),
                        Some(d) => format!("raté, {d} dégât(s) non létaux, fatigué"),
                    }
                ));
            }
            EvenementAventure::Repos {
                pjs,
                heures,
                alitement_complet,
                interrompu,
            } => {
                if pjs.is_empty() || *heures == 0 {
                    return Err(ErreurAventure::Invalide("repos vide".into()));
                }
                if *alitement_complet && *heures < 24 {
                    return Err(ErreurAventure::Invalide(
                        "l'alitement complet demande 24 heures".into(),
                    ));
                }
                for pj in pjs {
                    self.suivi(*pj)?;
                    Self::contexte(ctx, *pj)?;
                }
                self.passer_le_temps(*heures as i64 * ROUNDS_PAR_HEURE, ctx, &mut s)?;
                let nuit_complete = !*interrompu && *heures >= 8;
                for pj in pjs {
                    let c = Self::contexte(ctx, *pj)?;
                    if !nuit_complete {
                        self.noter(format!(
                            "{} se repose {heures} h : repos insuffisant ou interrompu, aucun effet.",
                            c.nom
                        ));
                        continue;
                    }
                    // Guérison naturelle : 1 PV par niveau et par nuit ; alitement complet : le
                    // double par période de 24 h. Dommages de caractéristique : 1 (ou 2) par nuit.
                    let periodes = if *alitement_complet {
                        (*heures / 24) as i32
                    } else {
                        1
                    };
                    let facteur = if *alitement_complet { 2 } else { 1 };
                    let pv = c.niveau as i32 * facteur * periodes;
                    let rendus = self.suivi(*pj)?.vital.guerir_naturellement(pv);
                    let mut caracs = Vec::new();
                    for (carac, dommages) in &c.dommages {
                        let delta = (facteur * periodes).min(*dommages);
                        if delta > 0 {
                            s.fiche.push((
                                *pj,
                                EvenementFiche::DommagesCaracteristique {
                                    carac: *carac,
                                    delta: -delta,
                                },
                            ));
                            caracs.push(format!("{} +{delta}", carac.abrege()));
                        }
                    }
                    s.fiche.push((*pj, EvenementFiche::Repos));
                    self.noter(format!(
                        "{} se repose {heures} h{} : {rendus} PV rendus{}, emplacements de sorts rendus.",
                        c.nom,
                        if *alitement_complet { " (alitement complet)" } else { "" },
                        if caracs.is_empty() {
                            String::new()
                        } else {
                            format!(", caractéristiques récupérées ({})", caracs.join(", "))
                        }
                    ));
                }
            }
        }
        Ok(s)
    }
}
