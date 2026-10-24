//! Modèles d'affichage du mode aventure et du mode combat : tout ce que l'interface montre est
//! calculé ici à partir de l'état de la campagne ; l'interface ne refait aucun calcul de règles.

use aventure::{
    Besoin, ContextePj, EtatAventure, GenreObjet, IdPj, Sante, SuiviPj, UniteRessource,
};
use campagne::etat::Etat;
use campagne::modele::IdPersonnage;
use combat::{Combat, Demande, EtatCombat, Phase};
use fiches::traductions::Traductions;
use fiches::Referentiel;
use temps::{texte_rounds, Echeance, Instant};

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct HorlogeVue {
    pub jour: i64,
    pub heure: u32,
    pub minute: u32,
    /// « jour 3, 14:05 »
    pub texte: String,
    pub en_combat: bool,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct EffetVue {
    pub id: u64,
    pub nom: String,
    pub source: String,
    pub duree: String,
    /// Temps qui reste (« 3 min 20 s »), si l'effet a une échéance.
    pub restant: Option<String>,
    /// Part de la durée qui reste (0 à 1), pour le chronomètre circulaire.
    pub fraction: Option<f64>,
    /// Bonus, malus et protections, un par ligne.
    pub details: Vec<String>,
    pub etats: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct AlerteVue {
    pub id: u64,
    pub nom: String,
    pub personnage: String,
    pub source: String,
    /// « a pris fin » ou « dissipé à la fin du combat ».
    pub raison: String,
    pub quand: String,
    /// L'effet pouvait être dissipé par la conversion de fin de combat : on peut le maintenir.
    pub dissipe: bool,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct EmplacementVue {
    pub niveau: u32,
    pub restants: u32,
    pub total: u32,
    pub dd: i32,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct RessourceVue {
    /// Identifiant (nom anglais du pouvoir) à renvoyer au moteur.
    pub cle: String,
    pub nom: String,
    pub unite: String,
    pub restantes: u32,
    pub max: u32,
    pub source: String,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct ObjetVue {
    pub id: u64,
    pub nom: String,
    pub genre: String,
    pub genre_code: String,
    pub sort: Option<String>,
    pub quantite: u32,
    pub charges: Option<(u32, u32)>,
    pub par_jour: Option<(u32, u32)>,
    pub niveau_de_lanceur: u32,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct SortConnuVue {
    pub nom_en: String,
    pub nom: String,
    pub niveau: u32,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct EtatVue {
    pub id: String,
    pub nom: String,
}

/// Un personnage joué dans l'aventure ou le combat.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct PjAventure {
    pub id: u32,
    pub nom: String,
    pub niveau: u32,
    pub pv: i32,
    pub pv_max: i32,
    pub temporaires: i32,
    pub non_letaux: i32,
    pub sante: String,
    pub sante_code: String,
    pub etats: Vec<EtatVue>,
    pub effets: Vec<EffetVue>,
    pub emplacements: Vec<EmplacementVue>,
    pub sorts_connus: Vec<SortConnuVue>,
    pub ressources: Vec<RessourceVue>,
    pub inventaire: Vec<ObjetVue>,
    pub degats_continus: bool,
    /// Initiative de la fiche (modificateur).
    pub initiative: i32,
    pub caster: bool,
    /// Niveau de lanceur (0 si le personnage ne lance pas de sorts).
    pub niveau_de_lanceur: u32,
    /// Chance d'échec des sorts profanes due à l'armure (%).
    pub echec_arcanique_pct: u32,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct DemandeVue {
    pub pj: u32,
    pub nom: String,
    /// `stabilisation` | `massifs` | `soif` | `faim`
    pub genre: String,
    pub texte: String,
    /// DD du jet, s'il est connu.
    pub dd: Option<i32>,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct LigneJournal {
    pub quand: String,
    pub texte: String,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct AventureVue {
    pub actif: bool,
    pub horloge: Option<HorlogeVue>,
    pub survie_active: bool,
    pub degats_massifs: bool,
    pub personnages: Vec<PjAventure>,
    /// Personnages avec une fiche qui n'ont pas encore rejoint l'aventure.
    pub disponibles: Vec<(u32, String)>,
    pub alertes: Vec<AlerteVue>,
    pub demandes: Vec<DemandeVue>,
    pub journal: Vec<LigneJournal>,
}

fn horloge(i: Instant, en_combat: bool) -> HorlogeVue {
    HorlogeVue {
        jour: i.jour(),
        heure: i.heure(),
        minute: i.minute(),
        texte: format!("jour {}, {:02}:{:02}", i.jour(), i.heure(), i.minute()),
        en_combat,
    }
}

fn texte_instant(i: Instant) -> String {
    format!(
        "jour {}, {:02}:{:02}:{:02}",
        i.jour(),
        i.heure(),
        i.minute(),
        i.seconde()
    )
}

fn code_sante(s: Sante) -> &'static str {
    match s {
        Sante::Conscient => "conscient",
        Sante::Chancelant => "chancelant",
        Sante::HorsDeCombat => "hors_de_combat",
        Sante::Inconscient => "inconscient",
        Sante::Mourant => "mourant",
        Sante::Stable => "stable",
        Sante::Mort => "mort",
    }
}

fn code_genre(g: GenreObjet) -> &'static str {
    match g {
        GenreObjet::Potion => "potion",
        GenreObjet::Parchemin => "parchemin",
        GenreObjet::Baguette => "baguette",
        GenreObjet::Baton => "baton",
        GenreObjet::Charges => "charges",
        GenreObjet::Munition => "munition",
        GenreObjet::Autre => "autre",
    }
}

fn nom_etat(id: &str) -> String {
    Referentiel::partage()
        .etat(id)
        .map_or_else(|| id.to_string(), |e| e.nom.clone())
}

fn nom_sort_fr(nom_en: &str) -> String {
    Traductions::partage()
        .sort(nom_en)
        .map_or_else(|| nom_en.to_string(), |s| s.nom.clone())
}

/// Nom français d'un pouvoir de lignage (la source indique le lignage : « lignage aberrant, … »).
fn nom_pouvoir_fr(source: &str, nom_en: &str) -> String {
    source
        .strip_prefix("lignage ")
        .and_then(|s| s.split(',').next())
        .and_then(|id| Traductions::partage().lignage(id))
        .and_then(|l| l.pouvoir(nom_en).map(|(n, _)| n.to_string()))
        .unwrap_or_else(|| nom_en.to_string())
}

fn effets_du_pj(a: &EtatAventure, id: u32, maintenant: Instant) -> Vec<EffetVue> {
    let proprio = id.to_string();
    a.effets
        .actifs
        .iter()
        .filter(|e| e.proprietaire == proprio)
        .map(|e| {
            let restant = e.restant(maintenant);
            let total = match e.echeance {
                Echeance::A(fin) => Some(fin.ecart(e.debut)),
                Echeance::Indeterminee => None,
            };
            let mut details: Vec<String> = e
                .contenu
                .effets
                .iter()
                .map(|x| {
                    format!(
                        "{:+} {} ({})",
                        x.valeur,
                        x.type_bonus.nom(),
                        libelle_cible(&x.cible.0)
                    )
                })
                .collect();
            for p in &e.contenu.protections {
                details.push(match p {
                    aventure::Protection::Immunite { type_degats } => {
                        format!("immunité : {type_degats}")
                    }
                    aventure::Protection::Resistance {
                        type_degats,
                        valeur,
                    } => {
                        format!("résistance {valeur} : {type_degats}")
                    }
                    aventure::Protection::Reduction { valeur } => format!("RD {valeur}"),
                });
            }
            if e.contenu.soin_par_round > 0 {
                details.push(format!("{} PV rendus par round", e.contenu.soin_par_round));
            }
            EffetVue {
                id: e.id,
                nom: e.nom.clone(),
                source: e.source.clone(),
                duree: e.duree.texte(),
                restant: restant.map(texte_rounds),
                fraction: match (restant, total) {
                    (Some(r), Some(t)) if t > 0 => Some((r as f64 / t as f64).clamp(0.0, 1.0)),
                    _ => None,
                },
                details,
                etats: e.contenu.etats.iter().map(|s| nom_etat(s)).collect(),
            }
        })
        .collect()
}

fn libelle_cible(c: &str) -> String {
    match c {
        "ca" => "CA".into(),
        "attaque" => "attaques".into(),
        "init" => "initiative".into(),
        "bmo" => "BMO".into(),
        "dmd" => "DMD".into(),
        "degats" => "dégâts".into(),
        "jds.vig" => "Vigueur".into(),
        "jds.ref" => "Réflexes".into(),
        "jds.vol" => "Volonté".into(),
        "comp.*" => "compétences".into(),
        c if c.starts_with("carac.") => c[6..].to_uppercase(),
        autre => autre.to_string(),
    }
}

/// Un personnage tel que l'interface l'affiche. `depenses` : emplacements déjà dépensés dans le
/// combat en cours.
fn pj_vue(
    id: u32,
    suivi: &SuiviPj,
    c: &ContextePj,
    a: &EtatAventure,
    depenses: &dyn Fn(u32) -> u32,
) -> PjAventure {
    let sante = suivi.vital.sante(c.pv_max, c.con_score);
    let maintenant = a.horloge.maintenant;
    let (emplacements, sorts_connus) = match &c.incantation {
        Some(inc) => {
            let r = Referentiel::partage();
            let emplacements = inc
                .emplacements
                .iter()
                .map(|e| EmplacementVue {
                    niveau: e.niveau_de_sort,
                    restants: e.restants.saturating_sub(depenses(e.niveau_de_sort)),
                    total: e.total,
                    dd: e.dd,
                })
                .collect();
            let mut sorts: Vec<SortConnuVue> = inc
                .sorts_connus
                .iter()
                .filter_map(|n| {
                    let s = r.sort_approx(n)?;
                    Some(SortConnuVue {
                        nom_en: s.nom_en.clone(),
                        nom: nom_sort_fr(&s.nom_en),
                        niveau: s.niveau_pour(&inc.classe_en)?,
                    })
                })
                .collect();
            sorts.sort_by(|x, y| x.niveau.cmp(&y.niveau).then(x.nom.cmp(&y.nom)));
            (emplacements, sorts)
        }
        None => (Vec::new(), Vec::new()),
    };
    PjAventure {
        id,
        nom: c.nom.clone(),
        niveau: c.niveau,
        pv: suivi.vital.pv_actuels(c.pv_max),
        pv_max: c.pv_max,
        temporaires: suivi.vital.temporaires,
        non_letaux: suivi.vital.non_letaux,
        sante: sante.nom().to_string(),
        sante_code: code_sante(sante).to_string(),
        etats: suivi
            .etats_fiche
            .iter()
            .map(|e| EtatVue {
                id: e.clone(),
                nom: nom_etat(e),
            })
            .collect(),
        effets: effets_du_pj(a, id, maintenant),
        emplacements,
        sorts_connus,
        ressources: suivi
            .ressources
            .iter()
            .map(|(nom, r)| RessourceVue {
                cle: nom.clone(),
                nom: nom_pouvoir_fr(&r.source, nom),
                unite: match r.unite {
                    UniteRessource::Utilisations => "utilisations",
                    UniteRessource::Rounds => "rounds",
                    UniteRessource::Minutes => "minutes",
                }
                .to_string(),
                restantes: r.restantes(),
                max: r.max,
                source: r.source.clone(),
            })
            .collect(),
        inventaire: suivi
            .inventaire
            .iter()
            .map(|(id, o)| ObjetVue {
                id: *id,
                nom: o.nom.clone(),
                genre: o.genre.nom().to_string(),
                genre_code: code_genre(o.genre).to_string(),
                sort: o.sort.as_deref().map(nom_sort_fr),
                quantite: o.quantite,
                charges: o.charges.map(|c| (c.restantes, c.max)),
                par_jour: o.par_jour.map(|c| (c.restantes, c.max)),
                niveau_de_lanceur: o.niveau_de_lanceur,
            })
            .collect(),
        degats_continus: suivi.vital.continu,
        initiative: c.initiative,
        caster: c.incantation.is_some(),
        niveau_de_lanceur: c.incantation.as_ref().map_or(0, |i| i.niveau_de_lanceur),
        echec_arcanique_pct: c.incantation.as_ref().map_or(0, |i| i.echec_arcanique_pct),
    }
}

fn alertes(a: &EtatAventure, nom_de: &dyn Fn(u32) -> String) -> Vec<AlerteVue> {
    a.effets
        .alertes()
        .into_iter()
        .map(|t| AlerteVue {
            id: t.effet.id,
            nom: t.effet.nom.clone(),
            personnage: nom_de(t.effet.proprietaire.parse().unwrap_or(0)),
            source: t.effet.source.clone(),
            raison: match t.raison {
                temps::RaisonFin::Echeance => "a pris fin".into(),
                temps::RaisonFin::Retire => "a été retiré".into(),
                temps::RaisonFin::Dissipe => "dissipé à la fin du combat".into(),
            },
            quand: texte_instant(t.a),
            dissipe: t.raison == temps::RaisonFin::Dissipe,
        })
        .collect()
}

fn demandes_survie(
    a: &EtatAventure,
    contextes: &std::collections::BTreeMap<IdPj, ContextePj>,
) -> Vec<DemandeVue> {
    let ctx = |id: IdPj| contextes.get(&id).cloned();
    let mut v: Vec<DemandeVue> = a
        .tests_survie_dus(&ctx)
        .into_iter()
        .map(|(pj, t)| DemandeVue {
            pj,
            nom: contextes
                .get(&pj)
                .map(|c| c.nom.clone())
                .unwrap_or_default(),
            genre: match t.besoin {
                Besoin::Eau => "soif",
                Besoin::Nourriture => "faim",
            }
            .to_string(),
            texte: format!("Test de Constitution ({})", t.besoin.nom()),
            dd: Some(t.dd),
        })
        .collect();
    for pj in a.massifs_demandes() {
        v.push(DemandeVue {
            pj,
            nom: contextes
                .get(&pj)
                .map(|c| c.nom.clone())
                .unwrap_or_default(),
            genre: "massifs".into(),
            texte: "Dégâts massifs : jet de Vigueur".into(),
            dd: Some(15),
        });
    }
    v
}

/// Vue du mode aventure.
pub fn aventure(etat: &Etat) -> AventureVue {
    let Some(a) = &etat.aventure else {
        return AventureVue {
            actif: false,
            horloge: None,
            survie_active: false,
            degats_massifs: false,
            personnages: Vec::new(),
            disponibles: etat
                .personnages
                .iter()
                .filter(|(id, _)| etat.contexte_aventure(**id).is_some())
                .map(|(id, p)| (id.0, p.nom.clone()))
                .collect(),
            alertes: Vec::new(),
            demandes: Vec::new(),
            journal: Vec::new(),
        };
    };
    let contextes = etat.contextes_aventure();
    let personnages: Vec<PjAventure> = a
        .pjs
        .iter()
        .filter_map(|(id, s)| contextes.get(id).map(|c| pj_vue(*id, s, c, a, &|_| 0)))
        .collect();
    let nom_de = |id: u32| {
        contextes
            .get(&id)
            .map_or_else(|| format!("PJ {id}"), |c| c.nom.clone())
    };
    AventureVue {
        actif: true,
        horloge: Some(horloge(a.horloge.maintenant, a.horloge.en_combat())),
        survie_active: a.survie_active,
        degats_massifs: a.degats_massifs,
        disponibles: etat
            .personnages
            .iter()
            .filter(|(id, _)| !a.pjs.contains_key(&id.0) && contextes.contains_key(&id.0))
            .map(|(id, p)| (id.0, p.nom.clone()))
            .collect(),
        personnages,
        alertes: alertes(a, &nom_de),
        demandes: demandes_survie(a, &contextes),
        journal: a
            .journal
            .iter()
            .rev()
            .take(60)
            .map(|e| LigneJournal {
                quand: texte_instant(e.instant),
                texte: e.texte.clone(),
            })
            .collect(),
    }
}

// ------------------------------------------------------------------ combat

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct InitiativeVue {
    pub id: u32,
    pub nom: String,
    pub total: i32,
    pub d20: i32,
    pub modificateur: i32,
    pub departage: Option<i32>,
    pub actif: bool,
    /// Joue dans le round courant (tous, sauf au round de surprise).
    pub joue: bool,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct EconomieVue {
    pub simple: bool,
    pub mouvement: bool,
    pub complexe: bool,
    pub rapide: bool,
    pub pas: bool,
    /// Une seule action simple ou de mouvement (surprise, chancelant…).
    pub limite: bool,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct PjCombat {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub pj: PjAventure,
    pub economie: EconomieVue,
    pub degats_subis: i32,
    pub soins_recus: i32,
    pub sorts_lances: u32,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct CombatVue {
    /// `initiative` (on saisit les initiatives) ou `enCours`.
    pub phase: String,
    pub round: u32,
    pub surprise: bool,
    pub actif: Option<u32>,
    pub initiatives: Vec<InitiativeVue>,
    /// Engagés qui n'ont pas encore leur initiative.
    pub sans_initiative: Vec<(u32, String)>,
    pub egalites: Vec<Vec<String>>,
    pub personnages: Vec<PjCombat>,
    pub demandes: Vec<DemandeVue>,
    pub alertes: Vec<AlerteVue>,
    pub journal: Vec<(u32, String)>,
    pub peut_annuler: bool,
    pub peut_retablir: bool,
    pub horloge: HorlogeVue,
    pub rounds_joues: u32,
}

/// Vue du combat en cours, calculée à partir du combat et des fiches de la campagne.
pub fn combat(etat: &Etat, c: &Combat) -> CombatVue {
    let e: &EtatCombat = c.etat();
    let contextes = etat.contextes_aventure();
    let ctx = |id: IdPj| contextes.get(&id).cloned();
    let nom_de = |id: u32| {
        contextes
            .get(&id)
            .map_or_else(|| format!("PJ {id}"), |c| c.nom.clone())
    };
    let ordre_round = e.ordre_du_round();
    let initiatives: Vec<InitiativeVue> = e
        .ordre
        .iter()
        .filter_map(|id| {
            let i = e.initiatives.get(id)?;
            Some(InitiativeVue {
                id: *id,
                nom: nom_de(*id),
                total: i.total,
                d20: i.d20,
                modificateur: i.modificateur,
                departage: i.departage,
                actif: e.actif == Some(*id),
                joue: ordre_round.contains(id),
            })
        })
        .collect();
    let personnages = e
        .engages
        .iter()
        .filter_map(|id| {
            let s = e.aventure.pjs.get(id)?;
            let cx = contextes.get(id)?;
            let eco = e.economie.get(id).copied().unwrap_or_default();
            let limite = e.limite(*id, &ctx);
            let r = eco.reste(limite);
            let stats = e.stats.get(id).cloned().unwrap_or_default();
            let pj = pj_vue(*id, s, cx, &e.aventure, &|n| {
                e.emplacements_depenses.get(&(*id, n)).copied().unwrap_or(0)
            });
            Some(PjCombat {
                pj,
                economie: EconomieVue {
                    simple: r.simple,
                    mouvement: r.mouvement,
                    complexe: r.complexe,
                    rapide: r.rapide,
                    pas: r.pas,
                    limite,
                },
                degats_subis: stats.degats_subis,
                soins_recus: stats.soins_recus,
                sorts_lances: stats.sorts_lances,
            })
        })
        .collect();
    let mut demandes: Vec<DemandeVue> = e
        .demandes(&ctx)
        .into_iter()
        .map(|d| match d {
            Demande::Stabilisation { pj, malus } => DemandeVue {
                pj,
                nom: nom_de(pj),
                genre: "stabilisation".into(),
                texte: format!("Jet de stabilisation : d20 + Constitution {malus:+}"),
                dd: Some(10),
            },
            Demande::DegatsMassifs { pj } => DemandeVue {
                pj,
                nom: nom_de(pj),
                genre: "massifs".into(),
                texte: "Dégâts massifs : jet de Vigueur".into(),
                dd: Some(15),
            },
        })
        .collect();
    demandes.sort_by_key(|d| d.pj);
    CombatVue {
        phase: match e.phase {
            Phase::Initiative => "initiative".into(),
            Phase::EnCours => "enCours".into(),
        },
        round: e.round,
        surprise: e.surprise.is_some() && e.round == 0,
        actif: e.actif,
        initiatives,
        sans_initiative: e
            .engages
            .iter()
            .filter(|id| !e.initiatives.contains_key(id))
            .map(|id| (*id, nom_de(*id)))
            .collect(),
        egalites: e
            .egalites()
            .into_iter()
            .map(|g| g.into_iter().map(&nom_de).collect())
            .collect(),
        personnages,
        demandes,
        alertes: alertes(&e.aventure, &nom_de),
        journal: e
            .journal
            .iter()
            .rev()
            .take(60)
            .map(|j| (j.round, j.texte.clone()))
            .collect(),
        peut_annuler: c.peut_annuler(),
        peut_retablir: c.peut_retablir(),
        horloge: horloge(e.aventure.horloge.maintenant, true),
        rounds_joues: e.rounds_joues,
    }
}

/// Personnages de la campagne (avec fiche) que l'on peut engager dans un combat.
pub fn personnages_de_l_aventure(etat: &Etat) -> Vec<(IdPersonnage, String)> {
    let Some(a) = &etat.aventure else {
        return Vec::new();
    };
    a.pjs
        .keys()
        .filter_map(|id| {
            etat.personnages
                .get(&IdPersonnage(*id))
                .map(|p| (IdPersonnage(*id), p.nom.clone()))
        })
        .collect()
}

// ------------------------------------------------- aperçus (avant de confirmer)

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct ApercuSortVue {
    pub nom_en: String,
    pub nom: String,
    pub niveau_du_sort: u32,
    pub niveau_effectif: u32,
    pub niveau_emplacement: u32,
    /// `None` pour un tour de magie (à volonté).
    pub restants: Option<u32>,
    pub dd: i32,
    pub niveau_de_lanceur: u32,
    pub type_action: String,
    pub verbal: bool,
    pub gestuel: bool,
    pub echec_arcanique_pct: u32,
    pub dd_lancement_defensif: i32,
    pub bonus_concentration: i32,
    pub duree_texte: Option<String>,
    /// Durée proposée d'après le livre ; le joueur peut la corriger.
    pub duree_suggeree: Option<temps::Duree>,
    pub duree_suggeree_texte: Option<String>,
    pub portee: Option<String>,
    pub jet_de_sauvegarde: Option<String>,
    pub description: String,
    pub effet_automatique: bool,
    pub choix_energie: bool,
    pub notes: Option<String>,
}

pub fn apercu_sort(a: &combat::ApercuSort) -> ApercuSortVue {
    let fr = Traductions::partage().sort(&a.nom_en);
    ApercuSortVue {
        nom_en: a.nom_en.clone(),
        nom: fr.map_or_else(|| a.nom_en.clone(), |s| s.nom.clone()),
        niveau_du_sort: a.niveau_du_sort,
        niveau_effectif: a.niveau_effectif,
        niveau_emplacement: a.niveau_emplacement,
        restants: (a.niveau_emplacement > 0).then_some(a.restants),
        dd: a.dd,
        niveau_de_lanceur: a.niveau_de_lanceur,
        type_action: a.type_action.nom().to_string(),
        verbal: a.verbal,
        gestuel: a.gestuel,
        echec_arcanique_pct: a.echec_arcanique_pct,
        dd_lancement_defensif: a.dd_lancement_defensif,
        bonus_concentration: a.bonus_concentration,
        duree_texte: fr
            .and_then(|s| s.duree.clone())
            .or_else(|| a.duree_en.clone()),
        duree_suggeree: a.duree_suggeree.clone(),
        duree_suggeree_texte: a.duree_suggeree.as_ref().map(|d| d.texte()),
        portee: fr
            .and_then(|s| s.portee.clone())
            .or_else(|| a.portee_en.clone()),
        jet_de_sauvegarde: fr
            .and_then(|s| s.jet_de_sauvegarde.clone())
            .or_else(|| a.jet_de_sauvegarde_en.clone()),
        description: fr.map_or_else(|| a.description_en.clone(), |s| s.description.clone()),
        effet_automatique: a.effet_automatique,
        choix_energie: a.choix_energie,
        notes: a.notes.clone(),
    }
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct BilanPjVue {
    pub id: u32,
    pub nom: String,
    pub pv_avant: i32,
    pub pv_apres: i32,
    pub degats_subis: i32,
    pub soins_recus: i32,
    pub sorts_lances: u32,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct DissipationVue {
    pub id: u64,
    pub nom: String,
    pub personnage: String,
    pub source: String,
    pub duree: String,
    /// Temps qui restait et qui est perdu.
    pub perdu: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct ArrondiVue {
    pub nom: String,
    pub personnage: String,
    pub avant: String,
    pub apres: String,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct ApercuFinVue {
    pub rounds_joues: u32,
    pub personnages: Vec<BilanPjVue>,
    pub dissipes: Vec<DissipationVue>,
    pub arrondis: Vec<ArrondiVue>,
    pub persistants: Vec<(String, String, Option<String>)>,
}

pub fn apercu_fin(etat: &Etat, f: &combat::ApercuFin) -> ApercuFinVue {
    let contextes = etat.contextes_aventure();
    let nom_de = |s: &str| {
        s.parse::<u32>()
            .ok()
            .and_then(|id| contextes.get(&id))
            .map_or_else(|| s.to_string(), |c| c.nom.clone())
    };
    ApercuFinVue {
        rounds_joues: f.rounds_joues,
        personnages: f
            .par_pj
            .iter()
            .map(|r| BilanPjVue {
                id: r.pj,
                nom: nom_de(&r.pj.to_string()),
                pv_avant: r.pv_avant,
                pv_apres: r.pv_apres,
                degats_subis: r.stats.degats_subis,
                soins_recus: r.stats.soins_recus,
                sorts_lances: r.stats.sorts_lances,
            })
            .collect(),
        dissipes: f
            .bilan
            .dissipes
            .iter()
            .map(|d| DissipationVue {
                id: d.id,
                nom: d.nom.clone(),
                personnage: nom_de(&d.proprietaire),
                source: d.source.clone(),
                duree: d.duree.texte(),
                perdu: d.perdu.map(texte_rounds),
            })
            .collect(),
        arrondis: f
            .bilan
            .arrondis
            .iter()
            .map(|a| ArrondiVue {
                nom: a.nom.clone(),
                personnage: nom_de(&a.proprietaire),
                avant: texte_rounds(a.avant),
                apres: texte_rounds(a.apres),
            })
            .collect(),
        persistants: f
            .persistants
            .iter()
            .map(|(_, nom, proprio, fin)| {
                (
                    nom.clone(),
                    nom_de(proprio),
                    match fin {
                        Echeance::A(i) => Some(texte_instant(*i)),
                        Echeance::Indeterminee => None,
                    },
                )
            })
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use campagne::evenements::Evenement;
    use campagne::Campagne;
    use combat::{ActionCombat, ParamsLancer};
    use domaine::Montant;
    use fiches::journal::EvenementFiche;

    fn campagne() -> (Campagne, regles::Referentiel, IdPersonnage) {
        let r = regles::referentiel_officiel();
        let mut c = Campagne::nouvelle("Test");
        let pj = c.nouvel_id_personnage();
        for ev in [
            Evenement::PersonnageCree {
                id: pj,
                nom: "Lyra".into(),
                compte_initial: Montant::po(10),
                prestige: None,
            },
            Evenement::Fiche {
                pj,
                evenement: EvenementFiche::Creation(Box::new(fiches::exemple::elfe_ensorceleuse())),
            },
        ] {
            c.executer(ev, &r).unwrap();
        }
        (c, r, pj)
    }

    #[test]
    fn vue_sans_aventure_propose_les_personnages_a_fiche() {
        let (mut c, r, pj) = campagne();
        let v = aventure(c.etat(&r));
        assert!(!v.actif);
        assert_eq!(v.disponibles, vec![(pj.0, "Lyra".to_string())]);
    }

    #[test]
    fn vue_de_l_aventure_et_du_combat() {
        let (mut c, r, pj) = campagne();
        for ev in [
            Evenement::AventureDemarree {
                heure: 9,
                minute: 30,
                survie: false,
                degats_massifs: false,
            },
            Evenement::Aventure(aventure::EvenementAventure::Entree { pj: pj.0 }),
        ] {
            c.executer(ev, &r).unwrap();
        }
        let etat = c.etat(&r).clone();
        let v = aventure(&etat);
        assert!(v.actif);
        assert_eq!(v.horloge.as_ref().unwrap().texte, "jour 1, 09:30");
        let l = &v.personnages[0];
        assert_eq!(
            (l.nom.as_str(), l.sante_code.as_str()),
            ("Lyra", "conscient")
        );
        assert!(l.pv > 0 && l.pv == l.pv_max);
        assert!(l.caster && !l.emplacements.is_empty());
        assert!(l
            .sorts_connus
            .iter()
            .any(|s| s.nom_en == "Mage Armor" && s.niveau == 1));
        assert!(v.disponibles.is_empty());

        // combat : initiative puis lancer une armure du mage sur soi
        let contextes = etat.contextes_aventure();
        let ctx = |id: aventure::IdPj| contextes.get(&id).cloned();
        let base = etat.aventure.clone().unwrap();
        let mut cb = Combat::nouveau(&base, vec![pj.0], &ctx).unwrap();
        let vc = combat(&etat, &cb);
        assert_eq!(vc.phase, "initiative");
        assert_eq!(vc.sans_initiative, vec![(pj.0, "Lyra".to_string())]);
        cb.appliquer(ActionCombat::Initiative { pj: pj.0, d20: 14 }, &ctx)
            .unwrap();
        cb.appliquer(ActionCombat::Demarrer { surprise: None }, &ctx)
            .unwrap();
        let a = cb
            .etat()
            .apercu_sort(pj.0, "Mage Armor", &[], None, &ctx)
            .unwrap();
        let av = apercu_sort(&a);
        assert_eq!(av.nom, "Armure de mage");
        assert_eq!(av.niveau_emplacement, 1);
        assert_eq!(av.duree_suggeree, Some(temps::Duree::Heures(1)));
        cb.appliquer(
            ActionCombat::Lancer(ParamsLancer {
                pj: pj.0,
                sort: "Mage Armor".into(),
                cibles: vec![pj.0],
                ..Default::default()
            }),
            &ctx,
        )
        .unwrap();
        let vc = combat(&etat, &cb);
        assert_eq!(vc.phase, "enCours");
        assert_eq!(vc.round, 1);
        assert_eq!(vc.actif, Some(pj.0));
        let p = &vc.personnages[0];
        assert!(!p.economie.simple && p.economie.mouvement);
        assert_eq!(p.pj.effets.len(), 1);
        assert_eq!(p.pj.effets[0].nom, "Armure de mage");
        assert!(p.pj.effets[0].details[0].contains("+4 armure (CA)"));
        let niv1 = p.pj.emplacements.iter().find(|e| e.niveau == 1).unwrap();
        assert!(niv1.restants < niv1.total);
        assert_eq!(p.sorts_lances, 1);
        assert!(vc
            .journal
            .iter()
            .any(|(_, t)| t.contains("lance Armure de mage")));
        let f = apercu_fin(&etat, &cb.apercu_fin(&ctx).unwrap());
        assert_eq!(f.persistants.len(), 1);
        assert_eq!(f.personnages[0].nom, "Lyra");
    }
}

/// Ce qu'un recul d'horloge toucherait : effets expirés qu'on peut réactiver, effets créés après le nouvel instant.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct ReculVue {
    pub reactivables: Vec<(u64, String, String)>,
    pub crees: Vec<(u64, String, String)>,
    /// Reculer avant minuit est refusé (il faut annuler, ce qui défait aussi le jour d'intermède).
    pub avant_minuit: bool,
}

pub fn apercu_recul(etat: &Etat, instant: i64) -> Option<ReculVue> {
    let a = etat.aventure.as_ref()?;
    let nouvel = Instant(instant);
    let ap = a.apercu_recul(nouvel);
    let contextes = etat.contextes_aventure();
    let nom_de = |s: &str| {
        s.parse::<u32>()
            .ok()
            .and_then(|id| contextes.get(&id))
            .map_or_else(|| s.to_string(), |c| c.nom.clone())
    };
    let trouve = |id: u64| {
        a.effets
            .actifs
            .iter()
            .map(|e| (e.id, e.nom.clone(), nom_de(&e.proprietaire)))
            .chain(
                a.effets
                    .termines
                    .iter()
                    .map(|t| (t.effet.id, t.effet.nom.clone(), nom_de(&t.effet.proprietaire))),
            )
            .find(|(i, _, _)| *i == id)
    };
    Some(ReculVue {
        reactivables: ap.reactivables.iter().filter_map(|i| trouve(*i)).collect(),
        crees: ap.crees.iter().filter_map(|i| trouve(*i)).collect(),
        avant_minuit: nouvel.jour() < a.horloge.maintenant.jour(),
    })
}
