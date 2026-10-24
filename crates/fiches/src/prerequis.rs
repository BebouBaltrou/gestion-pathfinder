//! Moteur de prérequis de dons : conjonctions (tous les termes) et disjonctions (« ou »).

use crate::caracteristiques::Carac;
use crate::catalogue::{nom_de_base, Don, Prerequis};
use crate::donnees::Referentiel;
use crate::fiche::{Fiche, Valeurs};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Statut {
    Rempli,
    /// Prérequis non satisfait, avec ce qui manque.
    Manquant(String),
    /// Prérequis que le moteur ne sait pas évaluer : à vérifier par le joueur.
    AVerifier(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Verification {
    /// Chaque prérequis du don, en français, avec son statut.
    pub details: Vec<(String, Statut)>,
}

impl Verification {
    /// Tous les prérequis évaluables sont remplis (les « à vérifier » ne bloquent pas).
    pub fn rempli(&self) -> bool {
        !self
            .details
            .iter()
            .any(|(_, s)| matches!(s, Statut::Manquant(_)))
    }

    pub fn a_verifier(&self) -> bool {
        self.details
            .iter()
            .any(|(_, s)| matches!(s, Statut::AVerifier(_)))
    }
}

/// Situation du personnage au moment de la vérification.
pub struct Contexte<'a> {
    pub referentiel: &'a Referentiel,
    pub fiche: &'a Fiche,
    pub valeurs: &'a Valeurs,
    /// Dons déjà possédés (noms anglais, avec leur choix éventuel).
    pub dons: &'a [String],
}

fn carac_depuis_id(id: &str) -> Option<Carac> {
    Carac::TOUTES
        .into_iter()
        .find(|c| c.abrege().eq_ignore_ascii_case(id))
}

fn rangs_de(fiche: &Fiche, competence: &str, sous_type: Option<&str>) -> u32 {
    let mut total: BTreeMap<&str, u32> = BTreeMap::new();
    for n in &fiche.niveaux {
        for (cle, v) in &n.rangs {
            *total.entry(cle.as_str()).or_default() += v;
        }
    }
    total
        .iter()
        .filter(|(cle, _)| {
            let (base, sous) = match cle.split_once(':') {
                Some((b, s)) => (b, Some(s)),
                None => (**cle, None),
            };
            base == competence && sous_type.is_none_or(|s| sous == Some(s))
        })
        .map(|(_, v)| *v)
        .sum()
}

fn evaluer(ctx: &Contexte, p: &Prerequis) -> (String, Statut) {
    match p {
        Prerequis::Caracteristique { carac, min } => {
            let nom = carac_depuis_id(carac)
                .map(|c| c.abrege().to_string())
                .unwrap_or_else(|| carac.to_uppercase());
            let texte = format!("{nom} {min}");
            match carac_depuis_id(carac) {
                Some(c) => {
                    let s = ctx.valeurs.scores[&c].total;
                    if s >= *min {
                        (texte, Statut::Rempli)
                    } else {
                        (texte, Statut::Manquant(format!("{nom} actuel : {s}")))
                    }
                }
                None => (texte.clone(), Statut::AVerifier(texte)),
            }
        }
        Prerequis::Bba { min } => {
            let b = ctx.valeurs.bba.total;
            let t = format!("bonus de base à l'attaque +{min}");
            if b >= *min {
                (t, Statut::Rempli)
            } else {
                (t, Statut::Manquant(format!("BBA actuel : +{b}")))
            }
        }
        Prerequis::NiveauPersonnage { min } => {
            let n = ctx.valeurs.niveau_de_personnage;
            let t = format!("niveau de personnage {min}");
            if n >= *min {
                (t, Statut::Rempli)
            } else {
                (t, Statut::Manquant(format!("niveau actuel : {n}")))
            }
        }
        Prerequis::NiveauLanceur { min } => {
            let n = ctx
                .valeurs
                .incantateurs
                .iter()
                .map(|i| i.niveau_de_lanceur)
                .max()
                .unwrap_or(0);
            let t = format!("niveau de lanceur de sorts {min}");
            if n >= *min {
                (t, Statut::Rempli)
            } else {
                (
                    t,
                    Statut::Manquant(format!("niveau de lanceur actuel : {n}")),
                )
            }
        }
        Prerequis::Rangs {
            competence,
            sous_type,
            min,
        } => {
            let r = rangs_de(ctx.fiche, competence, sous_type.as_deref());
            let nom = competence.replace('_', " ");
            let t = match sous_type {
                Some(s) => format!("{nom} ({s}) : {min} rangs"),
                None => format!("{nom} : {min} rangs"),
            };
            if r >= *min {
                (t, Statut::Rempli)
            } else {
                (t, Statut::Manquant(format!("rangs actuels : {r}")))
            }
        }
        Prerequis::SortsDeNiveau {
            niveau_de_sort,
            genre,
        } => {
            let t = format!("peut lancer des sorts de niveau {niveau_de_sort}");
            let ok = ctx.valeurs.incantateurs.iter().any(|i| {
                let type_ok = match (genre.as_deref(), ctx.referentiel.classe(&i.classe)) {
                    (Some(g), Some(c)) => c
                        .incantation
                        .as_ref()
                        .is_some_and(|inc| (g == "arcane") == (inc.genre == "profane")),
                    _ => true,
                };
                type_ok
                    && i.emplacements
                        .iter()
                        .any(|e| e.niveau_de_sort >= *niveau_de_sort && e.par_jour != Some(0))
            });
            if ok {
                (t, Statut::Rempli)
            } else {
                (t.clone(), Statut::Manquant(t))
            }
        }
        Prerequis::NiveauDeClasse { classe, min } => {
            let id = ctx
                .referentiel
                .classes
                .iter()
                .find(|c| c.nom_en.to_lowercase() == *classe)
                .map(|c| c.id.clone());
            let t = format!("niveau de {classe} {min}");
            match id {
                Some(id) => {
                    let n = ctx.fiche.niveau_de_classe(&id);
                    if n >= *min {
                        (t, Statut::Rempli)
                    } else {
                        (t, Statut::Manquant(format!("niveau actuel : {n}")))
                    }
                }
                // Classe absente du référentiel (pas encore importée) : 0 niveau.
                None => (t.clone(), Statut::Manquant(t)),
            }
        }
        Prerequis::Don { nom_en } => {
            let t = format!("don {nom_en}");
            if ctx
                .dons
                .iter()
                .any(|d| nom_de_base(d).eq_ignore_ascii_case(nom_de_base(nom_en)))
            {
                (t, Statut::Rempli)
            } else {
                (t.clone(), Statut::Manquant(t))
            }
        }
        Prerequis::Ou { termes } => {
            let evals: Vec<_> = termes.iter().map(|t| evaluer(ctx, t)).collect();
            let libelle = evals
                .iter()
                .map(|(l, _)| l.as_str())
                .collect::<Vec<_>>()
                .join(" ou ");
            if evals.iter().any(|(_, s)| *s == Statut::Rempli) {
                (libelle, Statut::Rempli)
            } else if evals.iter().any(|(_, s)| matches!(s, Statut::AVerifier(_))) {
                (libelle.clone(), Statut::AVerifier(libelle))
            } else {
                (libelle.clone(), Statut::Manquant(libelle))
            }
        }
        Prerequis::Texte { texte_en } => (texte_en.clone(), Statut::AVerifier(texte_en.clone())),
    }
}

/// Vérifie tous les prérequis d'un don.
pub fn verifier(ctx: &Contexte, don: &Don) -> Verification {
    Verification {
        details: don.prerequis.iter().map(|p| evaluer(ctx, p)).collect(),
    }
}

/// Dons de la fiche dont les prérequis ne sont plus remplis (statistiques changées, niveau annulé…).
/// Le don n'est pas perdu : il est inutilisable tant que le prérequis manque (Core p. 111).
pub fn dons_invalides(
    r: &Referentiel,
    fiche: &Fiche,
    valeurs: &Valeurs,
) -> Vec<(String, Verification)> {
    let mut possedes: Vec<String> = fiche.dons_automatiques(r);
    let mut invalides = Vec::new();
    for n in &fiche.niveaux {
        for d in &n.dons {
            if let Some(def) = r.don(nom_de_base(d)) {
                let ctx = Contexte {
                    referentiel: r,
                    fiche,
                    valeurs,
                    dons: &possedes,
                };
                let v = verifier(&ctx, def);
                if !v.rempli() {
                    invalides.push((d.clone(), v));
                }
            }
            possedes.push(d.clone());
        }
    }
    invalides
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalogue::Don;
    use crate::donnees::SourceLivre;

    fn don(prerequis: Vec<Prerequis>) -> Don {
        Don {
            nom_en: "Test".into(),
            types_en: vec![],
            source: SourceLivre {
                livre: "test".into(),
                page: None,
                url: None,
            },
            resume_en: String::new(),
            prerequis_en: None,
            prerequis,
            avantage_en: None,
            normal_en: None,
            special_en: None,
        }
    }

    fn verif(f: &Fiche, d: &Don, dons: &[String]) -> Verification {
        let r = Referentiel::core();
        let v = f.calculer(&r).unwrap();
        verifier(
            &Contexte {
                referentiel: &r,
                fiche: f,
                valeurs: &v,
                dons,
            },
            d,
        )
    }

    #[test]
    fn caracteristique_et_bba() {
        let f = crate::fiche::tests::elfe_ensorceleuse();
        let pa = Referentiel::core().don("Power Attack").unwrap().clone();
        let v = verif(&f, &pa, &[]);
        assert!(!v.rempli()); // FOR 6, BBA +0
        assert_eq!(v.details.len(), 2);
        let dodge = Referentiel::core().don("Dodge").unwrap().clone();
        assert!(verif(&f, &dodge, &[]).rempli()); // DEX 16
    }

    #[test]
    fn disjonction_ou() {
        let f = crate::fiche::tests::elfe_ensorceleuse();
        let ou = Prerequis::Ou {
            termes: vec![
                Prerequis::Caracteristique {
                    carac: "for".into(),
                    min: 13,
                },
                Prerequis::Caracteristique {
                    carac: "dex".into(),
                    min: 13,
                },
            ],
        };
        assert!(verif(&f, &don(vec![ou]), &[]).rempli());
        let ou_non = Prerequis::Ou {
            termes: vec![
                Prerequis::Caracteristique {
                    carac: "for".into(),
                    min: 13,
                },
                Prerequis::Caracteristique {
                    carac: "sag".into(),
                    min: 13,
                },
            ],
        };
        assert!(!verif(&f, &don(vec![ou_non]), &[]).rempli());
    }

    #[test]
    fn prerequis_de_don_et_rangs() {
        let f = crate::fiche::tests::elfe_ensorceleuse();
        let besoin = don(vec![
            Prerequis::Don {
                nom_en: "Great Fortitude".into(),
            },
            Prerequis::Rangs {
                competence: "spellcraft".into(),
                sous_type: None,
                min: 1,
            },
        ]);
        let v = verif(&f, &besoin, &[]);
        assert!(!v.rempli());
        assert_eq!(v.details[1].1, Statut::Rempli); // 1 rang d'Art de la magie
        let v = verif(&f, &besoin, &["Great Fortitude".to_string()]);
        assert!(v.rempli());
    }

    #[test]
    fn prerequis_non_evaluable_est_signale_sans_bloquer() {
        let f = crate::fiche::tests::elfe_ensorceleuse();
        let d = don(vec![Prerequis::Texte {
            texte_en: "Channel energy class feature".into(),
        }]);
        let v = verif(&f, &d, &[]);
        assert!(v.rempli() && v.a_verifier());
    }

    #[test]
    fn niveau_de_lanceur_et_sorts() {
        let f = crate::fiche::tests::elfe_ensorceleuse();
        let d = don(vec![
            Prerequis::NiveauLanceur { min: 1 },
            Prerequis::SortsDeNiveau {
                niveau_de_sort: 1,
                genre: Some("arcane".into()),
            },
        ]);
        assert!(verif(&f, &d, &[]).rempli());
        let d = don(vec![Prerequis::NiveauLanceur { min: 5 }]);
        assert!(!verif(&f, &d, &[]).rempli());
        let d = don(vec![Prerequis::SortsDeNiveau {
            niveau_de_sort: 1,
            genre: Some("divine".into()),
        }]);
        assert!(!verif(&f, &d, &[]).rempli());
    }

    #[test]
    fn un_don_devient_invalide_quand_la_caracteristique_baisse() {
        let r = Referentiel::core();
        let mut f = crate::fiche::tests::elfe_ensorceleuse();
        f.niveaux[0].dons = vec!["Dodge".into()]; // DEX 13 requis
        let v = f.calculer(&r).unwrap();
        assert!(dons_invalides(&r, &f, &v).is_empty());
        f.scores_base.insert(Carac::Dex, 8); // 8 + 2 (elfe) = 10
        let v = f.calculer(&r).unwrap();
        let inv = dons_invalides(&r, &f, &v);
        assert_eq!(inv.len(), 1);
        assert_eq!(inv[0].0, "Dodge");
    }

    #[test]
    fn le_catalogue_core_a_ses_prerequis_structures() {
        let r = Referentiel::core();
        assert_eq!(r.dons.len(), 176);
        // Quelques prérequis connus du livre.
        let imp = r.don("Improved Great Fortitude").unwrap();
        assert_eq!(
            imp.prerequis,
            vec![Prerequis::Don {
                nom_en: "Great Fortitude".into()
            }]
        );
        let gwf = r.don("Greater Weapon Focus").unwrap();
        assert!(gwf.prerequis.iter().any(
            |p| matches!(p, Prerequis::NiveauDeClasse { classe, min: 8 } if classe == "fighter")
        ));
        assert!(r.don("Weapon Focus").unwrap().repetable());
        assert!(!r.don("Dodge").unwrap().repetable());
    }
}
