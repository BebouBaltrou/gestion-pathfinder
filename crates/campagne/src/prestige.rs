//! Le don Prestige (VF du don *Leadership* du Core, p. 129) : un acolyte (compagnon d'armes, avec sa
//! propre fiche et son XP) et des partisans (simples effectifs par niveau, sans fiche ni XP).
//!
//! Source : https://www.pathfinder-fr.org/Wiki/Pathfinder-RPG.Prestige.ashx ; le tableau est aussi
//! dans `data/core/fr/dons_04.json`. Proposition validée : `docs/PRESTIGE_DON.md`.

use crate::etat::{Erreur, Etat};
use crate::modele::IdPersonnage;
use fiches::journal::EvenementFiche;
use fiches::progression::{xp_requis, Vitesse};
use fiches::Referentiel as ReferentielFiches;
use std::collections::BTreeMap;

/// Nom (anglais, comme dans le catalogue des dons) du don Prestige.
pub const DON: &str = "Leadership";

/// Niveau minimal du personnage pour prendre le don.
pub const NIVEAU_MINIMAL: u32 = 7;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "lowercase"))]
pub enum Portee {
    /// S'applique au score de l'acolyte comme à celui des partisans (réputation).
    Tous,
    Acolyte,
    Partisans,
}

/// Modificateur du score de Prestige du don.
pub struct Modificateur {
    pub id: &'static str,
    pub libelle: &'static str,
    pub valeur: i32,
    pub portee: Portee,
    /// Peut être compté plusieurs fois (« cumulatif par acolyte tué »).
    pub cumulable: bool,
}

const fn m(
    id: &'static str,
    libelle: &'static str,
    valeur: i32,
    portee: Portee,
    cumulable: bool,
) -> Modificateur {
    Modificateur {
        id,
        libelle,
        valeur,
        portee,
        cumulable,
    }
}

/// Réputation (lecture à valider : les facteurs de réputation se cumulent), puis facteurs propres à
/// l'acolyte et aux partisans, d'après le texte du don.
pub const MODIFICATEURS: &[Modificateur] = &[
    m("renommee", "Grande renommée", 2, Portee::Tous, false),
    m("equite", "Juste et généreux", 1, Portee::Tous, false),
    m("pouvoir", "Pouvoir spécial", 1, Portee::Tous, false),
    m("echecs", "Échecs répétés", -1, Portee::Tous, false),
    m("arrogance", "Arrogant", -1, Portee::Tous, false),
    m("cruaute", "Cruel", -2, Portee::Tous, false),
    m(
        "compagnon_animal",
        "A un familier, une monture spéciale ou un compagnon animal",
        -2,
        Portee::Acolyte,
        false,
    ),
    m(
        "alignement_different",
        "Recrute un acolyte d'un autre alignement",
        -1,
        Portee::Acolyte,
        false,
    ),
    m(
        "morts_acolytes",
        "A causé la mort d'un acolyte (par acolyte)",
        -2,
        Portee::Acolyte,
        true,
    ),
    m(
        "forteresse",
        "A une forteresse, une base, une guilde…",
        2,
        Portee::Partisans,
        false,
    ),
    m(
        "deplacements",
        "Se déplace beaucoup",
        -1,
        Portee::Partisans,
        false,
    ),
    m(
        "morts_partisans",
        "A causé la mort d'autres partisans",
        -1,
        Portee::Partisans,
        false,
    ),
];

/// Ligne du tableau « Prestige » : niveau maximal de l'acolyte et partisans par niveau (1er à 6e).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct LigneTableau {
    pub niveau_acolyte: u32,
    pub partisans: [u32; 6],
}

const fn t(niveau_acolyte: u32, partisans: [u32; 6]) -> LigneTableau {
    LigneTableau {
        niveau_acolyte,
        partisans,
    }
}

/// Tableau du don, scores 1 (« 1 ou moins ») à 25 (« 25 ou plus »).
const TABLEAU: [LigneTableau; 25] = [
    t(0, [0, 0, 0, 0, 0, 0]),
    t(1, [0, 0, 0, 0, 0, 0]),
    t(2, [0, 0, 0, 0, 0, 0]),
    t(3, [0, 0, 0, 0, 0, 0]),
    t(3, [0, 0, 0, 0, 0, 0]),
    t(4, [0, 0, 0, 0, 0, 0]),
    t(5, [0, 0, 0, 0, 0, 0]),
    t(5, [0, 0, 0, 0, 0, 0]),
    t(6, [0, 0, 0, 0, 0, 0]),
    t(7, [5, 0, 0, 0, 0, 0]),
    t(7, [6, 0, 0, 0, 0, 0]),
    t(8, [8, 0, 0, 0, 0, 0]),
    t(9, [10, 1, 0, 0, 0, 0]),
    t(10, [15, 1, 0, 0, 0, 0]),
    t(10, [20, 2, 1, 0, 0, 0]),
    t(11, [25, 2, 1, 0, 0, 0]),
    t(12, [30, 3, 1, 1, 0, 0]),
    t(12, [35, 3, 1, 1, 0, 0]),
    t(13, [40, 4, 2, 1, 1, 0]),
    t(14, [50, 5, 3, 2, 1, 0]),
    t(15, [60, 6, 3, 2, 1, 1]),
    t(15, [75, 7, 4, 2, 2, 1]),
    t(16, [90, 9, 5, 3, 2, 1]),
    t(17, [110, 11, 6, 3, 2, 1]),
    t(17, [135, 13, 7, 4, 2, 2]),
];

/// Ligne du tableau pour un score (borné à 1..=25).
pub fn ligne_du_tableau(score: i32) -> LigneTableau {
    TABLEAU[(score.clamp(1, 25) - 1) as usize]
}

/// Un partisan : effectif sans fiche ni XP.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Partisan {
    pub niveau: u32,
    pub nom: String,
}

/// État du don pour un personnage : modificateurs cochés et partisans.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PrestigeDon {
    /// Identifiant du modificateur → nombre de fois (0 ou 1, sauf modificateurs cumulables).
    pub modificateurs: BTreeMap<String, u32>,
    pub partisans: Vec<Partisan>,
}

fn modificateur(id: &str) -> Option<&'static Modificateur> {
    MODIFICATEURS.iter().find(|m| m.id == id)
}

impl Etat {
    /// Le personnage possède-t-il le don (d'après sa fiche) ?
    pub fn a_le_don_prestige(&self, pj: IdPersonnage) -> bool {
        self.fiches.get(&pj).is_some_and(|j| {
            j.etat(
                ReferentielFiches::partage(),
                &self.regles_personnages.fiche(),
            )
            .is_ok_and(|e| {
                e.fiche
                    .niveaux
                    .iter()
                    .any(|n| n.dons.iter().any(|d| d == DON))
            })
        })
    }

    /// Acolyte du personnage, s'il en a un.
    pub fn acolyte_de(&self, maitre: IdPersonnage) -> Option<IdPersonnage> {
        self.acolytes
            .iter()
            .find(|(_, m)| **m == maitre)
            .map(|(a, _)| *a)
    }

    /// Niveau du personnage et modificateur de Charisme (base du score).
    fn niveau_et_charisme(&self, pj: IdPersonnage) -> Option<(u32, i32)> {
        let e = self.fiche_de(pj)?;
        let v = e.fiche.calculer(ReferentielFiches::partage()).ok()?;
        Some((v.niveau_de_personnage, v.modificateurs[&fiches::Carac::Cha]))
    }

    /// Score de Prestige du don : niveau + modificateur de Charisme + modificateurs cochés qui
    /// s'appliquent (`acolyte` : facteurs de l'acolyte, sinon ceux des partisans).
    pub fn score_prestige_don(&self, pj: IdPersonnage, acolyte: bool) -> Option<i32> {
        let (niveau, cha) = self.niveau_et_charisme(pj)?;
        let coches = self.prestige_don.get(&pj);
        let somme: i32 = MODIFICATEURS
            .iter()
            .filter(|m| match m.portee {
                Portee::Tous => true,
                Portee::Acolyte => acolyte,
                Portee::Partisans => !acolyte,
            })
            .map(|m| {
                m.valeur
                    * coches
                        .and_then(|c| c.modificateurs.get(m.id))
                        .copied()
                        .unwrap_or(0) as i32
            })
            .sum();
        Some(niveau as i32 + cha + somme)
    }

    /// Niveau maximal de l'acolyte : le tableau, et au moins deux niveaux sous le maître.
    pub fn plafond_acolyte(&self, maitre: IdPersonnage) -> Option<u32> {
        let score = self.score_prestige_don(maitre, true)?;
        let (niveau, _) = self.niveau_et_charisme(maitre)?;
        Some(
            ligne_du_tableau(score)
                .niveau_acolyte
                .min(niveau.saturating_sub(2)),
        )
    }

    /// Partisans permis par niveau (1er à 6e).
    pub fn places_de_partisans(&self, maitre: IdPersonnage) -> [u32; 6] {
        self.score_prestige_don(maitre, false)
            .map(|s| ligne_du_tableau(s).partisans)
            .unwrap_or([0; 6])
    }

    fn exiger_le_don(&self, pj: IdPersonnage) -> Result<(), Erreur> {
        self.pj(pj)?;
        if !self.a_le_don_prestige(pj) {
            return Err(Erreur::Prestige(
                "ce personnage n'a pas le don Prestige".into(),
            ));
        }
        Ok(())
    }

    /// Événements du don : modificateurs, acolyte, partisans.
    pub(crate) fn appliquer_prestige(
        &mut self,
        ev: &crate::evenements::Evenement,
    ) -> Result<(), Erreur> {
        use crate::evenements::Evenement::*;
        match ev {
            PrestigeDonRegle { pj, modificateurs } => {
                self.exiger_le_don(*pj)?;
                for (id, n) in modificateurs {
                    let m = modificateur(id)
                        .ok_or_else(|| Erreur::Prestige(format!("modificateur inconnu : {id}")))?;
                    if *n > 1 && !m.cumulable {
                        return Err(Erreur::Prestige(format!(
                            "« {} » ne se compte qu'une fois",
                            m.libelle
                        )));
                    }
                }
                let d = self.prestige_don.entry(*pj).or_default();
                d.modificateurs = modificateurs
                    .iter()
                    .filter(|(_, n)| **n > 0)
                    .map(|(k, n)| (k.clone(), *n))
                    .collect();
            }
            AcolyteLie { acolyte, maitre } => {
                self.exiger_le_don(*maitre)?;
                self.pj(*acolyte)?;
                if acolyte == maitre {
                    return Err(Erreur::Prestige(
                        "un personnage ne peut pas être son propre acolyte".into(),
                    ));
                }
                if self.acolytes.contains_key(acolyte) || self.prestige_don_a_un_acolyte(*acolyte) {
                    return Err(Erreur::Prestige(
                        "ce personnage est déjà acolyte, ou a lui-même un acolyte".into(),
                    ));
                }
                if self.acolyte_de(*maitre).is_some() {
                    return Err(Erreur::Prestige(
                        "ce personnage a déjà un acolyte : il faut d'abord le délier".into(),
                    ));
                }
                if self.a_le_don_prestige(*acolyte) {
                    return Err(Erreur::Prestige(
                        "un acolyte ne peut pas avoir le don Prestige".into(),
                    ));
                }
                let plafond = self.plafond_acolyte(*maitre).unwrap_or(0);
                if plafond == 0 {
                    return Err(Erreur::Prestige(
                        "le score de Prestige actuel ne permet pas d'avoir un acolyte".into(),
                    ));
                }
                self.acolytes.insert(*acolyte, *maitre);
            }
            AcolyteDelie { acolyte } => {
                if self.acolytes.remove(acolyte).is_none() {
                    return Err(Erreur::Prestige(
                        "ce personnage n'est pas un acolyte".into(),
                    ));
                }
            }
            PartisanAjoute {
                maitre,
                niveau,
                nom,
            } => {
                self.exiger_le_don(*maitre)?;
                if !(1..=6).contains(niveau) {
                    return Err(Erreur::Prestige(
                        "le niveau d'un partisan va de 1 à 6".into(),
                    ));
                }
                let permis = self.places_de_partisans(*maitre)[(*niveau - 1) as usize];
                let deja = self
                    .prestige_don
                    .get(maitre)
                    .map(|d| d.partisans.iter().filter(|p| p.niveau == *niveau).count())
                    .unwrap_or(0) as u32;
                if deja >= permis {
                    return Err(Erreur::Prestige(format!(
                        "le score de Prestige permet {permis} partisan(s) de niveau {niveau}, \
                         tous sont déjà recrutés"
                    )));
                }
                self.prestige_don
                    .entry(*maitre)
                    .or_default()
                    .partisans
                    .push(Partisan {
                        niveau: *niveau,
                        nom: nom.clone(),
                    });
            }
            PartisanRetire { maitre, index } => {
                let d = self
                    .prestige_don
                    .get_mut(maitre)
                    .filter(|d| *index < d.partisans.len())
                    .ok_or_else(|| Erreur::Prestige("partisan inconnu".into()))?;
                d.partisans.remove(*index);
            }
            _ => {}
        }
        Ok(())
    }

    fn prestige_don_a_un_acolyte(&self, pj: IdPersonnage) -> bool {
        self.acolyte_de(pj).is_some()
    }

    /// Contrôles propres à un acolyte avant d'appliquer un événement de sa fiche : pas de don
    /// Prestige, et niveau limité par le tableau et par le niveau du maître.
    pub(crate) fn verifier_fiche_d_acolyte(
        &self,
        pj: IdPersonnage,
        ev: &EvenementFiche,
    ) -> Result<(), Erreur> {
        let Some(maitre) = self.acolytes.get(&pj) else {
            return Ok(());
        };
        let plafond = self.plafond_acolyte(*maitre).unwrap_or(0);
        match ev {
            EvenementFiche::Creation(f) => {
                if f.niveaux.iter().any(|n| n.dons.iter().any(|d| d == DON)) {
                    return Err(Erreur::Prestige(
                        "un acolyte ne peut pas avoir le don Prestige".into(),
                    ));
                }
            }
            EvenementFiche::MonteeDeNiveau(m) => {
                if m.dons.iter().any(|d| d == DON) {
                    return Err(Erreur::Prestige(
                        "un acolyte ne peut pas avoir le don Prestige".into(),
                    ));
                }
                let niveau = self
                    .fiche_de(pj)
                    .map(|e| e.fiche.niveau_de_personnage() + 1)
                    .unwrap_or(1);
                if niveau > plafond {
                    return Err(Erreur::Prestige(format!(
                        "l'acolyte ne peut pas dépasser le niveau {plafond} (tableau du don et au \
                         moins deux niveaux sous son maître)"
                    )));
                }
            }
            _ => {}
        }
        Ok(())
    }

    /// Quand le maître gagne de l'XP, son acolyte reçoit `XP × niveau de l'acolyte ÷ niveau du
    /// maître` (arrondi vers le bas : lecture à valider), sans jamais atteindre le niveau « maître
    /// moins un » : son total s'arrête alors à 1 de moins que l'XP du niveau suivant.
    pub(crate) fn partager_l_xp_avec_l_acolyte(&mut self, maitre: IdPersonnage, nombre: u32) {
        let Some(acolyte) = self.acolyte_de(maitre) else {
            return;
        };
        if self.regles_personnages.fiche().vitesse == Vitesse::Jalons {
            return; // pas de points d'expérience en mode jalons
        }
        let (Some(em), Some(ea)) = (self.fiche_de(maitre), self.fiche_de(acolyte)) else {
            return;
        };
        let (nm, na) = (
            em.fiche.niveau_de_personnage(),
            ea.fiche.niveau_de_personnage(),
        );
        if nm == 0 {
            return;
        }
        let part = (nombre as u64 * na as u64 / nm as u64) as u32;
        let regles = self.regles_personnages.fiche();
        let r = ReferentielFiches::partage();
        let limite = if nm >= 2 {
            xp_requis(r, regles.vitesse, nm - 1).map(|x| x.saturating_sub(1))
        } else {
            None
        };
        let gain = match limite {
            Some(l) => part.min(l.saturating_sub(ea.xp)),
            None => part,
        };
        if gain == 0 {
            return;
        }
        if let Some(j) = self.fiches.get_mut(&acolyte) {
            let mut copie = j.clone();
            if copie
                .ajouter(
                    EvenementFiche::XpGagnes {
                        nombre: gain,
                        motif: "Part de l'acolyte (don Prestige)".into(),
                    },
                    r,
                    &regles,
                )
                .is_ok()
            {
                *j = copie;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn le_tableau_suit_le_don() {
        assert_eq!(ligne_du_tableau(-3).niveau_acolyte, 0);
        assert_eq!(ligne_du_tableau(9).niveau_acolyte, 6);
        assert_eq!(ligne_du_tableau(9).partisans, [0; 6]);
        assert_eq!(ligne_du_tableau(10).partisans, [5, 0, 0, 0, 0, 0]);
        assert_eq!(ligne_du_tableau(19).niveau_acolyte, 13);
        assert_eq!(ligne_du_tableau(19).partisans, [40, 4, 2, 1, 1, 0]);
        assert_eq!(ligne_du_tableau(25).partisans, [135, 13, 7, 4, 2, 2]);
        assert_eq!(ligne_du_tableau(40), ligne_du_tableau(25));
    }
}

#[cfg(test)]
mod essais {
    use super::*;
    use fiches::fiche::{BonusPredilection, Fiche, NiveauPris};

    /// Ensorceleuse elfe de niveau 7 avec le don Prestige.
    pub fn maitre_fiche() -> Fiche {
        let mut f = fiches::exemple::elfe_ensorceleuse();
        let dons: [&[&str]; 6] = [
            &[],
            &["Iron Will"],
            &[],
            &["Great Fortitude"],
            &[],
            &["Leadership", "Power Attack"],
        ];
        for d in dons {
            f.niveaux.push(NiveauPris {
                classe: "ensorceleur".into(),
                de_vie_obtenu: 4,
                rangs: Default::default(),
                predilection: Some(BonusPredilection::PointDeVie),
                dons: d.iter().map(|x| x.to_string()).collect(),
            });
        }
        f.sorts_connus.extend(
            [
                "Daze",
                "Flare",
                "Ray of Frost",
                "Shield",
                "Color Spray",
                "Grease",
                "Invisibility",
                "Scorching Ray",
                "Mirror Image",
                "Fireball",
                "Fly",
            ]
            .map(String::from),
        );
        f
    }

    use crate::evenements::Evenement;
    use crate::Campagne;
    use domaine::Montant;

    fn pj(c: &mut Campagne, r: &regles::Referentiel, nom: &str) -> IdPersonnage {
        let id = c.nouvel_id_personnage();
        c.executer(
            Evenement::PersonnageCree {
                id,
                nom: nom.into(),
                compte_initial: Montant::po(0),
                prestige: None,
            },
            r,
        )
        .unwrap();
        id
    }

    fn fiche(pj: IdPersonnage, f: Fiche) -> Evenement {
        Evenement::Fiche {
            pj,
            evenement: EvenementFiche::Creation(Box::new(f)),
        }
    }

    #[test]
    fn acolyte_partisans_et_xp_partage() {
        let r = regles::referentiel_officiel();
        let mut c = Campagne::nouvelle("Test");
        let maitre = pj(&mut c, &r, "Seoni");
        let acolyte = pj(&mut c, &r, "Kyra");
        // sans fiche ni don : rien
        assert!(c
            .executer(Evenement::AcolyteLie { acolyte, maitre }, &r)
            .is_err());
        c.executer(fiche(maitre, maitre_fiche()), &r).unwrap();
        // niveau 7 + CHA +3 = score 10 : acolyte jusqu'au niveau 7, mais deux niveaux sous le maître = 5
        assert_eq!(c.etat(&r).score_prestige_don(maitre, true), Some(10));
        assert_eq!(c.etat(&r).plafond_acolyte(maitre), Some(5));
        assert_eq!(c.etat(&r).places_de_partisans(maitre), [5, 0, 0, 0, 0, 0]);
        // partisans : 5 de niveau 1, pas un de plus
        for i in 0..5 {
            c.executer(
                Evenement::PartisanAjoute {
                    maitre,
                    niveau: 1,
                    nom: format!("garde {i}"),
                },
                &r,
            )
            .unwrap();
        }
        assert!(c
            .executer(
                Evenement::PartisanAjoute {
                    maitre,
                    niveau: 1,
                    nom: "de trop".into()
                },
                &r
            )
            .is_err());
        assert!(c
            .executer(
                Evenement::PartisanAjoute {
                    maitre,
                    niveau: 2,
                    nom: "niveau 2".into()
                },
                &r
            )
            .is_err());
        // les modificateurs de réputation changent le score (grande renommée +2 → 12 : 8 partisans)
        c.executer(
            Evenement::PrestigeDonRegle {
                pj: maitre,
                modificateurs: [("renommee".to_string(), 1)].into(),
            },
            &r,
        )
        .unwrap();
        assert_eq!(c.etat(&r).score_prestige_don(maitre, true), Some(12));
        assert_eq!(c.etat(&r).places_de_partisans(maitre)[0], 8);
        assert!(c
            .executer(
                Evenement::PrestigeDonRegle {
                    pj: maitre,
                    modificateurs: [("renommee".to_string(), 2)].into()
                },
                &r
            )
            .is_err());
        // l'acolyte
        c.executer(Evenement::AcolyteLie { acolyte, maitre }, &r)
            .unwrap();
        c.executer(fiche(acolyte, fiches::exemple::elfe_ensorceleuse()), &r)
            .unwrap();
        assert!(c.etat(&r).evenement_niveau_de_depart(acolyte, 6).is_err());
        let ev = c
            .etat(&r)
            .evenement_niveau_de_depart(acolyte, 5)
            .unwrap()
            .unwrap();
        c.executer(
            Evenement::Fiche {
                pj: acolyte,
                evenement: ev,
            },
            &r,
        )
        .unwrap();
        let xp_depart = c.etat(&r).fiche_de(acolyte).unwrap().xp;
        // l'acolyte est niveau 1 : il reçoit 7000 × 1 ÷ 7 = 1000 XP quand le maître gagne 7000 XP
        c.executer(
            Evenement::Fiche {
                pj: maitre,
                evenement: EvenementFiche::XpGagnes {
                    nombre: 7000,
                    motif: "essai".into(),
                },
            },
            &r,
        )
        .unwrap();
        assert_eq!(c.etat(&r).fiche_de(acolyte).unwrap().xp, xp_depart + 1000);
        // une seule annulation défait le gain du maître et la part de l'acolyte
        c.annuler().unwrap();
        assert_eq!(c.etat(&r).fiche_de(acolyte).unwrap().xp, xp_depart);
        // un acolyte n'a pas le don Prestige
        let autre = pj(&mut c, &r, "Autre");
        c.executer(
            Evenement::AcolyteLie {
                acolyte: autre,
                maitre,
            },
            &r,
        )
        .unwrap_err(); // un seul acolyte par maître
    }

    #[test]
    fn la_fiche_de_maitre_est_valide() {
        let r = ReferentielFiches::partage();
        let p = maitre_fiche().valider(r);
        assert!(p.is_empty(), "{p:?}");
    }
}
