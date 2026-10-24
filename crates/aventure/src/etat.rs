//! État du mode aventure et événements qui le font évoluer.
//!
//! L'état se **recalcule en rejouant les événements** (comme le reste de la campagne) : annuler,
//! rétablir et la simulation fonctionnent sans code supplémentaire. Chaque événement renvoie une
//! `Sortie` : les événements à répercuter sur les fiches (repos, effets en cours, dommages de
//! caractéristique…) et le nombre de minuits franchis (jours d'intermède à avancer).
//! Le moteur ne recalcule aucun bonus : les effets gardent leurs valeurs typées et c'est la fiche
//! qui les empile.

use crate::survie::{Besoin, Survie, TestDu};
use crate::vital::{Sante, Vital};
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
    /// Bonus de base au jet de Vigueur (fiche).
    pub vigueur: i32,
    /// Modificateur d'initiative de la fiche.
    pub initiative: i32,
    /// Dommages de caractéristique en cours (pour la guérison au repos).
    pub dommages: Vec<(Carac, i32)>,
    /// Valeurs d'incantation de la fiche (lanceurs de sorts seulement).
    pub incantation: Option<IncantationPj>,
}

/// Emplacements de sorts d'un niveau, tels que la fiche les donne au début du combat.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmplacementPj {
    pub niveau_de_sort: u32,
    pub restants: u32,
    pub dd: i32,
}

/// Ce que le lancement de sorts demande à la fiche (calculé par la fiche, jamais par le combat).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IncantationPj {
    /// Classe, nom anglais du catalogue des sorts (`sorcerer`).
    pub classe_en: String,
    /// Nom de la classe dans la fiche (pour décompter les emplacements).
    pub classe: String,
    pub arcanique: bool,
    /// Lanceur spontané : le sort se choisit parmi les sorts connus, sans préparation.
    pub spontane: bool,
    pub niveau_de_lanceur: u32,
    /// Modificateur de la caractéristique d'incantation.
    pub mod_incantation: i32,
    /// Bonus divers aux tests de concentration (dons…).
    pub bonus_concentration: i32,
    pub emplacements: Vec<EmplacementPj>,
    /// Noms anglais des sorts connus (sorts de lignage compris).
    pub sorts_connus: Vec<String>,
    /// Dons de métamagie connus (noms anglais).
    pub metamagies: Vec<String>,
    /// Chance d'échec des sorts profanes due à l'armure et au bouclier (%).
    pub echec_arcanique_pct: u32,
}

pub type Contexte<'a> = &'a dyn Fn(IdPj) -> Option<ContextePj>;

/// Ce que porte un effet temporisé : des effets typés (le temps n'en connaît pas le détail).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Contenu {
    pub effets: Vec<Effet>,
    /// États du Core (`data/core/etats.json`, ex. `shaken`) donnés tant que l'effet dure.
    #[cfg_attr(feature = "serde", serde(default))]
    pub etats: Vec<String>,
    /// PV rendus au début de chaque tour de combat (guérison accélérée, régénération) : valeur
    /// saisie par le joueur, jamais déduite.
    #[cfg_attr(feature = "serde", serde(default))]
    pub soin_par_round: i32,
    #[cfg_attr(feature = "serde", serde(default))]
    pub protections: Vec<Protection>,
}

/// Protection contre les dégâts donnée par un effet.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Protection {
    /// Aucun dégât de ce type (ex. « feu »).
    Immunite { type_degats: String },
    /// Résistance à une énergie : retire `valeur` à chaque attaque de ce type.
    Resistance { type_degats: String, valeur: i32 },
    /// Réduction des dégâts (RD) : retire `valeur` aux dégâts physiques, sauf s'ils la contournent.
    Reduction { valeur: i32 },
}

/// Types d'énergie du Core (acide, froid, électricité, feu, son) : les autres dégâts sont physiques.
pub const TYPES_D_ENERGIE: [&str; 5] = ["acide", "froid", "électricité", "feu", "son"];

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
    /// Dégâts d'un type donné (« feu », « tranchant »…) : immunités, résistances et RD du personnage
    /// s'appliquent, avec décomposition dans le journal. `contourne_rd` : l'attaque passe la RD.
    DegatsTypes {
        pj: IdPj,
        nombre: i32,
        type_degats: String,
        contourne_rd: bool,
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
    /// Jet de stabilisation d'un mourant (d20 physique ; Con. et malus calculés par le moteur).
    Stabilisation {
        pj: IdPj,
        d20: i32,
    },
    /// Jet de Vigueur DD 15 demandé après des dégâts massifs (règle optionnelle) : d20 physique.
    DegatsMassifs {
        pj: IdPj,
        d20: i32,
    },
    /// Début ou fin de dégâts continus (saignement…) : un mourant ne peut pas se stabiliser tant
    /// qu'ils durent ; un soin d'au moins 1 PV les arrête.
    DegatsContinus {
        pj: IdPj,
        actif: bool,
    },
    EffetAjoute {
        pj: IdPj,
        nom: String,
        source: String,
        duree: Duree,
        effets: Vec<Effet>,
        #[cfg_attr(feature = "serde", serde(default))]
        etats: Vec<String>,
        #[cfg_attr(feature = "serde", serde(default))]
        soin_par_round: i32,
        #[cfg_attr(feature = "serde", serde(default))]
        protections: Vec<Protection>,
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
    /// Un jet de Vigueur contre les dégâts massifs est demandé.
    #[cfg_attr(feature = "serde", serde(default))]
    pub massifs_en_attente: bool,
    /// États que ce moteur a déjà donnés à la fiche (santé, effets, fatigue) : sert à n'envoyer
    /// que les différences.
    #[cfg_attr(feature = "serde", serde(default))]
    pub etats_fiche: BTreeSet<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct EtatAventure {
    pub horloge: Horloge,
    /// Règle de campagne : suivre la faim et la soif (désactivable).
    pub survie_active: bool,
    /// Règle optionnelle de campagne : dégâts massifs (Core p. 189).
    #[cfg_attr(feature = "serde", serde(default))]
    pub degats_massifs: bool,
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
            degats_massifs: false,
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

    /// Met fin aux effets « jusqu'au début / à la fin du prochain tour » du personnage.
    pub fn terminer_effets_du_tour(
        &mut self,
        pj: IdPj,
        moment: temps::MomentDuTour,
        ctx: Contexte,
    ) -> Sortie {
        let mut s = Sortie::default();
        let maintenant = self.horloge.maintenant;
        // On repère les effets avant de les retirer, pour répercuter leur fin sur la fiche.
        let ids: Vec<IdEffet> = self
            .effets
            .actifs
            .iter()
            .filter(|e| {
                e.proprietaire == pj.to_string() && e.duree == Duree::JusquauProchainTour(moment)
            })
            .map(|e| e.id)
            .collect();
        for id in ids {
            self.effet_vers_fiche(id, &mut s, false);
            let _ = self.effets.retirer(id, maintenant);
            self.noter(format!("Effet n° {id} terminé avec le tour."));
        }
        self.synchroniser(ctx, &mut s);
        s
    }

    /// Entrée en combat : l'horloge horaire est suspendue, elle avance maintenant round par round.
    pub fn entrer_en_combat(&mut self) -> Result<(), ErreurAventure> {
        self.horloge.entrer_en_combat()?;
        self.noter("Début d'un combat.".into());
        Ok(())
    }

    /// Un round de combat s'achève : l'horloge avance de 6 secondes, les effets arrivés à
    /// échéance prennent fin, les dégâts non létaux guérissent avec le temps.
    pub fn round_de_combat(&mut self, ctx: Contexte) -> Result<Sortie, ErreurAventure> {
        let mut s = Sortie::default();
        self.passer_le_temps(1, ctx, &mut s)?;
        self.synchroniser(ctx, &mut s);
        Ok(s)
    }

    /// Fin d'un combat validé : conversion des durées (spécification §1.2), alertes pour les
    /// effets dissipés, puis retour à l'horloge d'aventure. `contestations` maintient des
    /// effets dissipés avec la durée saisie par le joueur.
    pub fn terminer_combat(
        &mut self,
        contestations: &[(IdEffet, Duree)],
        ctx: Contexte,
    ) -> Result<(temps::BilanFinDeCombat, Sortie), ErreurAventure> {
        let mut s = Sortie::default();
        let maintenant = self.horloge.maintenant;
        let bilan = self.effets.fin_de_combat(maintenant);
        for d in &bilan.dissipes {
            self.effet_vers_fiche(d.id, &mut s, false);
        }
        for (id, duree) in contestations {
            if !bilan.dissipes.iter().any(|d| d.id == *id) {
                return Err(ErreurAventure::Invalide(format!(
                    "l'effet {id} n'a pas été dissipé"
                )));
            }
            self.effets.retablir(*id, duree.clone(), maintenant)?;
            self.effet_vers_fiche(*id, &mut s, true);
        }
        self.horloge.sortir_du_combat(true)?;
        self.synchroniser(ctx, &mut s);
        self.noter(format!(
            "Fin du combat : {} effet(s) dissipé(s), {} maintenu(s) par le joueur.",
            bilan.dissipes.len(),
            contestations.len()
        ));
        Ok((bilan, s))
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
        s.minuits += if self.horloge.en_combat() {
            self.horloge.round_suivant()?
        } else {
            self.horloge.avancer_de(rounds)?
        };
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

    /// La fatigue due à la faim ou à la soif cesse quand plus aucun de leurs dégâts ne reste.
    fn retirer_fatigue_si_fini(&mut self, pj: IdPj) -> Result<(), ErreurAventure> {
        let suivi = self.suivi(pj)?;
        if suivi.fatigue_de_survie && !suivi.vital.a_des_degats_de_survie() {
            suivi.fatigue_de_survie = false;
        }
        Ok(())
    }

    /// États que le personnage doit avoir sur sa fiche : santé, effets en cours, fatigue de survie.
    fn etats_voulus(&self, pj: IdPj, ctx: Contexte) -> Option<BTreeSet<String>> {
        let c = ctx(pj)?;
        let suivi = self.pjs.get(&pj)?;
        let mut v: BTreeSet<String> = suivi
            .vital
            .etats_de_sante(c.pv_max, c.con_score)
            .into_iter()
            .map(String::from)
            .collect();
        if suivi.fatigue_de_survie {
            v.insert("fatigued".into());
        }
        let proprio = pj.to_string();
        for e in self
            .effets
            .actifs
            .iter()
            .filter(|e| e.proprietaire == proprio)
        {
            v.extend(e.contenu.etats.iter().cloned());
        }
        Some(v)
    }

    /// Répercute sur les fiches les changements d'états (ajouts puis retraits, par personnage).
    pub(crate) fn synchroniser(&mut self, ctx: Contexte, s: &mut Sortie) {
        let ids: Vec<IdPj> = self.pjs.keys().copied().collect();
        for pj in ids {
            let Some(voulus) = self.etats_voulus(pj, ctx) else {
                continue;
            };
            let Some(suivi) = self.pjs.get_mut(&pj) else {
                continue;
            };
            for e in voulus.difference(&suivi.etats_fiche) {
                s.fiche.push((pj, EvenementFiche::EtatAjoute(e.clone())));
            }
            for e in suivi.etats_fiche.difference(&voulus) {
                s.fiche.push((pj, EvenementFiche::EtatRetire(e.clone())));
            }
            suivi.etats_fiche = voulus;
        }
    }

    /// Protections actives d'un personnage, avec l'effet qui les donne.
    fn protections(&self, pj: IdPj) -> Vec<(String, Protection)> {
        let proprio = pj.to_string();
        let mut v = Vec::new();
        for e in self
            .effets
            .actifs
            .iter()
            .filter(|e| e.proprietaire == proprio)
        {
            for p in &e.contenu.protections {
                v.push((e.nom.clone(), p.clone()));
            }
        }
        v
    }

    /// Dégâts réellement subis après immunités, résistances et RD, et le détail du calcul.
    /// Plusieurs résistances ou RD : la plus haute s'applique (lecture à valider).
    fn degats_apres_protections(
        &self,
        pj: IdPj,
        nombre: i32,
        type_degats: &str,
        contourne_rd: bool,
    ) -> (i32, String) {
        let t = type_degats.to_lowercase();
        let energie = TYPES_D_ENERGIE.contains(&t.as_str());
        let protections = self.protections(pj);
        let mut detail = format!("{nombre} {type_degats}");
        if let Some((source, _)) = protections.iter().find(
            |(_, p)| matches!(p, Protection::Immunite { type_degats } if type_degats.to_lowercase() == t),
        ) {
            return (0, format!("{detail}, immunité ({source}) = 0"));
        }
        let (retrait, libelle) = if energie {
            protections
                .iter()
                .filter_map(|(src, p)| match p {
                    Protection::Resistance {
                        type_degats,
                        valeur,
                    } if type_degats.to_lowercase() == t => {
                        Some((*valeur, format!("résistance au {t} ({src})")))
                    }
                    _ => None,
                })
                .max_by_key(|(v, _)| *v)
                .unwrap_or((0, String::new()))
        } else if contourne_rd {
            (0, String::new())
        } else {
            protections
                .iter()
                .filter_map(|(src, p)| match p {
                    Protection::Reduction { valeur } => Some((*valeur, format!("RD ({src})"))),
                    _ => None,
                })
                .max_by_key(|(v, _)| *v)
                .unwrap_or((0, String::new()))
        };
        let final_ = (nombre - retrait).max(0);
        if retrait > 0 {
            detail.push_str(&format!(", −{retrait} {libelle}"));
        }
        (final_, format!("{detail} = {final_}"))
    }

    fn appliquer_interne(
        &mut self,
        ev: &EvenementAventure,
        ctx: Contexte,
    ) -> Result<Sortie, ErreurAventure> {
        let mut s = Sortie::default();
        let maintenant = self.horloge.maintenant;
        if self.horloge.en_combat()
            && matches!(
                ev,
                EvenementAventure::Entree { .. }
                    | EvenementAventure::Pas { .. }
                    | EvenementAventure::AvancerTemps { .. }
                    | EvenementAventure::RegleHeure { .. }
                    | EvenementAventure::Repos { .. }
            )
        {
            return Err(ErreurTemps::HorlogeSuspendue.into());
        }
        match ev {
            EvenementAventure::Entree { pj } => {
                let c = Self::contexte(ctx, *pj)?;
                self.pjs.entry(*pj).or_insert_with(|| SuiviPj {
                    vital: Vital::default(),
                    survie: Survie::nouvelle(maintenant),
                    fatigue_de_survie: false,
                    massifs_en_attente: false,
                    etats_fiche: BTreeSet::new(),
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
                self.verifier_massifs(*pj, *nombre, *letaux, &c)?;
            }
            EvenementAventure::Soin { pj, nombre } => {
                let c = Self::contexte(ctx, *pj)?;
                let vital = &mut self.suivi(*pj)?.vital;
                let avant = vital.pv_actuels(c.pv_max);
                let r = vital.soigner(*nombre);
                if r.pv_rendus >= 1 {
                    // un soin d'au moins 1 PV arrête les dégâts continus (p. 190) ; sur un mourant
                    // il le stabilise
                    if avant < 0 {
                        vital.stable = true;
                    }
                    vital.continu = false;
                }
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
                etats,
                soin_par_round,
                protections,
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
                        etats: etats.clone(),
                        soin_par_round: *soin_par_round,
                        protections: protections.clone(),
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
                self.retirer_fatigue_si_fini(*pj)?;
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
                        suivi.fatigue_de_survie = true;
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
            EvenementAventure::DegatsTypes {
                pj,
                nombre,
                type_degats,
                contourne_rd,
                letaux,
            } => {
                let c = Self::contexte(ctx, *pj)?;
                if *nombre < 0 {
                    return Err(ErreurAventure::Invalide("dégâts négatifs".into()));
                }
                let (finaux, detail) =
                    self.degats_apres_protections(*pj, *nombre, type_degats, *contourne_rd);
                let v = &mut self.suivi(*pj)?.vital;
                v.prendre_degats(finaux, *letaux);
                let pv = v.pv_actuels(c.pv_max);
                self.noter(format!(
                    "{} : {detail} dégât(s) {} : {pv}/{} PV.",
                    c.nom,
                    if *letaux { "létaux" } else { "non létaux" },
                    c.pv_max
                ));
                self.verifier_massifs(*pj, finaux, *letaux, &c)?;
            }
            EvenementAventure::Stabilisation { pj, d20 } => {
                let c = Self::contexte(ctx, *pj)?;
                if !(1..=20).contains(d20) {
                    return Err(ErreurAventure::Invalide("d20 : de 1 à 20".into()));
                }
                let v = &mut self.suivi(*pj)?.vital;
                let pv = v.pv_actuels(c.pv_max);
                if v.sante(c.pv_max, c.con_score) != Sante::Mourant {
                    return Err(ErreurAventure::Invalide(format!(
                        "{} n'est pas mourant : pas de jet de stabilisation",
                        c.nom
                    )));
                }
                if v.continu {
                    return Err(ErreurAventure::Invalide(format!(
                        "{} subit des dégâts continus : il faut d'abord un soin, puis le jet de stabilisation",
                        c.nom
                    )));
                }
                // DD 10, malus égal au total négatif de PV ; 20 naturel : réussite (p. 190)
                let total = *d20 + c.con_mod + pv;
                let reussi = *d20 == 20 || total >= 10;
                if reussi {
                    v.stable = true;
                } else {
                    v.degats += 1;
                }
                let apres = v.pv_actuels(c.pv_max);
                self.noter(format!(
                    "{} : jet de stabilisation {d20} + {} (Con.) {pv} (PV négatifs) = {total} contre DD 10 : {}.",
                    c.nom,
                    c.con_mod,
                    if reussi {
                        "stabilisé".to_string()
                    } else {
                        format!("raté, perd 1 PV ({apres}/{})", c.pv_max)
                    }
                ));
            }
            EvenementAventure::DegatsMassifs { pj, d20 } => {
                let c = Self::contexte(ctx, *pj)?;
                if !(1..=20).contains(d20) {
                    return Err(ErreurAventure::Invalide("d20 : de 1 à 20".into()));
                }
                let suivi = self.suivi(*pj)?;
                if !suivi.massifs_en_attente {
                    return Err(ErreurAventure::Invalide(format!(
                        "aucun jet de dégâts massifs n'est demandé pour {}",
                        c.nom
                    )));
                }
                suivi.massifs_en_attente = false;
                let total = *d20 + c.vigueur;
                // jet de sauvegarde : 20 naturel réussit, 1 naturel rate
                let reussi = *d20 == 20 || (*d20 != 1 && total >= 15);
                if !reussi {
                    // la mort, quels que soient les PV restants (p. 189)
                    suivi.vital.degats = suivi.vital.degats.max(c.pv_max + c.con_score);
                }
                self.noter(format!(
                    "{} : dégâts massifs, Vigueur {d20} + {} = {total} contre DD 15 : {}.",
                    c.nom,
                    c.vigueur,
                    if reussi { "réussi" } else { "raté, il meurt" }
                ));
            }
            EvenementAventure::DegatsContinus { pj, actif } => {
                let c = Self::contexte(ctx, *pj)?;
                self.suivi(*pj)?.vital.continu = *actif;
                self.noter(format!(
                    "{} : dégâts continus {}.",
                    c.nom,
                    if *actif { "en cours" } else { "arrêtés" }
                ));
            }
        }
        self.synchroniser(ctx, &mut s);
        Ok(s)
    }

    /// Un jet de Vigueur contre les dégâts massifs est-il demandé ? Règle optionnelle (p. 189) :
    /// une seule attaque d'au moins 50 PV et d'au moins la moitié des PV du personnage, sans le tuer.
    fn verifier_massifs(
        &mut self,
        pj: IdPj,
        degats_finaux: i32,
        letaux: bool,
        c: &ContextePj,
    ) -> Result<(), ErreurAventure> {
        if !self.degats_massifs || !letaux {
            return Ok(());
        }
        let seuil = 50.max((c.pv_max + 1) / 2);
        let suivi = self.suivi(pj)?;
        if degats_finaux >= seuil && suivi.vital.sante(c.pv_max, c.con_score) != Sante::Mort {
            suivi.massifs_en_attente = true;
            self.noter(format!(
                "{} : dégâts massifs (au moins {seuil}) : jet de Vigueur DD 15 demandé.",
                c.nom
            ));
        }
        Ok(())
    }

    /// Jets de dégâts massifs demandés au joueur.
    pub fn massifs_demandes(&self) -> Vec<IdPj> {
        self.pjs
            .iter()
            .filter(|(_, s)| s.massifs_en_attente)
            .map(|(id, _)| *id)
            .collect()
    }

    /// Pertes de début de tour d'un mourant qui subit des dégâts continus : le jet de
    /// stabilisation est raté d'office (1 PV) et 1 PV de plus est perdu (p. 190, lecture à valider).
    pub fn pertes_continues_de_debut_de_tour(&mut self, pj: IdPj, ctx: Contexte) -> Sortie {
        let mut s = Sortie::default();
        if let (Some(c), Some(suivi)) = (ctx(pj), self.pjs.get_mut(&pj)) {
            if suivi.vital.continu && suivi.vital.sante(c.pv_max, c.con_score) == Sante::Mourant {
                suivi.vital.degats += 2;
                let pv = suivi.vital.pv_actuels(c.pv_max);
                self.noter(format!(
                    "{} : dégâts continus, jet de stabilisation raté d'office et 1 PV de plus perdu ({pv}/{}).",
                    c.nom, c.pv_max
                ));
            }
        }
        self.synchroniser(ctx, &mut s);
        s
    }

    /// Soins de début de tour (guérison accélérée, régénération) donnés par les effets en cours.
    pub fn soins_de_debut_de_tour(&mut self, pj: IdPj, ctx: Contexte) -> Sortie {
        let mut s = Sortie::default();
        let proprio = pj.to_string();
        let total: i32 = self
            .effets
            .actifs
            .iter()
            .filter(|e| e.proprietaire == proprio)
            .map(|e| e.contenu.soin_par_round)
            .sum();
        if total > 0 {
            if let (Some(c), Some(suivi)) = (ctx(pj), self.pjs.get_mut(&pj)) {
                let avant = suivi.vital.pv_actuels(c.pv_max);
                let r = suivi.vital.soigner(total);
                if avant < 0 && r.pv_rendus >= 1 {
                    suivi.vital.stable = true;
                }
                self.noter(format!(
                    "{} récupère {} PV (effets de soin par round : {total}).",
                    c.nom, r.pv_rendus
                ));
            }
        }
        self.synchroniser(ctx, &mut s);
        s
    }
}
