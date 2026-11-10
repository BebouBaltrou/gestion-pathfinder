//! Convocation d'alliés naturels en combat (Summon Nature's Ally I à IX) : les créatures
//! invoquées entrent dans l'ordre d'initiative et sont jouées comme les personnages, avec leur
//! fiche (PV, CA, jets, caractéristiques). Elles ne vivent que pendant le combat : à la validation,
//! elles ne sont pas recopiées dans la campagne.
//!
//! Point à confirmer : durée (1 round par niveau) non décomptée, les créatures restent jusqu'à la
//! fin du combat ; un seul d20 d'initiative pour tout le groupe.

use crate::economie::TypeAction;
use crate::etat::{ActionCombat, ErreurCombat, EtatCombat, Init, Phase};
use crate::sorts::{nom_fr, referentiel};
use aventure::Survie;
use aventure::Vital;
use aventure::{ContextePj, ErreurAventure, IdPj, SuiviPj};
use fiches::convocation::{niveau_de_convocation, nombre_pour, Creature};
use fiches::journal::EvenementFiche;
use std::collections::{BTreeMap, BTreeSet};

/// Premier identifiant donné aux créatures invoquées : au-delà, ce n'est pas un personnage.
pub const BASE_INVOCATIONS: IdPj = 1_000_000;

/// Une créature invoquée, présente pendant le combat seulement.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Invocation {
    pub id: IdPj,
    /// Personnage qui a convoqué la créature.
    pub lanceur: IdPj,
    pub creature: Creature,
    /// Niveau du sort qui l'a convoquée.
    pub niveau_sort: u32,
}

impl Invocation {
    /// Valeurs de fiche attendues par le mode aventure, tirées de la créature.
    pub fn contexte(&self) -> ContextePj {
        let con_score = self
            .creature
            .caracteristiques
            .get("Con")
            .copied()
            .unwrap_or(10);
        ContextePj {
            nom: self.creature.nom.clone(),
            niveau: self.creature.niveau,
            pv_max: self.creature.pv,
            con_score,
            con_mod: fiches::caracteristiques::modificateur(con_score),
            vigueur: self.creature.sauvegardes.get("Vig").copied().unwrap_or(0),
            initiative: self.creature.init,
            dommages: Vec::new(),
            incantation: None,
        }
    }
}

fn erreur<T>(m: impl Into<String>) -> Result<T, ErreurCombat> {
    Err(ErreurCombat::Invalide(m.into()))
}

/// Vrai si `id` désigne une créature invoquée et non un personnage.
pub fn est_invocation(id: IdPj) -> bool {
    id >= BASE_INVOCATIONS
}

impl EtatCombat {
    /// Contexte qui connaît aussi les créatures invoquées, en plus de celui des personnages.
    pub fn avec_invocations<'a>(
        &'a self,
        ctx: &'a dyn Fn(IdPj) -> Option<ContextePj>,
    ) -> impl Fn(IdPj) -> Option<ContextePj> + 'a {
        move |id| {
            self.invocations
                .get(&id)
                .map(Invocation::contexte)
                .or_else(|| ctx(id))
        }
    }

    /// Convoque `nombre` créatures nommées `creature` avec le sort `sort` (« Summon Nature's Ally N »).
    /// Consomme un emplacement du niveau du sort ; un seul d20 d'initiative pour le groupe.
    pub(crate) fn convoquer(
        &mut self,
        lanceur: IdPj,
        sort: &str,
        creature: &str,
        nombre: u32,
        d20: i32,
        ctx: &dyn Fn(IdPj) -> Option<ContextePj>,
    ) -> Result<(), ErreurCombat> {
        if self.phase != Phase::EnCours {
            return erreur("le combat n'est pas commencé");
        }
        self.verifier_engage(lanceur)?;
        let niveau = niveau_de_convocation(sort).ok_or_else(|| {
            ErreurCombat::Invalide(format!(
                "{sort} n'est pas une convocation d'alliés naturels"
            ))
        })?;
        let c = ctx(lanceur).ok_or(ErreurAventure::PjInconnu(lanceur))?;
        let inc = c
            .incantation
            .clone()
            .ok_or_else(|| ErreurCombat::Invalide(format!("{} ne lance pas de sorts", c.nom)))?;
        let fiche_creature = referentiel()
            .convocations
            .creatures
            .iter()
            .find(|x| x.nom == creature && x.complete)
            .ok_or_else(|| {
                ErreurCombat::Invalide(format!(
                    "{creature} : créature absente ou sans statistiques"
                ))
            })?;
        let regle = nombre_pour(niveau, fiche_creature.niveau).ok_or_else(|| {
            ErreurCombat::Invalide(format!(
                "{} (niveau {}) n'est pas convocable par un sort de niveau {niveau}",
                fiche_creature.nom, fiche_creature.niveau
            ))
        })?;
        if !regle.valide(nombre) {
            let (min, max) = regle.intervalle();
            return erreur(format!(
                "nombre de créatures : de {min} à {max} pour ce sort"
            ));
        }
        if !(1..=20).contains(&d20) {
            return erreur("d20 : de 1 à 20");
        }
        let emplacement = inc
            .emplacements
            .iter()
            .find(|e| e.niveau_de_sort == niveau)
            .ok_or_else(|| {
                ErreurCombat::Invalide(format!(
                    "{} n'a pas d'emplacement de niveau {niveau}",
                    c.nom
                ))
            })?;
        let depenses = self
            .emplacements_depenses
            .get(&(lanceur, niveau))
            .copied()
            .unwrap_or(0);
        if emplacement.restants <= depenses {
            return erreur(format!(
                "plus d'emplacement de sort de niveau {niveau} pour {}",
                c.nom
            ));
        }
        // économie d'actions (une convocation dure un round : action complexe)
        self.appliquer_interne(
            &ActionCombat::Agir {
                pj: lanceur,
                type_action: TypeAction::Complexe,
                libelle: format!("Convoquer {}", nom_fr(sort)),
                deplacement: false,
            },
            ctx,
        )?;
        *self
            .emplacements_depenses
            .entry((lanceur, niveau))
            .or_default() += 1;
        self.fiche.push((
            lanceur,
            EvenementFiche::SortLance {
                classe: inc.classe.clone(),
                niveau_de_sort: niveau,
                sort: sort.to_string(),
            },
        ));
        let fiche_creature = fiche_creature.clone();
        let maintenant = self.aventure.horloge.maintenant;
        for _ in 0..nombre {
            let id = self.prochain_invocation;
            self.prochain_invocation += 1;
            self.aventure.pjs.insert(
                id,
                SuiviPj {
                    vital: Vital::default(),
                    survie: Survie::nouvelle(maintenant),
                    fatigue_de_survie: false,
                    massifs_en_attente: false,
                    etats_fiche: BTreeSet::new(),
                    inventaire: BTreeMap::new(),
                    ressources: BTreeMap::new(),
                },
            );
            self.engages.push(id);
            self.initiatives.insert(
                id,
                Init {
                    d20,
                    modificateur: fiche_creature.init,
                    total: d20 + fiche_creature.init,
                    departage: None,
                },
            );
            self.invocations.insert(
                id,
                Invocation {
                    id,
                    lanceur,
                    creature: fiche_creature.clone(),
                    niveau_sort: niveau,
                },
            );
        }
        self.trier();
        self.noter(format!(
            "{} convoque {nombre} × {} (d20 {d20}).",
            c.nom, fiche_creature.nom
        ));
        Ok(())
    }
}

impl EtatCombat {
    /// Prend une forme animale ou élémentaire (sorts forme bestiale, corps élémentaire, forme
    /// végétale) : les effets chiffrés du livre s'appliquent au personnage pour la durée du sort.
    pub(crate) fn prendre_forme(
        &mut self,
        pj: IdPj,
        sort: &str,
        forme: &str,
        ctx: &dyn Fn(IdPj) -> Option<ContextePj>,
    ) -> Result<(), ErreurCombat> {
        if self.phase != Phase::EnCours {
            return erreur("le combat n'est pas commencé");
        }
        self.verifier_engage(pj)?;
        let c = ctx(pj).ok_or(ErreurAventure::PjInconnu(pj))?;
        let inc = c
            .incantation
            .clone()
            .ok_or_else(|| ErreurCombat::Invalide(format!("{} ne lance pas de sorts", c.nom)))?;
        let f = referentiel()
            .formes
            .iter()
            .find(|f| f.id == forme && f.sort_en.eq_ignore_ascii_case(sort))
            .ok_or_else(|| {
                ErreurCombat::Invalide(format!("{forme} n'est pas une forme de {sort}"))
            })?;
        let fiche_sort = referentiel()
            .sort(sort)
            .ok_or_else(|| ErreurCombat::Invalide(format!("sort inconnu : {sort}")))?;
        let niveau = fiche_sort.niveau_pour(&inc.classe_en).ok_or_else(|| {
            ErreurCombat::Invalide(format!("{sort} n'est pas un sort de {}", inc.classe_en))
        })?;
        // une seule forme à la fois
        let prefixe = fiches::formes::PREFIXE_FORME;
        if self
            .aventure
            .effets
            .actifs
            .iter()
            .any(|t| t.proprietaire == pj.to_string() && t.nom.starts_with(prefixe))
        {
            return erreur(format!("{} est déjà sous une forme", c.nom));
        }
        let emplacement = inc
            .emplacements
            .iter()
            .find(|e| e.niveau_de_sort == niveau)
            .ok_or_else(|| {
                ErreurCombat::Invalide(format!(
                    "{} n'a pas d'emplacement de niveau {niveau}",
                    c.nom
                ))
            })?;
        let depenses = self
            .emplacements_depenses
            .get(&(pj, niveau))
            .copied()
            .unwrap_or(0);
        if emplacement.restants <= depenses {
            return erreur(format!(
                "plus d'emplacement de sort de niveau {niveau} pour {}",
                c.nom
            ));
        }
        let duree = crate::sorts::duree_suggeree(
            fiche_sort.duree_en.as_deref().unwrap_or(""),
            inc.niveau_de_lanceur,
        )
        .ok_or_else(|| {
            ErreurCombat::Invalide(format!("durée de {sort} : à saisir (point à confirmer)"))
        })?;
        // action simple (sorts de forme : 1 action simple)
        self.appliquer_interne(
            &ActionCombat::Agir {
                pj,
                type_action: TypeAction::Simple,
                libelle: format!("Prendre la forme {}", f.nom),
                deplacement: false,
            },
            ctx,
        )?;
        *self.emplacements_depenses.entry((pj, niveau)).or_default() += 1;
        self.fiche.push((
            pj,
            EvenementFiche::SortLance {
                classe: inc.classe.clone(),
                niveau_de_sort: niveau,
                sort: sort.to_string(),
            },
        ));
        let ev = aventure::EvenementAventure::EffetAjoute {
            pj,
            nom: f.nom_effet(),
            source: format!("sort {}, forme {}", sort, f.nom),
            duree,
            effets: f.effets_chiffres(),
            etats: Vec::new(),
            soin_par_round: 0,
            protections: Vec::new(),
        };
        let s = self.aventure.appliquer(&ev, ctx)?;
        self.fusionner(s);
        self.noter(format!("{} prend la forme {}.", c.nom, f.nom));
        Ok(())
    }
}
