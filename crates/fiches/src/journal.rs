//! Journal d'événements d'un personnage : historique complet, annulation, recalcul propre.
//! L'état (fiche, XP, jalons) est toujours recalculé en rejouant le journal.

use crate::donnees::Referentiel;
use crate::fiche::Fiche;
use crate::progression::{appliquer, niveau_atteignable, ErreurMontee, Montee, ReglesFiche};

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
}

impl EtatPersonnage {
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
        }
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
    use crate::fiche::BonusPredilection;
    use std::collections::BTreeMap;

    fn niveau_simple() -> Montee {
        Montee {
            classe: "ensorceleur".into(),
            de_vie: 4,
            rangs: BTreeMap::new(),
            predilection: Some(BonusPredilection::PointDeVie),
            dons: vec![],
            augmentation: None,
        }
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
}
