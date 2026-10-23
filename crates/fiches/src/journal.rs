//! Journal d'événements d'un personnage : historique complet, annulation, recalcul propre.
//! L'état (fiche, XP, jalons) est toujours recalculé en rejouant le journal.

use crate::donnees::Referentiel;
use crate::etats::EffetTemporaire;
use crate::fiche::Fiche;
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
            EvenementFiche::ArmeAjoutee(a) => format!("Arme prise : {a}"),
            EvenementFiche::ArmeRetiree(a) => format!("Arme rangée : {a}"),
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
                if let Some(total) = e.par_jour {
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
        }
    }
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
                EvenementFiche::Repos => sorts_depenses.clear(),
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
                        .and_then(|e| e.par_jour)
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
                    match emplacement {
                        EmplacementProtection::Armure => fiche.equipement.armure = objet.clone(),
                        EmplacementProtection::Bouclier => {
                            fiche.equipement.bouclier = objet.clone()
                        }
                    }
                    verifier_equipement(r, &fiche, i)?;
                }
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
}
