//! Journal d'événements d'un personnage : historique complet, annulation, recalcul propre.
//! L'état (fiche, XP, jalons) est toujours recalculé en rejouant le journal.

use crate::donnees::Referentiel;
use crate::etats::EffetTemporaire;
use crate::fiche::{Fiche, Probleme};
use crate::progression::{appliquer, niveau_atteignable, ErreurMontee, Montee, ReglesFiche};

/// Où se porte une protection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
pub enum EmplacementProtection {
    Armure,
    Bouclier,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum EvenementFiche {
    /// Création du personnage (niveau 1).
    Creation(Box<Fiche>),
    XpGagnes {
        nombre: u32,
        motif: String,
    },
    /// Mode jalons : le MJ accorde le niveau indiqué.
    JalonAccorde {
        niveau: u32,
    },
    MonteeDeNiveau(Montee),
    /// Un sort connu en double (ou déjà accordé par le lignage) est remplacé par un autre
    /// sort du même niveau, de la liste de la classe.
    SortRemplace {
        ancien: String,
        nouveau: String,
    },
    /// Un emplacement de sort dépensé (niveau de sort 1 à 9). Les tours de magie sont à volonté.
    SortLance {
        classe: String,
        niveau_de_sort: u32,
        sort: String,
    },
    /// Repos : tous les emplacements sont rendus.
    Repos,
    /// On enfile (ou retire, avec `None`) une armure ou un bouclier (nom anglais du catalogue).
    Equiper {
        emplacement: EmplacementProtection,
        objet: Option<String>,
    },
    ArmeAjoutee(String),
    ArmeRetiree(String),
    /// Bonus d'altération (1 à 5) de l'armure, du bouclier ou d'une arme portés ; 0 l'enlève.
    BonusMagique {
        objet: String,
        bonus: i32,
    },
    /// Objet magique porté (anneau, cape…) : le joueur en saisit les effets.
    ObjetMagiqueAjoute(crate::equipement::ObjetMagique),
    ObjetMagiqueRetire(String),
    /// Le joueur indique sa charge (légère, intermédiaire, lourde) ; elle n'est pas déduite du poids.
    ChargeDeclaree(crate::equipement::Charge),
    ObjetAjoute {
        nom_en: String,
        quantite: u32,
    },
    ObjetRetire {
        nom_en: String,
        quantite: u32,
    },
    EtatAjoute(String),
    EtatRetire(String),
    EffetTemporaireAjoute(EffetTemporaire),
    EffetTemporaireRetire(String),
    /// Le temps passe : les effets temporaires à durée perdent des rounds et disparaissent à zéro.
    TempsEcoule {
        rounds: u32,
    },
    /// Dommages de caractéristique subis (`delta` > 0) ou récupérés (`delta` < 0), jamais sous zéro.
    DommagesCaracteristique {
        carac: crate::caracteristiques::Carac,
        delta: i32,
    },
    /// Préparation des sorts du jour d'un lanceur qui prépare (prêtre, druide) : remplace la
    /// préparation précédente et rend tous les emplacements de cette classe.
    Preparation {
        classe: String,
        sorts: Vec<crate::preparation::SortPrepare>,
    },
    /// Un pouvoir limité par jour est utilisé (canalisation, pouvoir de domaine, forme animale…).
    /// `cle` : identifiant donné par les valeurs de la fiche.
    PouvoirUtilise {
        cle: String,
    },
    /// Le druide prend une forme (forme animale) : consomme une utilisation du jour et ajoute les
    /// bonus de la forme comme effet temporaire (1 heure par niveau de druide).
    FormeAdoptee {
        forme: String,
    },
    /// Le druide reprend sa forme normale.
    FormeQuittee,
    /// Choix du compagnon animal (nom, développement, augmentations de caractéristique) : remplace
    /// le précédent.
    CompagnonModifie(crate::compagnon::Compagnon),
    /// Un prêtre sacrifie un sort préparé (ni oraison ni sort de domaine) pour lancer
    /// spontanément un sort de soins ou de blessure de niveau égal ou inférieur.
    SortConverti {
        classe: String,
        niveau_sacrifie: u32,
        sort: String,
    },
    /// Les rangs de compétence d'un niveau (1 = premier niveau du personnage) sont replacés.
    RangsReattribues {
        niveau: u32,
        rangs: std::collections::BTreeMap<String, u32>,
    },
    /// Rangs réattribués globalement : total voulu par compétence, réparti automatiquement
    /// sur les niveaux selon les rangs disponibles à chacun.
    RangsReattribuesGlobal {
        rangs: std::collections::BTreeMap<String, u32>,
    },
    /// Dons choisis d'un niveau (1 = premier niveau du personnage) remplacés. Le nombre de dons
    /// ne peut pas dépasser celui du niveau ; il peut rester inférieur (fiche à compléter).
    DonsReattribues {
        niveau: u32,
        dons: Vec<String>,
    },
    /// Liste des sorts connus remplacée (les sorts accordés par le lignage ne se choisissent pas).
    /// Chaque sort est vérifié contre la liste de sa classe et le nombre par niveau de sort.
    SortsReattribues {
        sorts: Vec<String>,
    },
}

impl EvenementFiche {
    /// Ligne lisible pour l'historique.
    pub fn description(&self) -> String {
        match self {
            EvenementFiche::Creation(f) => format!("Création de {}", f.nom),
            EvenementFiche::XpGagnes { nombre, motif } if motif.is_empty() => {
                format!("+{nombre} XP")
            }
            EvenementFiche::XpGagnes { nombre, motif } => format!("+{nombre} XP ({motif})"),
            EvenementFiche::JalonAccorde { niveau } => format!("Jalon : niveau {niveau} accordé"),
            EvenementFiche::MonteeDeNiveau(m) => format!("Montée de niveau en {}", m.classe),
            EvenementFiche::SortRemplace { ancien, nouveau } => {
                format!("Sort remplacé : {ancien} → {nouveau}")
            }
            EvenementFiche::SortLance {
                niveau_de_sort,
                sort,
                ..
            } => format!("Sort lancé : {sort} (niveau {niveau_de_sort})"),
            EvenementFiche::Repos => "Repos : emplacements de sorts rendus".to_string(),
            EvenementFiche::Equiper { emplacement, objet } => match (emplacement, objet) {
                (EmplacementProtection::Armure, Some(o)) => format!("Armure portée : {o}"),
                (EmplacementProtection::Armure, None) => "Armure retirée".to_string(),
                (EmplacementProtection::Bouclier, Some(o)) => format!("Bouclier porté : {o}"),
                (EmplacementProtection::Bouclier, None) => "Bouclier retiré".to_string(),
            },
            EvenementFiche::ChargeDeclaree(c) => format!("Charge portée : {}", c.nom()),
            EvenementFiche::ArmeAjoutee(a) => format!("Arme prise : {a}"),
            EvenementFiche::ArmeRetiree(a) => format!("Arme rangée : {a}"),
            EvenementFiche::BonusMagique { objet, bonus: 0 } => {
                format!("Bonus magique enlevé : {objet}")
            }
            EvenementFiche::BonusMagique { objet, bonus } => {
                format!("Bonus magique : {objet} +{bonus}")
            }
            EvenementFiche::ObjetMagiqueAjoute(o) => format!("Objet magique porté : {}", o.nom),
            EvenementFiche::ObjetMagiqueRetire(n) => format!("Objet magique retiré : {n}"),
            EvenementFiche::ObjetAjoute { nom_en, quantite } => {
                format!("Objet ajouté : {nom_en} × {quantite}")
            }
            EvenementFiche::ObjetRetire { nom_en, quantite } => {
                format!("Objet retiré : {nom_en} × {quantite}")
            }
            EvenementFiche::EtatAjoute(e) => format!("État subi : {e}"),
            EvenementFiche::EtatRetire(e) => format!("État terminé : {e}"),
            EvenementFiche::EffetTemporaireAjoute(t) => format!("Effet temporaire : {}", t.nom),
            EvenementFiche::EffetTemporaireRetire(n) => format!("Effet temporaire terminé : {n}"),
            EvenementFiche::TempsEcoule { rounds } => format!("{rounds} round(s) écoulé(s)"),
            EvenementFiche::DommagesCaracteristique { carac, delta } if *delta >= 0 => {
                format!("Dommages de caractéristique : {} −{delta}", carac.abrege())
            }
            EvenementFiche::DommagesCaracteristique { carac, delta } => {
                format!("Caractéristique récupérée : {} +{}", carac.abrege(), -delta)
            }
            EvenementFiche::PouvoirUtilise { cle } => format!("Pouvoir utilisé : {cle}"),
            EvenementFiche::FormeAdoptee { forme } => format!("Forme animale prise : {forme}"),
            EvenementFiche::FormeQuittee => "Forme normale retrouvée".to_string(),
            EvenementFiche::CompagnonModifie(c) => format!("Compagnon animal modifié : {}", c.nom),
            EvenementFiche::Preparation { sorts, .. } => {
                format!("Sorts préparés : {} sort(s)", sorts.len())
            }
            EvenementFiche::SortConverti {
                niveau_sacrifie,
                sort,
                ..
            } => format!("Sort converti : {sort} (emplacement de niveau {niveau_sacrifie})"),
            EvenementFiche::RangsReattribuesGlobal { .. } => {
                "Rangs de compétence réattribués (tous niveaux)".to_string()
            }
            EvenementFiche::RangsReattribues { niveau, .. } => {
                format!("Rangs de compétence réattribués (niveau {niveau})")
            }
            EvenementFiche::DonsReattribues { niveau, .. } => {
                format!("Dons réattribués (niveau {niveau})")
            }
            EvenementFiche::SortsReattribues { .. } => "Sorts connus réattribués".to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EtatPersonnage {
    pub fiche: Fiche,
    pub xp: u32,
    pub jalon: u32,
    /// Niveau permis par les XP ou les jalons.
    pub niveau_atteignable: u32,
    /// Emplacements de sorts dépensés depuis le dernier repos : (classe, niveau de sort) → nombre.
    pub sorts_depenses: std::collections::BTreeMap<(String, u32), u32>,
    /// Sorts préparés du jour, par classe (lanceurs qui préparent).
    pub prepares: std::collections::BTreeMap<String, Vec<crate::preparation::SortPrepare>>,
    /// Pour chaque sort préparé (même ordre), s'il a déjà été lancé.
    pub prepares_utilises: std::collections::BTreeMap<String, Vec<bool>>,
    /// Utilisations de pouvoirs depuis le dernier repos : identifiant → nombre.
    pub pouvoirs_utilises: std::collections::BTreeMap<String, u32>,
}

impl EtatPersonnage {
    /// Emplacements restants : (classe, niveau de sort, restants, total par jour).
    pub fn emplacements_restants(&self, r: &Referentiel) -> Vec<(String, u32, u32, u32)> {
        let Ok(v) = self.fiche.calculer(r) else {
            return Vec::new();
        };
        let mut out = Vec::new();
        for inc in &v.incantateurs {
            for e in &inc.emplacements {
                if let Some(par_jour) = e.par_jour {
                    let total = par_jour + e.domaine;
                    let utilises = self
                        .sorts_depenses
                        .get(&(inc.classe.clone(), e.niveau_de_sort))
                        .copied()
                        .unwrap_or(0);
                    out.push((
                        inc.classe.clone(),
                        e.niveau_de_sort,
                        total.saturating_sub(utilises),
                        total,
                    ));
                }
            }
        }
        out
    }

    /// Niveaux gagnés mais pas encore appliqués (« montée de niveau en attente »).
    /// Tant qu'ils ne sont pas appliqués, tout reste calculé au niveau actuel.
    pub fn niveaux_en_attente(&self) -> u32 {
        self.niveau_atteignable
            .saturating_sub(self.fiche.niveau_de_personnage())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErreurJournal {
    Vide,
    CreationManquante,
    Montee {
        evenement: usize,
        erreurs: Vec<ErreurMontee>,
    },
    JalonHorsLimites(u32),
    PlusDEmplacement {
        evenement: usize,
        classe: String,
        niveau_de_sort: u32,
    },
    SortLanceImpossible {
        evenement: usize,
        raison: String,
    },
    Equipement {
        evenement: usize,
        raison: String,
    },
    Rangs {
        evenement: usize,
        raison: String,
    },
    Choix {
        evenement: usize,
        raison: String,
    },
}

impl std::fmt::Display for ErreurJournal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ErreurJournal::Vide => write!(f, "journal vide"),
            ErreurJournal::CreationManquante => {
                write!(f, "le journal doit commencer par la création")
            }
            ErreurJournal::Montee { evenement, erreurs } => {
                write!(f, "événement {evenement} :")?;
                for e in erreurs {
                    write!(f, " {e};")?;
                }
                Ok(())
            }
            ErreurJournal::JalonHorsLimites(n) => write!(f, "jalon {n} hors de 1 à 20"),
            ErreurJournal::PlusDEmplacement {
                classe,
                niveau_de_sort,
                ..
            } => write!(
                f,
                "plus d'emplacement de sort de niveau {niveau_de_sort} ({classe})"
            ),
            ErreurJournal::SortLanceImpossible { raison, .. } => write!(f, "{raison}"),
            ErreurJournal::Equipement { raison, .. } => write!(f, "{raison}"),
            ErreurJournal::Rangs { raison, .. } => write!(f, "{raison}"),
            ErreurJournal::Choix { raison, .. } => write!(f, "{raison}"),
        }
    }
}

/// Consomme une utilisation d'un pouvoir limité par jour ; Err explique le refus.
fn consommer_pouvoir(
    fiche: &Fiche,
    r: &Referentiel,
    cle: &str,
    utilises: &mut std::collections::BTreeMap<String, u32>,
) -> Result<(), String> {
    let valeurs = fiche.calculer(r).map_err(|p| p.to_string())?;
    let pouvoir = valeurs
        .pouvoirs
        .iter()
        .find(|p| p.cle == cle)
        .ok_or_else(|| format!("pouvoir inconnu ou pas encore accordé : {cle}"))?;
    let n = utilises.entry(cle.to_string()).or_default();
    if pouvoir.max.is_some_and(|m| *n as i32 >= m) {
        return Err(format!(
            "plus d'utilisation de « {} » aujourd'hui",
            pouvoir.nom
        ));
    }
    *n += 1;
    Ok(())
}

/// Refuse un événement d'équipement ou d'état qui nomme un objet ou un état inconnu.
fn verifier_equipement(r: &Referentiel, fiche: &Fiche, i: usize) -> Result<(), ErreurJournal> {
    match fiche.problemes_equipement(r).first() {
        Some(p) => Err(ErreurJournal::Equipement {
            evenement: i,
            raison: p.to_string(),
        }),
        None => Ok(()),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Journal {
    pub evenements: Vec<EvenementFiche>,
}

impl Journal {
    pub fn nouveau(fiche: Fiche) -> Journal {
        Journal {
            evenements: vec![EvenementFiche::Creation(Box::new(fiche))],
        }
    }

    /// Rejoue tout le journal.
    pub fn etat(
        &self,
        r: &Referentiel,
        regles: &ReglesFiche,
    ) -> Result<EtatPersonnage, ErreurJournal> {
        let mut it = self.evenements.iter().enumerate();
        let mut fiche = match it.next() {
            Some((_, EvenementFiche::Creation(f))) => (**f).clone(),
            Some(_) => return Err(ErreurJournal::CreationManquante),
            None => return Err(ErreurJournal::Vide),
        };
        let (mut xp, mut jalon) = (0u32, 0u32);
        let mut sorts_depenses: std::collections::BTreeMap<(String, u32), u32> = Default::default();
        let mut prepares: std::collections::BTreeMap<String, Vec<crate::preparation::SortPrepare>> =
            Default::default();
        let mut prepares_utilises: std::collections::BTreeMap<String, Vec<bool>> =
            Default::default();
        let mut pouvoirs_utilises: std::collections::BTreeMap<String, u32> = Default::default();
        for (i, ev) in it {
            match ev {
                EvenementFiche::Creation(_) => return Err(ErreurJournal::CreationManquante),
                EvenementFiche::XpGagnes { nombre, .. } => xp = xp.saturating_add(*nombre),
                EvenementFiche::JalonAccorde { niveau } => {
                    if !(1..=20).contains(niveau) {
                        return Err(ErreurJournal::JalonHorsLimites(*niveau));
                    }
                    jalon = jalon.max(*niveau);
                }
                EvenementFiche::PouvoirUtilise { cle } => {
                    consommer_pouvoir(&fiche, r, cle, &mut pouvoirs_utilises).map_err(
                        |raison| ErreurJournal::SortLanceImpossible {
                            evenement: i,
                            raison,
                        },
                    )?;
                }
                EvenementFiche::FormeAdoptee { forme } => {
                    let erreur = |raison: String| ErreurJournal::SortLanceImpossible {
                        evenement: i,
                        raison,
                    };
                    let def = r
                        .forme(forme)
                        .ok_or_else(|| erreur(format!("forme inconnue : {forme}")))?;
                    let classe = fiche
                        .niveaux
                        .iter()
                        .filter_map(|n| r.classe(&n.classe))
                        .find(|c| c.pouvoirs.iter().any(|p| p.cle == "forme_animale"))
                        .ok_or_else(|| erreur("ce personnage n'a pas de forme animale".into()))?;
                    let nc = fiche.niveau_de_classe(&classe.id);
                    if nc < def.niveau_druide {
                        return Err(erreur(format!(
                            "{} demande le niveau {} de {}",
                            def.nom, def.niveau_druide, classe.nom
                        )));
                    }
                    consommer_pouvoir(&fiche, r, "forme_animale", &mut pouvoirs_utilises)
                        .map_err(erreur)?;
                    fiche
                        .temporaires
                        .retain(|t| !t.nom.starts_with(crate::formes::PREFIXE_FORME));
                    fiche.temporaires.push(EffetTemporaire {
                        nom: def.nom_effet(),
                        effets: def.effets_chiffres(),
                        // 1 heure par niveau de druide ; 1 round = 6 secondes.
                        rounds_restants: Some(nc * 600),
                    });
                }
                EvenementFiche::FormeQuittee => {
                    fiche
                        .temporaires
                        .retain(|t| !t.nom.starts_with(crate::formes::PREFIXE_FORME));
                }
                EvenementFiche::CompagnonModifie(c) => {
                    if fiche.pacte.as_deref() != Some("compagnon") {
                        return Err(ErreurJournal::SortLanceImpossible {
                            evenement: i,
                            raison: "ce personnage n'a pas choisi le compagnon animal".into(),
                        });
                    }
                    fiche.compagnon = Some(c.clone());
                    if let Some(p) = fiche.valider(r).iter().find(|p| !p.est_incomplet()) {
                        return Err(ErreurJournal::SortLanceImpossible {
                            evenement: i,
                            raison: p.to_string(),
                        });
                    }
                }
                EvenementFiche::Repos => {
                    pouvoirs_utilises.clear();
                    sorts_depenses.clear();
                    for v in prepares_utilises.values_mut() {
                        v.iter_mut().for_each(|u| *u = false);
                    }
                }
                EvenementFiche::Preparation { classe, sorts } => {
                    crate::preparation::valider(&fiche, r, classe, sorts).map_err(|raison| {
                        ErreurJournal::SortLanceImpossible {
                            evenement: i,
                            raison,
                        }
                    })?;
                    sorts_depenses.retain(|(c, _), _| c != classe);
                    prepares_utilises.insert(classe.clone(), vec![false; sorts.len()]);
                    prepares.insert(classe.clone(), sorts.clone());
                }
                EvenementFiche::SortConverti {
                    classe,
                    niveau_sacrifie,
                    sort,
                } => {
                    let erreur = |raison: String| ErreurJournal::SortLanceImpossible {
                        evenement: i,
                        raison,
                    };
                    let def = r
                        .sort(sort)
                        .ok_or_else(|| erreur(format!("sort inconnu : {sort}")))?;
                    let valeurs = fiche.calculer(r).map_err(|p| erreur(p.to_string()))?;
                    let positive = valeurs
                        .canalisation
                        .as_ref()
                        .ok_or_else(|| erreur("cette classe ne convertit pas ses sorts".into()))?
                        .positive
                        .ok_or_else(|| {
                            erreur("choisir d'abord l'énergie (positive ou négative)".into())
                        })?;
                    if !crate::preparation::est_sort_de_conversion(sort, positive) {
                        return Err(erreur(format!(
                            "{sort} n'est pas un sort de {}",
                            if positive { "soins" } else { "blessure" }
                        )));
                    }
                    let cle = r
                        .classe(classe)
                        .map(|c| c.nom_en.to_lowercase())
                        .ok_or_else(|| erreur(format!("classe inconnue : {classe}")))?;
                    match def.niveau_pour(&cle) {
                        Some(n) if n >= 1 && n <= *niveau_sacrifie => {}
                        _ => {
                            return Err(erreur(format!(
                            "{sort} n'est pas un sort de niveau 1 à {niveau_sacrifie} de la classe"
                        )))
                        }
                    }
                    let liste = prepares.get(classe).cloned().unwrap_or_default();
                    let utilises = prepares_utilises.entry(classe.clone()).or_default();
                    let Some(idx) = liste.iter().enumerate().position(|(k, s)| {
                        s.niveau == *niveau_sacrifie
                            && !s.domaine
                            && !utilises.get(k).copied().unwrap_or(true)
                    }) else {
                        return Err(erreur(format!(
                            "aucun sort préparé disponible au niveau {niveau_sacrifie} à sacrifier"
                        )));
                    };
                    utilises[idx] = true;
                    *sorts_depenses
                        .entry((classe.clone(), *niveau_sacrifie))
                        .or_default() += 1;
                }
                EvenementFiche::SortLance {
                    classe,
                    niveau_de_sort,
                    sort,
                } => {
                    if *niveau_de_sort == 0 || *niveau_de_sort > 9 {
                        return Err(ErreurJournal::SortLanceImpossible {
                            evenement: i,
                            raison: "les tours de magie se lancent à volonté".into(),
                        });
                    }
                    let valeurs =
                        fiche
                            .calculer(r)
                            .map_err(|p| ErreurJournal::SortLanceImpossible {
                                evenement: i,
                                raison: p.to_string(),
                            })?;
                    let total = valeurs
                        .incantateurs
                        .iter()
                        .find(|inc| &inc.classe == classe)
                        .and_then(|inc| {
                            inc.emplacements
                                .iter()
                                .find(|e| e.niveau_de_sort == *niveau_de_sort)
                        })
                        .and_then(|e| e.par_jour.map(|p| p + e.domaine))
                        .ok_or_else(|| ErreurJournal::SortLanceImpossible {
                            evenement: i,
                            raison: format!(
                                "{classe} ne lance pas de sorts de niveau {niveau_de_sort}"
                            ),
                        })?;
                    let utilises = sorts_depenses
                        .entry((classe.clone(), *niveau_de_sort))
                        .or_default();
                    if *utilises >= total {
                        return Err(ErreurJournal::PlusDEmplacement {
                            evenement: i,
                            classe: classe.clone(),
                            niveau_de_sort: *niveau_de_sort,
                        });
                    }
                    // Lanceur qui prépare : le sort doit faire partie de la préparation du jour et
                    // ne pas avoir déjà été lancé.
                    let prepare_classe = r
                        .classe(classe)
                        .and_then(|c| c.incantation.as_ref())
                        .is_some_and(|inc| inc.prepare());
                    if prepare_classe {
                        let liste = prepares.get(classe).cloned().unwrap_or_default();
                        let drapeaux = prepares_utilises.entry(classe.clone()).or_default();
                        let Some(idx) = liste.iter().enumerate().position(|(k, s)| {
                            s.niveau == *niveau_de_sort
                                && s.sort.eq_ignore_ascii_case(sort)
                                && !drapeaux.get(k).copied().unwrap_or(true)
                        }) else {
                            return Err(ErreurJournal::SortLanceImpossible {
                                evenement: i,
                                raison: format!(
                                    "{sort} n'est pas préparé au niveau {niveau_de_sort}, ou déjà lancé"
                                ),
                            });
                        };
                        drapeaux[idx] = true;
                        *utilises += 1;
                        continue;
                    }
                    let connu = fiche
                        .sorts_connus
                        .iter()
                        .any(|s| s.eq_ignore_ascii_case(sort))
                        || r.classe(classe).is_some_and(|c| {
                            fiche
                                .calculer(r)
                                .map(|v| {
                                    v.incantateurs.iter().any(|inc| {
                                        &inc.classe == classe
                                            && inc
                                                .sorts_de_lignage
                                                .iter()
                                                .any(|(n, _)| n.eq_ignore_ascii_case(sort))
                                    })
                                })
                                .unwrap_or(false)
                                && c.incantation.is_some()
                        });
                    if !connu {
                        return Err(ErreurJournal::SortLanceImpossible {
                            evenement: i,
                            raison: format!("{sort} n'est pas un sort connu"),
                        });
                    }
                    *utilises += 1;
                }
                EvenementFiche::Equiper { emplacement, objet } => {
                    let ancien = match emplacement {
                        EmplacementProtection::Armure => {
                            std::mem::replace(&mut fiche.equipement.armure, objet.clone())
                        }
                        EmplacementProtection::Bouclier => {
                            std::mem::replace(&mut fiche.equipement.bouclier, objet.clone())
                        }
                    };
                    // Le bonus magique part avec l'objet qu'on retire.
                    if let Some(a) = ancien.filter(|a| Some(a) != objet.as_ref()) {
                        fiche.equipement.bonus_magiques.remove(&a);
                    }
                    verifier_equipement(r, &fiche, i)?;
                }
                EvenementFiche::ChargeDeclaree(c) => fiche.equipement.charge_declaree = *c,
                EvenementFiche::ArmeAjoutee(a) => {
                    fiche.equipement.armes.push(a.clone());
                    verifier_equipement(r, &fiche, i)?;
                }
                EvenementFiche::ArmeRetiree(a) => {
                    let Some(pos) = fiche.equipement.armes.iter().position(|x| x == a) else {
                        return Err(ErreurJournal::Equipement {
                            evenement: i,
                            raison: format!("{a} n'est pas portée"),
                        });
                    };
                    fiche.equipement.armes.remove(pos);
                    if !fiche.equipement.armes.contains(a) {
                        fiche.equipement.bonus_magiques.remove(a);
                    }
                }
                EvenementFiche::BonusMagique { objet, bonus } => {
                    if *bonus == 0 {
                        fiche.equipement.bonus_magiques.remove(objet);
                    } else {
                        fiche
                            .equipement
                            .bonus_magiques
                            .insert(objet.clone(), *bonus);
                    }
                    verifier_equipement(r, &fiche, i)?;
                }
                EvenementFiche::ObjetMagiqueAjoute(o) => {
                    fiche.equipement.objets_magiques.retain(|x| x.nom != o.nom);
                    fiche.equipement.objets_magiques.push(o.clone());
                }
                EvenementFiche::ObjetMagiqueRetire(n) => {
                    let avant = fiche.equipement.objets_magiques.len();
                    fiche.equipement.objets_magiques.retain(|x| &x.nom != n);
                    if fiche.equipement.objets_magiques.len() == avant {
                        return Err(ErreurJournal::Equipement {
                            evenement: i,
                            raison: format!("{n} n'est pas porté"),
                        });
                    }
                }
                EvenementFiche::ObjetAjoute { nom_en, quantite } => {
                    fiche.equipement.ajouter_objet(nom_en, *quantite);
                    verifier_equipement(r, &fiche, i)?;
                }
                EvenementFiche::ObjetRetire { nom_en, quantite } => {
                    if !fiche.equipement.retirer_objet(nom_en, *quantite) {
                        return Err(ErreurJournal::Equipement {
                            evenement: i,
                            raison: format!("pas assez de {nom_en} à retirer"),
                        });
                    }
                }
                EvenementFiche::EtatAjoute(e) => {
                    if !fiche.etats.contains(e) {
                        fiche.etats.push(e.clone());
                    }
                    verifier_equipement(r, &fiche, i)?;
                }
                EvenementFiche::EtatRetire(e) => {
                    let avant = fiche.etats.len();
                    fiche.etats.retain(|x| x != e);
                    if fiche.etats.len() == avant {
                        return Err(ErreurJournal::Equipement {
                            evenement: i,
                            raison: format!("l'état {e} n'est pas actif"),
                        });
                    }
                }
                EvenementFiche::EffetTemporaireAjoute(t) => {
                    fiche.temporaires.retain(|x| x.nom != t.nom);
                    fiche.temporaires.push(t.clone());
                }
                EvenementFiche::EffetTemporaireRetire(n) => {
                    let avant = fiche.temporaires.len();
                    fiche.temporaires.retain(|x| &x.nom != n);
                    if fiche.temporaires.len() == avant {
                        return Err(ErreurJournal::Equipement {
                            evenement: i,
                            raison: format!("l'effet {n} n'est pas actif"),
                        });
                    }
                }
                EvenementFiche::TempsEcoule { rounds } => {
                    for t in &mut fiche.temporaires {
                        if let Some(restants) = &mut t.rounds_restants {
                            *restants = restants.saturating_sub(*rounds);
                        }
                    }
                    fiche.temporaires.retain(|t| t.rounds_restants != Some(0));
                }
                EvenementFiche::DommagesCaracteristique { carac, delta } => {
                    let d = fiche.dommages.entry(*carac).or_insert(0);
                    *d = (*d + *delta).max(0);
                }
                EvenementFiche::SortRemplace { ancien, nouveau } => {
                    let permis = fiche.remplacements_de_sorts(r).into_iter().any(|x| {
                        x.ancien.eq_ignore_ascii_case(ancien)
                            && x.propositions
                                .iter()
                                .any(|p| p.eq_ignore_ascii_case(nouveau))
                    });
                    if !permis {
                        return Err(ErreurJournal::SortLanceImpossible {
                            evenement: i,
                            raison: format!(
                                "{ancien} ne peut pas être remplacé par {nouveau} (même niveau, liste de la classe, pas de doublon)"
                            ),
                        });
                    }
                    if let Some(pos) = fiche
                        .sorts_connus
                        .iter()
                        .position(|s| s.eq_ignore_ascii_case(ancien))
                    {
                        // le doublon retiré est la dernière occurrence : la première reste
                        let pos = fiche
                            .sorts_connus
                            .iter()
                            .rposition(|s| s.eq_ignore_ascii_case(ancien))
                            .unwrap_or(pos);
                        fiche.sorts_connus[pos] = nouveau.clone();
                    }
                }
                EvenementFiche::RangsReattribuesGlobal { rangs } => {
                    for n in &mut fiche.niveaux {
                        n.rangs.clear();
                    }
                    let budgets: Vec<u32> = match fiche.calculer(r) {
                        Ok(v) => v.rangs.iter().map(|b| b.disponibles).collect(),
                        Err(p) => {
                            return Err(ErreurJournal::Rangs {
                                evenement: i,
                                raison: p.to_string(),
                            })
                        }
                    };
                    let total: u32 = rangs.values().sum();
                    if total > budgets.iter().sum::<u32>() {
                        return Err(ErreurJournal::Rangs {
                            evenement: i,
                            raison: format!(
                                "{total} rangs demandés pour {} disponibles",
                                budgets.iter().sum::<u32>()
                            ),
                        });
                    }
                    // remplissage du premier niveau qui a encore de la place
                    let mut restants = budgets;
                    for (c, v) in rangs.iter().filter(|(_, v)| **v > 0) {
                        for _ in 0..*v {
                            if let Some(k) = restants.iter().position(|b| *b > 0) {
                                restants[k] -= 1;
                                *fiche.niveaux[k].rangs.entry(c.clone()).or_default() += 1;
                            }
                        }
                    }
                    if let Some(p) = fiche.valider(r).into_iter().find(|p| {
                        matches!(
                            p,
                            Probleme::RangsDepasses { .. }
                                | Probleme::RangAuDessusDuNiveau { .. }
                                | Probleme::CompetenceInconnue(_)
                        )
                    }) {
                        return Err(ErreurJournal::Rangs {
                            evenement: i,
                            raison: p.to_string(),
                        });
                    }
                }
                EvenementFiche::RangsReattribues { niveau, rangs } => {
                    let Some(pris) = (*niveau as usize)
                        .checked_sub(1)
                        .and_then(|k| fiche.niveaux.get_mut(k))
                    else {
                        return Err(ErreurJournal::Rangs {
                            evenement: i,
                            raison: format!("le niveau {niveau} n'existe pas encore"),
                        });
                    };
                    pris.rangs = rangs
                        .iter()
                        .filter(|(_, v)| **v > 0)
                        .map(|(c, v)| (c.clone(), *v))
                        .collect();
                    let trouve = fiche.valider(r).into_iter().find(|p| {
                        matches!(
                            p,
                            Probleme::RangsDepasses { .. }
                                | Probleme::RangAuDessusDuNiveau { .. }
                                | Probleme::CompetenceInconnue(_)
                        )
                    });
                    if let Some(p) = trouve {
                        return Err(ErreurJournal::Rangs {
                            evenement: i,
                            raison: p.to_string(),
                        });
                    }
                }
                EvenementFiche::DonsReattribues { niveau, dons } => {
                    let Some(pris) = (*niveau as usize)
                        .checked_sub(1)
                        .and_then(|k| fiche.niveaux.get_mut(k))
                    else {
                        return Err(ErreurJournal::Choix {
                            evenement: i,
                            raison: format!("le niveau {niveau} n'existe pas encore"),
                        });
                    };
                    pris.dons = dons.clone();
                    let trouve = fiche.valider(r).into_iter().find(|p| match p {
                        Probleme::DonInconnu(d) => dons.contains(d),
                        Probleme::DonEnDouble(_) | Probleme::DonDeLignageManquant { .. } => true,
                        Probleme::DonsAuNiveau {
                            niveau: n,
                            attendus,
                            choisis,
                        } => n == niveau && choisis > attendus,
                        Probleme::DonImposeManquant(_) => *niveau == 1,
                        _ => false,
                    });
                    if let Some(p) = trouve {
                        return Err(ErreurJournal::Choix {
                            evenement: i,
                            raison: p.to_string(),
                        });
                    }
                }
                EvenementFiche::SortsReattribues { sorts } => {
                    fiche.sorts_connus = sorts.clone();
                    let trouve = fiche.valider(r).into_iter().find(|p| match p {
                        Probleme::SortInconnu(_)
                        | Probleme::SortHorsListe { .. }
                        | Probleme::SortEnDouble(_)
                        | Probleme::SortDeLignageChoisi(_)
                        | Probleme::CharismeInsuffisantPourSort { .. } => true,
                        Probleme::SortsAChoisir {
                            attendus, choisis, ..
                        } => choisis > attendus,
                        _ => false,
                    });
                    if let Some(p) = trouve {
                        return Err(ErreurJournal::Choix {
                            evenement: i,
                            raison: p.to_string(),
                        });
                    }
                }
                EvenementFiche::MonteeDeNiveau(m) => {
                    let atteignable = niveau_atteignable(r, regles.vitesse, xp, jalon);
                    fiche = appliquer(r, regles, &fiche, m, atteignable).map_err(|erreurs| {
                        ErreurJournal::Montee {
                            evenement: i,
                            erreurs,
                        }
                    })?;
                }
            }
        }
        let niveau_atteignable = niveau_atteignable(r, regles.vitesse, xp, jalon);
        Ok(EtatPersonnage {
            fiche,
            xp,
            jalon,
            niveau_atteignable,
            sorts_depenses,
            prepares,
            prepares_utilises,
            pouvoirs_utilises,
        })
    }

    /// Ajoute un événement s'il est valide ; sinon le journal reste inchangé.
    pub fn ajouter(
        &mut self,
        ev: EvenementFiche,
        r: &Referentiel,
        regles: &ReglesFiche,
    ) -> Result<(), ErreurJournal> {
        self.evenements.push(ev);
        if let Err(e) = self.etat(r, regles) {
            self.evenements.pop();
            return Err(e);
        }
        Ok(())
    }

    /// Annule le dernier événement (jamais la création) et le renvoie.
    pub fn annuler_dernier(&mut self) -> Option<EvenementFiche> {
        if self.evenements.len() > 1 {
            self.evenements.pop()
        } else {
            None
        }
    }

    pub fn historique(&self) -> Vec<String> {
        self.evenements.iter().map(|e| e.description()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Montée valide du niveau 1 au niveau 2 de la fiche de test.
    fn niveau_simple() -> Montee {
        let r = Referentiel::core();
        let f = crate::fiche::tests::elfe_ensorceleuse();
        crate::progression::montee_auto(&r, &f, "ensorceleur", 4)
    }

    fn journal() -> Journal {
        Journal::nouveau(crate::fiche::tests::elfe_ensorceleuse())
    }

    fn rangs(liste: &[(&str, u32)]) -> EvenementFiche {
        EvenementFiche::RangsReattribues {
            niveau: 1,
            rangs: liste.iter().map(|(c, v)| (c.to_string(), *v)).collect(),
        }
    }

    #[test]
    fn rangs_reattribues_remplacent_ceux_du_niveau() {
        let r = Referentiel::core();
        let regles = ReglesFiche::default();
        let mut j = journal();
        let ev = rangs(&[("bluff", 1), ("perception", 1), ("spellcraft", 1)]);
        j.ajouter(ev, &r, &regles).unwrap();
        let e = j.etat(&r, &regles).unwrap();
        let places: u32 = e.fiche.niveaux[0].rangs.values().sum();
        assert_eq!(places, 3);
        assert_eq!(e.fiche.niveaux[0].rangs.get("bluff"), Some(&1));
        // on peut en placer moins que le maximum, et annuler revient à l'état d'avant
        let avant = j.clone();
        j.ajouter(rangs(&[("bluff", 1)]), &r, &regles).unwrap();
        assert!(j.annuler_dernier().is_some());
        assert_eq!(j, avant);
    }

    #[test]
    fn rangs_reattribues_globalement() {
        let r = Referentiel::core();
        let regles = ReglesFiche::default();
        let mut j = journal();
        let global = |l: &[(&str, u32)]| EvenementFiche::RangsReattribuesGlobal {
            rangs: l.iter().map(|(c, v)| (c.to_string(), *v)).collect(),
        };
        j.ajouter(global(&[("bluff", 1), ("perception", 1)]), &r, &regles)
            .unwrap();
        let e = j.etat(&r, &regles).unwrap();
        assert_eq!(e.fiche.niveaux[0].rangs.values().sum::<u32>(), 2);
        // trop de rangs, rang au-dessus du niveau, compétence inconnue : refusés
        let avant = j.clone();
        let quatre = [("bluff", 1), ("perception", 1), ("climb", 1), ("swim", 1)];
        assert!(j.ajouter(global(&quatre), &r, &regles).is_err());
        assert!(j.ajouter(global(&[("bluff", 2)]), &r, &regles).is_err());
        assert!(j.ajouter(global(&[("inconnue", 1)]), &r, &regles).is_err());
        assert_eq!(j, avant);
    }

    #[test]
    fn rangs_reattribues_refuses_si_hors_regles() {
        let r = Referentiel::core();
        let regles = ReglesFiche::default();
        let mut j = journal();
        let avant = j.clone();
        // trop de rangs pour le niveau (3 disponibles)
        let e = j
            .ajouter(
                rangs(&[
                    ("bluff", 1),
                    ("perception", 1),
                    ("spellcraft", 1),
                    ("climb", 1),
                ]),
                &r,
                &regles,
            )
            .unwrap_err();
        assert!(matches!(e, ErreurJournal::Rangs { .. }));
        // rang maximum = niveau : 2 rangs en niveau 1
        assert!(j.ajouter(rangs(&[("bluff", 2)]), &r, &regles).is_err());
        // compétence inconnue
        assert!(j.ajouter(rangs(&[("inconnue", 1)]), &r, &regles).is_err());
        // niveau pas encore pris
        let ev = EvenementFiche::RangsReattribues {
            niveau: 2,
            rangs: Default::default(),
        };
        assert!(j.ajouter(ev, &r, &regles).is_err());
        assert_eq!(j, avant);
    }

    #[test]
    fn montee_en_attente_puis_appliquee() {
        let r = Referentiel::core();
        let regles = ReglesFiche::default();
        let mut j = journal();
        j.ajouter(
            EvenementFiche::XpGagnes {
                nombre: 2500,
                motif: "session 1".into(),
            },
            &r,
            &regles,
        )
        .unwrap();
        let e = j.etat(&r, &regles).unwrap();
        // 2 500 XP en vitesse moyenne : niveau 2 atteignable (2 000), pas le 3 (5 000)
        assert_eq!(e.niveau_atteignable, 2);
        assert_eq!(e.niveaux_en_attente(), 1);
        // tant que la montée n'est pas appliquée, tout est calculé au niveau 1
        assert_eq!(e.fiche.niveau_de_personnage(), 1);
        assert_eq!(e.fiche.calculer(&r).unwrap().bba.total, 0);
        j.ajouter(EvenementFiche::MonteeDeNiveau(niveau_simple()), &r, &regles)
            .unwrap();
        let e = j.etat(&r, &regles).unwrap();
        assert_eq!(e.niveaux_en_attente(), 0);
        assert_eq!(e.fiche.calculer(&r).unwrap().bba.total, 1);
    }

    #[test]
    fn plusieurs_niveaux_cumules() {
        let r = Referentiel::core();
        let regles = ReglesFiche::default();
        let mut j = journal();
        j.ajouter(
            EvenementFiche::XpGagnes {
                nombre: 9000,
                motif: String::new(),
            },
            &r,
            &regles,
        )
        .unwrap();
        assert_eq!(j.etat(&r, &regles).unwrap().niveaux_en_attente(), 3); // niveau 4 = 9 000 XP
    }

    #[test]
    fn montee_refusee_sans_xp_et_journal_inchange() {
        let r = Referentiel::core();
        let regles = ReglesFiche::default();
        let mut j = journal();
        let avant = j.clone();
        let e = j
            .ajouter(EvenementFiche::MonteeDeNiveau(niveau_simple()), &r, &regles)
            .unwrap_err();
        assert!(matches!(e, ErreurJournal::Montee { evenement: 1, .. }));
        assert_eq!(j, avant);
    }

    #[test]
    fn annuler_la_derniere_montee_restaure_l_etat() {
        let r = Referentiel::core();
        let regles = ReglesFiche::default();
        let mut j = journal();
        j.ajouter(
            EvenementFiche::XpGagnes {
                nombre: 2000,
                motif: String::new(),
            },
            &r,
            &regles,
        )
        .unwrap();
        let avant = j.etat(&r, &regles).unwrap();
        j.ajouter(EvenementFiche::MonteeDeNiveau(niveau_simple()), &r, &regles)
            .unwrap();
        let annule = j.annuler_dernier();
        assert!(matches!(annule, Some(EvenementFiche::MonteeDeNiveau(_))));
        assert_eq!(j.etat(&r, &regles).unwrap(), avant);
        // la création ne s'annule pas
        let mut seul = journal();
        assert_eq!(seul.annuler_dernier(), None);
    }

    #[test]
    fn jalons_et_historique() {
        let r = Referentiel::core();
        let regles = ReglesFiche {
            vitesse: crate::progression::Vitesse::Jalons,
            ..ReglesFiche::default()
        };
        let mut j = journal();
        j.ajouter(
            EvenementFiche::XpGagnes {
                nombre: 999_999,
                motif: String::new(),
            },
            &r,
            &regles,
        )
        .unwrap();
        assert_eq!(j.etat(&r, &regles).unwrap().niveaux_en_attente(), 0); // les XP ne comptent pas
        j.ajouter(EvenementFiche::JalonAccorde { niveau: 2 }, &r, &regles)
            .unwrap();
        assert_eq!(j.etat(&r, &regles).unwrap().niveaux_en_attente(), 1);
        assert!(j
            .ajouter(EvenementFiche::JalonAccorde { niveau: 25 }, &r, &regles)
            .is_err());
        j.ajouter(EvenementFiche::MonteeDeNiveau(niveau_simple()), &r, &regles)
            .unwrap();
        assert_eq!(
            j.historique(),
            vec![
                "Création de Lyra".to_string(),
                "+999999 XP".to_string(),
                "Jalon : niveau 2 accordé".to_string(),
                "Montée de niveau en ensorceleur".to_string()
            ]
        );
    }

    #[test]
    fn la_vitesse_peut_changer_sans_casser_l_historique() {
        let r = Referentiel::core();
        let moyenne = ReglesFiche::default();
        let mut j = journal();
        j.ajouter(
            EvenementFiche::XpGagnes {
                nombre: 2000,
                motif: String::new(),
            },
            &r,
            &moyenne,
        )
        .unwrap();
        j.ajouter(
            EvenementFiche::MonteeDeNiveau(niveau_simple()),
            &r,
            &moyenne,
        )
        .unwrap();
        // en vitesse lente, 2 000 XP ne suffisent plus : le journal rejoué signale le problème
        let lente = ReglesFiche {
            vitesse: crate::progression::Vitesse::Lente,
            ..moyenne
        };
        assert!(j.etat(&r, &lente).is_err());
    }

    #[test]
    fn emplacements_depenses_puis_rendus_au_repos() {
        let r = Referentiel::core();
        let regles = ReglesFiche::default();
        let mut j = journal();
        let sort = |nom: &str| EvenementFiche::SortLance {
            classe: "ensorceleur".into(),
            niveau_de_sort: 1,
            sort: nom.into(),
        };
        // niveau 1 : 3 + 1 (CHA 16) = 4 sorts de niveau 1 par jour
        for _ in 0..4 {
            j.ajouter(sort("Magic Missile"), &r, &regles).unwrap();
        }
        let e = j.etat(&r, &regles).unwrap();
        let restants = e.emplacements_restants(&r);
        assert!(restants.contains(&("ensorceleur".to_string(), 1, 0, 4)));
        // le cinquième est refusé et le journal reste inchangé
        let avant = j.clone();
        let err = j.ajouter(sort("Mage Armor"), &r, &regles).unwrap_err();
        assert!(matches!(
            err,
            ErreurJournal::PlusDEmplacement {
                niveau_de_sort: 1,
                ..
            }
        ));
        assert_eq!(j, avant);
        j.ajouter(EvenementFiche::Repos, &r, &regles).unwrap();
        let e = j.etat(&r, &regles).unwrap();
        assert!(e
            .emplacements_restants(&r)
            .contains(&("ensorceleur".to_string(), 1, 4, 4)));
    }

    #[test]
    fn sort_inconnu_ou_tour_de_magie_refuses() {
        let r = Referentiel::core();
        let regles = ReglesFiche::default();
        let mut j = journal();
        let inconnu = EvenementFiche::SortLance {
            classe: "ensorceleur".into(),
            niveau_de_sort: 1,
            sort: "Fireball".into(),
        };
        assert!(matches!(
            j.ajouter(inconnu, &r, &regles),
            Err(ErreurJournal::SortLanceImpossible { .. })
        ));
        let tour = EvenementFiche::SortLance {
            classe: "ensorceleur".into(),
            niveau_de_sort: 0,
            sort: "Light".into(),
        };
        assert!(j.ajouter(tour, &r, &regles).is_err());
        // pas de sort de niveau 2 au niveau 1
        let haut = EvenementFiche::SortLance {
            classe: "ensorceleur".into(),
            niveau_de_sort: 2,
            sort: "Magic Missile".into(),
        };
        assert!(j.ajouter(haut, &r, &regles).is_err());
    }

    #[test]
    fn dons_reattribues_respectent_le_nombre_du_niveau() {
        let r = Referentiel::core();
        let regles = ReglesFiche::default();
        let mut j = journal();
        let avant = j.clone();
        let dons = |l: &[&str]| EvenementFiche::DonsReattribues {
            niveau: 1,
            dons: l.iter().map(|d| d.to_string()).collect(),
        };
        // le don déjà pris est accepté tel quel (aucun changement)
        j.ajouter(dons(&["Toughness"]), &r, &regles).unwrap();
        assert_eq!(
            j.etat(&r, &regles).unwrap().fiche.niveaux[0].dons,
            ["Toughness"]
        );
        // plus de dons que le niveau n'en accorde, don inconnu, niveau absent : refusés
        assert!(j
            .ajouter(dons(&["Toughness", "Toughness"]), &r, &regles)
            .is_err());
        assert!(j.ajouter(dons(&["Inconnu"]), &r, &regles).is_err());
        let niveau_2 = EvenementFiche::DonsReattribues {
            niveau: 2,
            dons: vec![],
        };
        assert!(matches!(
            j.ajouter(niveau_2, &r, &regles),
            Err(ErreurJournal::Choix { .. })
        ));
        // annuler revient à l'état d'avant
        j.annuler_dernier();
        assert_eq!(j, avant);
    }

    #[test]
    fn sorts_reattribues_refuses_si_hors_regles() {
        let r = Referentiel::core();
        let regles = ReglesFiche::default();
        let mut j = journal();
        let sorts = |l: &[&str]| EvenementFiche::SortsReattribues {
            sorts: l.iter().map(|s| s.to_string()).collect(),
        };
        // un sort de la liste de l'ensorceleur, au bon niveau : accepté
        j.ajouter(sorts(&["Light", "Magic Missile"]), &r, &regles)
            .unwrap();
        let e = j.etat(&r, &regles).unwrap();
        assert_eq!(e.fiche.sorts_connus, ["Light", "Magic Missile"]);
        let avant = j.clone();
        // doublon, sort inconnu, sort de niveau 3 au niveau 1 : refusés
        assert!(j.ajouter(sorts(&["Light", "Light"]), &r, &regles).is_err());
        assert!(j.ajouter(sorts(&["Inconnu"]), &r, &regles).is_err());
        assert!(j.ajouter(sorts(&["Fireball"]), &r, &regles).is_err());
        assert_eq!(j, avant);
    }
}
