//! Lancement de sorts en combat : emplacements, métamagie, DD, niveau de lanceur, lancement
//! défensif, échec des sorts profanes, effets sur les personnages joués, concentration.
//!
//! Le moteur ne calcule ni DD ni bonus : il lit ce que la fiche fournit (`IncantationPj`). Les dés
//! sont physiques : l'aperçu dit quoi lancer, l'action reçoit les résultats saisis.
//! Sources : métamagie `data/combat/metamagie.json`, effets `data/combat/sorts_effets.json`,
//! concentration Core p. 206, échec des sorts profanes p. 83 et 208.

use crate::economie::TypeAction;
use crate::etat::{ActionCombat, ErreurCombat, EtatCombat, Phase};
use aventure::{
    ContextePj, ErreurAventure, EvenementAventure, IdPj, IncantationPj, Protection, TYPES_D_ENERGIE,
};
use fiches::{catalogue::Sort, Effet, Referentiel, TypeBonus};
use serde::Deserialize;
use std::sync::OnceLock;
use temps::{Duree, IdEffet};

const METAMAGIE: &str = include_str!("../../../data/combat/metamagie.json");
const EFFETS: &str = include_str!("../../../data/combat/sorts_effets.json");

#[derive(Debug, Clone, Deserialize)]
pub struct Metamagie {
    pub nom_en: String,
    pub delta: Option<u32>,
    pub page: u32,
    #[serde(default)]
    pub double_duree: bool,
    #[serde(default)]
    pub rapide: bool,
    #[serde(default)]
    pub sans_verbal: bool,
    #[serde(default)]
    pub sans_gestes: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EffetData {
    pub cible: String,
    #[serde(rename = "type")]
    pub type_bonus: TypeBonus,
    pub valeur: i32,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "genre", rename_all = "snake_case")]
pub enum ProtectionData {
    Reduction {
        valeur: i32,
    },
    /// `paliers` : (niveau de lanceur minimum, valeur). Le type d'énergie est choisi au lancement.
    Resistance {
        paliers: Vec<(u32, i32)>,
    },
    Immunite {
        type_degats: String,
    },
}

#[derive(Debug, Clone, Deserialize)]
pub struct EffetSort {
    pub nom_en: String,
    pub page: u32,
    #[serde(default)]
    pub effets: Vec<EffetData>,
    #[serde(default)]
    pub etats: Vec<String>,
    #[serde(default)]
    pub protections: Vec<ProtectionData>,
    #[serde(default)]
    pub choix_energie: bool,
    #[serde(default)]
    pub notes: Option<String>,
}

#[derive(Deserialize)]
struct FichierMeta {
    metamagies: Vec<Metamagie>,
}

#[derive(Deserialize)]
struct FichierEffets {
    sorts: Vec<EffetSort>,
}

fn referentiel() -> &'static Referentiel {
    static R: OnceLock<Referentiel> = OnceLock::new();
    R.get_or_init(Referentiel::core)
}

pub fn metamagies() -> &'static [Metamagie] {
    static M: OnceLock<Vec<Metamagie>> = OnceLock::new();
    M.get_or_init(|| {
        serde_json::from_str::<FichierMeta>(METAMAGIE)
            .expect("data/combat/metamagie.json est valide")
            .metamagies
    })
}

pub fn effets_des_sorts() -> &'static [EffetSort] {
    static E: OnceLock<Vec<EffetSort>> = OnceLock::new();
    E.get_or_init(|| {
        serde_json::from_str::<FichierEffets>(EFFETS)
            .expect("data/combat/sorts_effets.json est valide")
            .sorts
    })
}

fn effet_du_sort(nom_en: &str) -> Option<&'static EffetSort> {
    effets_des_sorts()
        .iter()
        .find(|e| e.nom_en.eq_ignore_ascii_case(nom_en))
}

/// Durée proposée d'après le texte du livre quand la formule est simple (« 1 min./level »).
/// Le joueur valide ou corrige ; `None` si le texte n'est pas une formule simple.
pub fn duree_suggeree(texte: &str, niveau_de_lanceur: u32) -> Option<Duree> {
    let t = texte.trim().to_lowercase();
    if t.starts_with("instantaneous") {
        return Some(Duree::Instantanee);
    }
    if t.starts_with("permanent") {
        return Some(Duree::Permanente);
    }
    if t.starts_with("concentration") {
        return Some(Duree::Concentration);
    }
    let chiffres: String = t.chars().take_while(|c| c.is_ascii_digit()).collect();
    let n: u32 = chiffres.parse().ok()?;
    let reste = t[chiffres.len()..].trim_start();
    let (unite, apres) = [
        ("round", 'r'),
        ("min", 'm'),
        ("hour", 'h'),
        ("hr", 'h'),
        ("day", 'j'),
    ]
    .iter()
    .find_map(|(mot, u)| reste.strip_prefix(mot).map(|r| (*u, r)))?;
    // sauter la fin du mot : « s », « .», « ute(s) »
    let apres = apres.trim_start_matches(|c: char| c.is_alphabetic() || c == '.');
    let par_niveau = apres.trim_start().starts_with("/level");
    let total = if par_niveau {
        n.checked_mul(niveau_de_lanceur)?
    } else {
        n
    };
    if total == 0 {
        return None;
    }
    Some(match unite {
        'r' => Duree::Rounds(total),
        'm' => Duree::Minutes(total),
        'h' => Duree::Heures(total),
        _ => Duree::Jours(total),
    })
}

fn doubler(d: Duree) -> Duree {
    match d {
        Duree::Rounds(n) => Duree::Rounds(n.saturating_mul(2)),
        Duree::Minutes(n) => Duree::Minutes(n.saturating_mul(2)),
        Duree::Heures(n) => Duree::Heures(n.saturating_mul(2)),
        Duree::Jours(n) => Duree::Jours(n.saturating_mul(2)),
        autre => autre,
    }
}

/// Composantes verbale et gestuelle d'après « V, S, M (…) ».
fn composantes(texte: Option<&str>) -> (bool, bool) {
    let t = texte.unwrap_or("");
    let avant = t.split('(').next().unwrap_or("");
    let jetons: Vec<&str> = avant.split(',').map(str::trim).collect();
    (jetons.contains(&"V"), jetons.contains(&"S"))
}

/// Type d'action pour lancer le sort d'après son temps d'incantation. Un lanceur spontané qui
/// applique de la métamagie lance en action complexe un sort qui prend une action simple,
/// sauf pour le sort accéléré (lecture à valider, voir `docs/POINTS_A_CONFIRMER_SORTS.md`).
pub fn type_d_action(
    temps: Option<&str>,
    spontane: bool,
    metamagie: bool,
    accelere: bool,
) -> Result<TypeAction, String> {
    let t = temps.unwrap_or("").to_lowercase();
    if accelere {
        return if t.contains("swift")
            || t.contains("immediate")
            || t.contains("standard")
            || t == "1 full-round action"
        {
            Ok(TypeAction::Rapide)
        } else {
            Err(format!("« {t} » ne peut pas être accéléré"))
        };
    }
    let base = if t.starts_with("1 standard action") {
        TypeAction::Simple
    } else if t.starts_with("1 full-round action") || t.starts_with("1 round") {
        TypeAction::Complexe
    } else if t.starts_with("1 swift action") {
        TypeAction::Rapide
    } else if t.starts_with("1 immediate action") {
        TypeAction::Immediate
    } else {
        return Err(format!(
            "temps d'incantation « {t} » : pas un sort de combat (action simple, complexe, rapide ou immédiate)"
        ));
    };
    if spontane && metamagie {
        return match base {
            TypeAction::Simple => Ok(TypeAction::Complexe),
            TypeAction::Complexe => Err(
                "métamagie spontanée sur un sort d'une action complexe ou plus : non gérée en combat"
                    .into(),
            ),
            autre => Ok(autre),
        };
    }
    Ok(base)
}

/// Ce que le joueur doit savoir (et lancer) avant de confirmer un lancement.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApercuSort {
    pub nom_en: String,
    pub niveau_du_sort: u32,
    /// Niveau du sort après Heighten Spell (celui du DD).
    pub niveau_effectif: u32,
    /// Niveau de l'emplacement dépensé (0 = tour de magie, sans emplacement).
    pub niveau_emplacement: u32,
    pub restants: u32,
    pub dd: i32,
    pub niveau_de_lanceur: u32,
    pub type_action: TypeAction,
    pub verbal: bool,
    pub gestuel: bool,
    /// Pourcentage d'échec à lancer au d100 (0 si aucun).
    pub echec_arcanique_pct: u32,
    /// Lancement défensif : DD du test (d20 + niveau de lanceur + mod. d'incantation + bonus).
    pub dd_lancement_defensif: i32,
    pub bonus_concentration: i32,
    pub duree_en: Option<String>,
    pub duree_suggeree: Option<Duree>,
    pub portee_en: Option<String>,
    pub jet_de_sauvegarde_en: Option<String>,
    pub description_en: String,
    pub effet_automatique: bool,
    pub choix_energie: bool,
    pub notes: Option<String>,
}

/// Demande de lancement : tout ce qui est saisi ou choisi par le joueur.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ParamsLancer {
    pub pj: IdPj,
    /// Nom anglais du catalogue.
    pub sort: String,
    #[cfg_attr(feature = "serde", serde(default))]
    pub metamagies: Vec<String>,
    /// Niveau visé par Heighten Spell.
    #[cfg_attr(feature = "serde", serde(default))]
    pub niveau_heighten: Option<u32>,
    /// Lancement défensif : résultat du d20 du test de concentration.
    #[cfg_attr(feature = "serde", serde(default))]
    pub defensif_d20: Option<i32>,
    /// Résultat du d100 si le sort est soumis à un échec arcanique.
    #[cfg_attr(feature = "serde", serde(default))]
    pub echec_arcanique_d100: Option<i32>,
    /// Personnages joués qui reçoivent l'effet (le lanceur lui-même compris).
    #[cfg_attr(feature = "serde", serde(default))]
    pub cibles: Vec<IdPj>,
    /// Durée saisie ; à défaut, la durée suggérée d'après le livre.
    #[cfg_attr(feature = "serde", serde(default))]
    pub duree: Option<Duree>,
    /// Type d'énergie choisi (résistance à l'énergie) : acide, froid, électricité, feu, son.
    #[cfg_attr(feature = "serde", serde(default))]
    pub choix_energie: Option<String>,
    /// Résultat d'un dé lancé pour le sort (dégâts, soins…), noté dans le journal.
    #[cfg_attr(feature = "serde", serde(default))]
    pub resultat_de: Option<i32>,
}

/// Cause d'une distraction qui oblige à un test de concentration (Core p. 206).
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Distraction {
    /// Dégâts subis : 10 + dégâts + niveau du sort.
    Blessure { degats: i32 },
    /// Dégâts continus : 10 + la moitié des derniers dégâts + niveau du sort.
    DegatsContinus { derniers_degats: i32 },
    /// Sort non destructeur qui nous affecte : DD de son jet + niveau du sort.
    SortSurSoi { dd_du_sort: i32 },
    /// Agrippé ou immobilisé : 10 + BMC de l'adversaire + niveau du sort.
    Agrippe { bmc: i32 },
    /// Mouvement vigoureux (monture, chariot…) : 10 + niveau du sort.
    MouvementVigoureux,
    /// Mouvement violent : 15 + niveau du sort.
    MouvementViolent,
    /// Mouvement extrêmement violent : 20 + niveau du sort.
    MouvementTresViolent,
    /// Entravé (filet…) : 15 + niveau du sort.
    Entrave,
    /// Mauvais temps, pluie ou grésil : 5 + niveau du sort.
    Intemperies,
    /// Mauvais temps, grêle, poussière, débris : 10 + niveau du sort.
    IntemperiesFortes,
    /// Autre cas : DD donné par le livre ou le MJ.
    Libre { dd: i32 },
}

impl Distraction {
    pub fn dd(&self, niveau_du_sort: u32) -> i32 {
        let n = niveau_du_sort as i32;
        match self {
            Distraction::Blessure { degats } => 10 + degats + n,
            Distraction::DegatsContinus { derniers_degats } => 10 + derniers_degats / 2 + n,
            Distraction::SortSurSoi { dd_du_sort } => dd_du_sort + n,
            Distraction::Agrippe { bmc } => 10 + bmc + n,
            Distraction::MouvementVigoureux => 10 + n,
            Distraction::MouvementViolent => 15 + n,
            Distraction::MouvementTresViolent => 20 + n,
            Distraction::Entrave => 15 + n,
            Distraction::Intemperies => 5 + n,
            Distraction::IntemperiesFortes => 10 + n,
            Distraction::Libre { dd } => *dd,
        }
    }
}

struct Preparation {
    sort: &'static Sort,
    inc: IncantationPj,
    niveau_du_sort: u32,
    niveau_effectif: u32,
    niveau_emplacement: u32,
    metas: Vec<&'static Metamagie>,
    restants: u32,
    dd: i32,
    type_action: TypeAction,
    verbal: bool,
    gestuel: bool,
    echec_pct: u32,
}

fn invalide<T>(m: impl Into<String>) -> Result<T, ErreurCombat> {
    Err(ErreurCombat::Invalide(m.into()))
}

impl EtatCombat {
    fn preparer_sort(
        &self,
        pj: IdPj,
        nom_sort: &str,
        metamagies_choisies: &[String],
        heighten: Option<u32>,
        ctx: &dyn Fn(IdPj) -> Option<ContextePj>,
    ) -> Result<Preparation, ErreurCombat> {
        let c = ctx(pj).ok_or(ErreurAventure::PjInconnu(pj))?;
        let Some(inc) = c.incantation else {
            return invalide(format!("{} ne lance pas de sorts", c.nom));
        };
        if !inc.spontane {
            return invalide("lanceur à sorts préparés : non géré (seul le lanceur spontané l'est pour l'instant)");
        }
        let sort = referentiel()
            .sort(nom_sort)
            .ok_or_else(|| ErreurCombat::Invalide(format!("sort inconnu : {nom_sort}")))?;
        if !inc
            .sorts_connus
            .iter()
            .any(|s| s.eq_ignore_ascii_case(&sort.nom_en))
        {
            return invalide(format!("{} ne connaît pas {}", c.nom, sort.nom_en));
        }
        let niveau_du_sort = sort.niveau_pour(&inc.classe_en).ok_or_else(|| {
            ErreurCombat::Invalide(format!(
                "{} n'est pas un sort de {}",
                sort.nom_en, inc.classe_en
            ))
        })?;
        if !metamagies_choisies.is_empty() && !inc.metamagie_active {
            return invalide(
                "la métamagie n'est pas encore intégrée (réglage de la fiche à activer)",
            );
        }
        let mut metas = Vec::new();
        let mut delta = 0;
        for m in metamagies_choisies {
            let def = metamagies()
                .iter()
                .find(|d| d.nom_en.eq_ignore_ascii_case(m))
                .ok_or_else(|| ErreurCombat::Invalide(format!("métamagie inconnue : {m}")))?;
            if !inc
                .metamagies
                .iter()
                .any(|k| k.eq_ignore_ascii_case(&def.nom_en))
            {
                return invalide(format!("{} ne connaît pas le don {}", c.nom, def.nom_en));
            }
            if metas.iter().any(|x: &&Metamagie| x.nom_en == def.nom_en) {
                return invalide(format!("{} ne s'applique qu'une fois", def.nom_en));
            }
            delta += def.delta.unwrap_or(0);
            metas.push(def);
        }
        let niveau_effectif = match (heighten, metas.iter().any(|m| m.nom_en == "Heighten Spell")) {
            (Some(n), true) => {
                if n <= niveau_du_sort || n > 9 {
                    return invalide("Heighten Spell : un niveau plus haut que le sort, 9 au plus");
                }
                n
            }
            (None, true) => return invalide("Heighten Spell : préciser le niveau visé"),
            (Some(_), false) => return invalide("niveau visé sans Heighten Spell"),
            (None, false) => niveau_du_sort,
        };
        let niveau_emplacement = if niveau_effectif == 0 && delta == 0 {
            0
        } else {
            niveau_effectif + delta
        };
        if niveau_emplacement > 9 {
            return invalide(format!(
                "emplacement de niveau {niveau_emplacement} : au plus 9"
            ));
        }
        let restants = if niveau_emplacement == 0 {
            u32::MAX // tours de magie : à volonté
        } else {
            let e = inc
                .emplacements
                .iter()
                .find(|e| e.niveau_de_sort == niveau_emplacement)
                .ok_or_else(|| {
                    ErreurCombat::Invalide(format!(
                        "{} n'a pas d'emplacement de niveau {niveau_emplacement}",
                        c.nom
                    ))
                })?;
            let depenses = self
                .emplacements_depenses
                .get(&(pj, niveau_emplacement))
                .copied()
                .unwrap_or(0);
            e.restants.saturating_sub(depenses)
        };
        let dd = inc
            .emplacements
            .iter()
            .find(|e| e.niveau_de_sort == niveau_effectif)
            .map_or(10 + niveau_effectif as i32 + inc.mod_incantation, |e| e.dd);
        let accelere = metas.iter().any(|m| m.rapide);
        let type_action = type_d_action(
            sort.temps_d_incantation_en.as_deref(),
            inc.spontane,
            !metas.is_empty(),
            accelere,
        )
        .map_err(ErreurCombat::Invalide)?;
        let (mut verbal, mut gestuel) = composantes(sort.composantes_en.as_deref());
        if metas.iter().any(|m| m.sans_verbal) {
            verbal = false;
        }
        if metas.iter().any(|m| m.sans_gestes) {
            gestuel = false;
        }
        let echec_pct = if inc.arcanique && gestuel {
            inc.echec_arcanique_pct
        } else {
            0
        };
        Ok(Preparation {
            sort,
            niveau_du_sort,
            niveau_effectif,
            niveau_emplacement,
            metas,
            restants,
            dd,
            type_action,
            verbal,
            gestuel,
            echec_pct,
            inc,
        })
    }

    /// Ce que le joueur doit savoir et lancer avant de confirmer le lancement d'un sort.
    pub fn apercu_sort(
        &self,
        pj: IdPj,
        sort: &str,
        metamagies: &[String],
        niveau_heighten: Option<u32>,
        ctx: &dyn Fn(IdPj) -> Option<ContextePj>,
    ) -> Result<ApercuSort, ErreurCombat> {
        let p = self.preparer_sort(pj, sort, metamagies, niveau_heighten, ctx)?;
        let effet = effet_du_sort(&p.sort.nom_en);
        let cl = p.inc.niveau_de_lanceur;
        let mut suggeree = p
            .sort
            .duree_en
            .as_deref()
            .and_then(|t| duree_suggeree(t, cl));
        if p.metas.iter().any(|m| m.double_duree) {
            suggeree = suggeree.map(doubler);
        }
        Ok(ApercuSort {
            nom_en: p.sort.nom_en.clone(),
            niveau_du_sort: p.niveau_du_sort,
            niveau_effectif: p.niveau_effectif,
            niveau_emplacement: p.niveau_emplacement,
            restants: if p.niveau_emplacement == 0 {
                u32::MAX
            } else {
                p.restants
            },
            dd: p.dd,
            niveau_de_lanceur: cl,
            type_action: p.type_action,
            verbal: p.verbal,
            gestuel: p.gestuel,
            echec_arcanique_pct: p.echec_pct,
            dd_lancement_defensif: 15 + 2 * p.niveau_effectif as i32,
            bonus_concentration: cl as i32 + p.inc.mod_incantation + p.inc.bonus_concentration,
            duree_en: p.sort.duree_en.clone(),
            duree_suggeree: suggeree,
            portee_en: p.sort.portee_en.clone(),
            jet_de_sauvegarde_en: p.sort.jet_de_sauvegarde_en.clone(),
            description_en: p.sort.description_en.clone(),
            effet_automatique: effet.is_some(),
            choix_energie: effet.is_some_and(|e| e.choix_energie),
            notes: effet.and_then(|e| e.notes.clone()),
        })
    }

    pub(crate) fn lancer(
        &mut self,
        a: &ParamsLancer,
        ctx: &dyn Fn(IdPj) -> Option<ContextePj>,
    ) -> Result<(), ErreurCombat> {
        if self.phase != Phase::EnCours {
            return invalide("le combat n'est pas commencé");
        }
        self.verifier_engage(a.pj)?;
        let p = self.preparer_sort(a.pj, &a.sort, &a.metamagies, a.niveau_heighten, ctx)?;
        let c = ctx(a.pj).ok_or(ErreurAventure::PjInconnu(a.pj))?;
        if p.restants == 0 {
            return invalide(format!(
                "plus d'emplacement de sort de niveau {} pour {}",
                p.niveau_emplacement, c.nom
            ));
        }
        let cl = p.inc.niveau_de_lanceur;
        let effet = effet_du_sort(&p.sort.nom_en);
        let accepte_cibles = effet.is_some() && !a.cibles.is_empty();
        for cible in &a.cibles {
            self.verifier_engage(*cible)?;
        }
        // durée de l'effet : saisie, sinon proposée d'après le livre
        let duree = if accepte_cibles {
            let mut d = a.duree.clone().or_else(|| {
                let s = p
                    .sort
                    .duree_en
                    .as_deref()
                    .and_then(|t| duree_suggeree(t, cl));
                if p.metas.iter().any(|m| m.double_duree) {
                    s.map(doubler)
                } else {
                    s
                }
            });
            if d == Some(Duree::Instantanee) {
                d = None;
            }
            Some(d.ok_or_else(|| {
                ErreurCombat::Invalide(format!(
                    "durée à saisir pour {} (« {} » n'est pas une formule simple)",
                    p.sort.nom_en,
                    p.sort.duree_en.clone().unwrap_or_default()
                ))
            })?)
        } else {
            None
        };
        let energie = match (effet.is_some_and(|e| e.choix_energie), &a.choix_energie) {
            (true, Some(e)) if TYPES_D_ENERGIE.contains(&e.to_lowercase().as_str()) => {
                Some(e.to_lowercase())
            }
            (true, _) if accepte_cibles => {
                return invalide(
                    "choisir le type d'énergie : acide, froid, électricité, feu ou son",
                );
            }
            _ => None,
        };
        // contrôles saisis (avant de dépenser quoi que ce soit : une erreur ne change rien)
        if p.echec_pct > 0 && a.echec_arcanique_d100.is_none() {
            return invalide(format!(
                "échec des sorts profanes : saisir le d100 (échec à {} % ou moins)",
                p.echec_pct
            ));
        }
        if let Some(d) = a.defensif_d20 {
            if !(1..=20).contains(&d) {
                return invalide("d20 : de 1 à 20");
            }
        }
        if let Some(d) = a.echec_arcanique_d100 {
            if !(1..=100).contains(&d) {
                return invalide("d100 : de 1 à 100");
            }
        }
        // économie d'actions du tour
        self.appliquer_interne(
            &ActionCombat::Agir {
                pj: a.pj,
                type_action: p.type_action,
                libelle: format!("Lancer {}", p.sort.nom_en),
                deplacement: false,
            },
            ctx,
        )?;
        // le sort est dépensé même s'il échoue
        if p.niveau_emplacement > 0 {
            *self
                .emplacements_depenses
                .entry((a.pj, p.niveau_emplacement))
                .or_default() += 1;
            self.fiche.push((
                a.pj,
                fiches::journal::EvenementFiche::SortLance {
                    classe: p.inc.classe.clone(),
                    niveau_de_sort: p.niveau_emplacement,
                    sort: p.sort.nom_en.clone(),
                },
            ));
        }
        self.stats.entry(a.pj).or_default().sorts_lances += 1;
        let metas_txt = if p.metas.is_empty() {
            String::new()
        } else {
            format!(
                " ({})",
                p.metas
                    .iter()
                    .map(|m| m.nom_en.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        };
        self.noter(format!(
            "{} lance {}{metas_txt} : emplacement de niveau {}, DD {}, niveau de lanceur {cl}.",
            c.nom, p.sort.nom_en, p.niveau_emplacement, p.dd
        ));
        if let Some(r) = a.resultat_de {
            self.noter(format!("{} : résultat du dé du sort : {r}.", c.nom));
        }
        // lancement défensif
        if let Some(d20) = a.defensif_d20 {
            let dd = 15 + 2 * p.niveau_effectif as i32;
            let bonus = cl as i32 + p.inc.mod_incantation + p.inc.bonus_concentration;
            let total = d20 + bonus;
            if total < dd {
                self.noter(format!(
                    "{} : lancement défensif raté ({d20} + {bonus} = {total} contre DD {dd}) : le sort est perdu.",
                    c.nom
                ));
                return Ok(());
            }
            self.noter(format!(
                "{} : lancement défensif réussi ({d20} + {bonus} = {total} contre DD {dd}).",
                c.nom
            ));
        }
        // échec des sorts profanes
        if let Some(d100) = a.echec_arcanique_d100 {
            if p.echec_pct > 0 {
                if d100 as u32 <= p.echec_pct {
                    self.noter(format!(
                        "{} : échec des sorts profanes ({d100} sur {} %) : le sort est perdu.",
                        c.nom, p.echec_pct
                    ));
                    return Ok(());
                }
                self.noter(format!(
                    "{} : échec des sorts profanes évité ({d100} sur {} %).",
                    c.nom, p.echec_pct
                ));
            }
        }
        // effets sur les personnages joués
        match (effet, duree) {
            (Some(e), Some(duree)) => {
                let effets: Vec<Effet> = e
                    .effets
                    .iter()
                    .map(|d| Effet::new(&d.cible, d.type_bonus, d.valeur, &p.sort.nom_en))
                    .collect();
                let mut protections = Vec::new();
                for pr in &e.protections {
                    protections.push(match pr {
                        ProtectionData::Reduction { valeur } => {
                            Protection::Reduction { valeur: *valeur }
                        }
                        ProtectionData::Immunite { type_degats } => Protection::Immunite {
                            type_degats: type_degats.clone(),
                        },
                        ProtectionData::Resistance { paliers } => {
                            let valeur = paliers
                                .iter()
                                .filter(|(n, _)| *n <= cl)
                                .map(|(_, v)| *v)
                                .max()
                                .unwrap_or(0);
                            Protection::Resistance {
                                type_degats: energie.clone().unwrap_or_default(),
                                valeur,
                            }
                        }
                    });
                }
                for cible in &a.cibles {
                    let ev = EvenementAventure::EffetAjoute {
                        pj: *cible,
                        nom: p.sort.nom_en.clone(),
                        source: format!("sort {}, lancé par {}", p.sort.nom_en, c.nom),
                        duree: duree.clone(),
                        effets: effets.clone(),
                        etats: e.etats.clone(),
                        soin_par_round: 0,
                        protections: protections.clone(),
                    };
                    let s = self.aventure.appliquer(&ev, ctx)?;
                    self.fusionner(s);
                }
                if let Some(n) = &e.notes {
                    self.noter(format!("{} : {n}", p.sort.nom_en));
                }
            }
            (None, _) if !a.cibles.is_empty() => {
                self.noter(format!(
                    "{} n'a pas d'effet automatique : l'appliquer à la main (« appliquer un effet »).",
                    p.sort.nom_en
                ));
            }
            _ => {}
        }
        Ok(())
    }

    /// Test de concentration (Core p. 206) : pour maintenir un sort, ou parce qu'on est distrait.
    /// Un échec rompt l'effet à concentration visé (`effet`).
    pub(crate) fn concentration(
        &mut self,
        pj: IdPj,
        d20: i32,
        cause: &Distraction,
        niveau_du_sort: u32,
        effet: Option<IdEffet>,
        ctx: &dyn Fn(IdPj) -> Option<ContextePj>,
    ) -> Result<(), ErreurCombat> {
        self.verifier_engage(pj)?;
        if !(1..=20).contains(&d20) {
            return invalide("d20 : de 1 à 20");
        }
        let c = ctx(pj).ok_or(ErreurAventure::PjInconnu(pj))?;
        let Some(inc) = c.incantation else {
            return invalide(format!("{} ne lance pas de sorts", c.nom));
        };
        let bonus = inc.niveau_de_lanceur as i32 + inc.mod_incantation + inc.bonus_concentration;
        let dd = cause.dd(niveau_du_sort);
        let total = d20 + bonus;
        let reussi = total >= dd;
        self.noter(format!(
            "{} : test de concentration {d20} + {bonus} = {total} contre DD {dd} : {}.",
            c.nom,
            if reussi {
                "réussi"
            } else {
                "raté, le sort est perdu"
            }
        ));
        if !reussi {
            if let Some(id) = effet {
                let s = self
                    .aventure
                    .appliquer(&EvenementAventure::EffetRetire { id }, ctx)?;
                self.fusionner(s);
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn durees_du_livre() {
        assert_eq!(
            duree_suggeree("1 min./level (D)", 5),
            Some(Duree::Minutes(5))
        );
        assert_eq!(duree_suggeree("10 min./level", 5), Some(Duree::Minutes(50)));
        assert_eq!(
            duree_suggeree("1 hour/level (D)", 3),
            Some(Duree::Heures(3))
        );
        assert_eq!(duree_suggeree("1 round/level", 7), Some(Duree::Rounds(7)));
        assert_eq!(duree_suggeree("1 round", 7), Some(Duree::Rounds(1)));
        assert_eq!(duree_suggeree("1 minute", 7), Some(Duree::Minutes(1)));
        assert_eq!(duree_suggeree("24 hours", 7), Some(Duree::Heures(24)));
        assert_eq!(duree_suggeree("2 days/level", 3), Some(Duree::Jours(6)));
        assert_eq!(duree_suggeree("instantaneous", 3), Some(Duree::Instantanee));
        assert_eq!(duree_suggeree("permanent", 3), Some(Duree::Permanente));
        assert_eq!(
            duree_suggeree("concentration", 3),
            Some(Duree::Concentration)
        );
        assert_eq!(duree_suggeree("see text", 3), None);
        assert_eq!(duree_suggeree("1d4 rounds", 3), None);
    }

    #[test]
    fn composantes_du_livre() {
        assert_eq!(
            composantes(Some("V, S, M (a ball of bat guano and sulfur)")),
            (true, true)
        );
        assert_eq!(composantes(Some("V")), (true, false));
        assert_eq!(composantes(Some("S, M/DF (x, y)")), (false, true));
        assert_eq!(
            composantes(Some("V, S, F (a piece of cured leather)")),
            (true, true)
        );
    }

    #[test]
    fn temps_d_incantation() {
        let t = |s, sp, m, q| type_d_action(Some(s), sp, m, q);
        assert_eq!(
            t("1 standard action", true, false, false),
            Ok(TypeAction::Simple)
        );
        assert_eq!(
            t("1 standard action", true, true, false),
            Ok(TypeAction::Complexe)
        );
        assert_eq!(
            t("1 standard action", true, true, true),
            Ok(TypeAction::Rapide)
        );
        assert_eq!(
            t("1 full-round action", true, false, false),
            Ok(TypeAction::Complexe)
        );
        assert!(t("1 full-round action", true, true, false).is_err());
        assert_eq!(t("1 round", true, false, false), Ok(TypeAction::Complexe));
        assert!(t("1 minute", true, false, false).is_err());
        assert_eq!(
            t("1 swift action", true, true, false),
            Ok(TypeAction::Rapide)
        );
    }

    #[test]
    fn distractions() {
        assert_eq!(Distraction::Blessure { degats: 7 }.dd(3), 20);
        assert_eq!(Distraction::DegatsContinus { derniers_degats: 5 }.dd(3), 15);
        assert_eq!(Distraction::Agrippe { bmc: 8 }.dd(2), 20);
        assert_eq!(Distraction::MouvementViolent.dd(2), 17);
        assert_eq!(Distraction::Entrave.dd(2), 17);
        assert_eq!(Distraction::Intemperies.dd(1), 6);
        assert_eq!(Distraction::SortSurSoi { dd_du_sort: 15 }.dd(2), 17);
    }

    #[test]
    fn les_donnees_sont_coherentes_avec_le_catalogue() {
        let r = referentiel();
        for e in effets_des_sorts() {
            let s = r
                .sort(&e.nom_en)
                .unwrap_or_else(|| panic!("sort absent : {}", e.nom_en));
            assert_eq!(s.source.page, Some(e.page), "page de {}", e.nom_en);
        }
        for m in metamagies() {
            let d = r
                .don(&m.nom_en)
                .unwrap_or_else(|| panic!("don absent : {}", m.nom_en));
            assert_eq!(d.source.page, Some(m.page), "page de {}", m.nom_en);
            // le niveau d'emplacement annoncé est dans le texte du livre
            let t = d.avantage_en.clone().unwrap_or_default();
            if let Some(delta) = m.delta {
                let mot = ["", "one", "two", "three", "four"][delta as usize];
                assert!(
                    t.contains(&format!("{mot} level")),
                    "{} : « {mot} level(s) higher » absent du texte",
                    m.nom_en
                );
            }
        }
    }
}
