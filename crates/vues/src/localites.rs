//! Page Localités : liste, fiche, création et modification, opérations sur la trésorerie
//! et les capitaux (injection, retrait, transfert, ajustement).
//!
//! Les saisies de l'interface arrivent en texte et sont lues ici : l'interface ne
//! connaît ni les règles ni le format des montants.

use crate::{Capitaux, Somme};
use campagne::evenements::Evenement;
use campagne::modele::{CategorieLocalite, IdLocalite, IdPersonnage, IdTransfert, Localite, Lot};
use domaine::{CoutCapitaux, Montant, TypeCapital};
use regles::Referentiel;

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct PageLocalites {
    pub localites: Vec<FicheLocalite>,
    /// Personnages pour l'injection et le retrait : « Seoni (145 po) ».
    pub personnages: Vec<Choix>,
    /// Catégories officielles de communauté, dans l'ordre du livre.
    pub categories: Vec<ChoixCategorie>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Choix {
    pub id: u32,
    pub libelle: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct ChoixCategorie {
    pub id: String,
    /// « Village (61 à 200 hab.) — 4/jour »
    pub libelle: String,
    pub nom: String,
    pub limite: u32,
    /// Sources de la tranche de population et de la limite de dépense.
    pub source_population: String,
    pub source_limite: String,
    pub point_a_confirmer: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct FicheLocalite {
    pub id: u32,
    pub nom: String,
    pub genre: String,
    pub population: Option<u32>,
    /// « Petite bourgade · limite 15/jour »
    pub categorie: String,
    pub personnalisee: bool,
    pub tresorerie: Somme,
    pub capitaux: Capitaux,
    /// Transferts en route depuis ou vers cette localité.
    pub transferts: Vec<String>,
    pub notes: String,
    /// Valeurs actuelles, pour préremplir le formulaire de modification.
    pub saisie: SaisieLocalite,
}

/// Formulaire de création ou de modification, tel que saisi.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct SaisieLocalite {
    pub nom: String,
    pub genre: String,
    pub population: String,
    /// Catégorie officielle (sinon personnalisée).
    pub officielle: bool,
    pub categorie_officielle: String,
    pub categorie_perso_nom: String,
    pub categorie_perso_limite: String,
    pub notes: String,
}

impl SaisieLocalite {
    fn depuis(l: &Localite) -> Self {
        let mut s = SaisieLocalite {
            nom: l.nom.clone(),
            genre: l.genre.clone(),
            population: l.population.map(|p| p.to_string()).unwrap_or_default(),
            notes: l.notes.clone(),
            ..SaisieLocalite::default()
        };
        match &l.categorie {
            CategorieLocalite::Officielle { id } => {
                s.officielle = true;
                s.categorie_officielle = id.clone();
            }
            CategorieLocalite::Personnalisee {
                nom,
                limite_depense_jour,
            } => {
                s.categorie_perso_nom = nom.clone();
                s.categorie_perso_limite = limite_depense_jour.to_string();
            }
        }
        s
    }

    fn population(&self) -> Result<Option<u32>, String> {
        let t: String = self
            .population
            .chars()
            .filter(|c| !c.is_whitespace())
            .collect();
        if t.is_empty() {
            return Ok(None);
        }
        t.parse()
            .map(Some)
            .map_err(|_| "La population doit être un nombre entier".to_string())
    }

    fn categorie(&self, r: &Referentiel) -> Result<CategorieLocalite, String> {
        if self.officielle {
            if r.communaute(&self.categorie_officielle).is_none() {
                return Err("Choisissez une catégorie de communauté".into());
            }
            return Ok(CategorieLocalite::Officielle {
                id: self.categorie_officielle.clone(),
            });
        }
        let limite =
            self.categorie_perso_limite.trim().parse().map_err(|_| {
                "Indiquez la limite de dépense par jour (nombre entier)".to_string()
            })?;
        let nom = self.categorie_perso_nom.trim();
        Ok(CategorieLocalite::Personnalisee {
            nom: if nom.is_empty() {
                "Personnalisée".into()
            } else {
                nom.into()
            },
            limite_depense_jour: limite,
        })
    }

    /// Événement de création (`id` : `Campagne::nouvel_id_localite`).
    pub fn creation(&self, id: IdLocalite, r: &Referentiel) -> Result<Evenement, String> {
        let (nom, genre, population, categorie, notes) = self.lire(r)?;
        Ok(Evenement::LocaliteCreee {
            id,
            nom,
            genre,
            population,
            categorie,
            notes,
        })
    }

    pub fn modification(&self, id: IdLocalite, r: &Referentiel) -> Result<Evenement, String> {
        let (nom, genre, population, categorie, notes) = self.lire(r)?;
        Ok(Evenement::LocaliteModifiee {
            id,
            nom,
            genre,
            population,
            categorie,
            notes,
        })
    }

    #[allow(clippy::type_complexity)]
    fn lire(
        &self,
        r: &Referentiel,
    ) -> Result<(String, String, Option<u32>, CategorieLocalite, String), String> {
        let nom = self.nom.trim();
        if nom.is_empty() {
            return Err("Donnez un nom à la localité".into());
        }
        Ok((
            nom.into(),
            self.genre.trim().into(),
            self.population()?,
            self.categorie(r)?,
            self.notes.clone(),
        ))
    }
}

pub fn page_localites(c: &mut campagne::Campagne, r: &Referentiel) -> PageLocalites {
    let etat = c.etat(r).clone();
    let cal = &c.calendrier;
    let localites = etat
        .localites
        .values()
        .map(|l| FicheLocalite {
            id: l.id.0,
            nom: l.nom.clone(),
            genre: l.genre.clone(),
            population: l.population,
            categorie: crate::tableau::libelle_categorie(&l.categorie, r),
            personnalisee: matches!(l.categorie, CategorieLocalite::Personnalisee { .. }),
            tresorerie: l.tresorerie.into(),
            capitaux: l.capitaux.into(),
            transferts: etat
                .transferts
                .iter()
                .filter(|t| t.vers == l.id || t.de == l.id)
                .map(|t| {
                    let (sens, autre) = if t.vers == l.id {
                        ("arrive de", t.de)
                    } else {
                        ("part vers", t.vers)
                    };
                    let nom = etat
                        .localites
                        .get(&autre)
                        .map(|x| x.nom.as_str())
                        .unwrap_or("?");
                    format!(
                        "{} {sens} {nom} — arrivée le jour {} ({})",
                        t.lot,
                        t.jour_arrivee,
                        cal.libelle(t.jour_arrivee)
                    )
                })
                .collect(),
            notes: l.notes.clone(),
            saisie: SaisieLocalite::depuis(l),
        })
        .collect();
    PageLocalites {
        localites,
        personnages: etat
            .personnages
            .values()
            .map(|p| Choix {
                id: p.id.0,
                libelle: format!("{} ({})", p.nom, p.compte),
            })
            .collect(),
        categories: r
            .communautes
            .iter()
            .map(|c| choix_categorie(c, r))
            .collect(),
    }
}

fn choix_categorie(c: &domaine::fiches::CategorieCommunaute, r: &Referentiel) -> ChoixCategorie {
    let pop = match c.population_max {
        Some(m) => format!("{} à {}", c.population_min, m),
        None => format!("plus de {}", c.population_min.saturating_sub(1)),
    };
    ChoixCategorie {
        id: c.id.clone(),
        libelle: format!("{} ({pop} hab.) — {}/jour", c.nom, c.limite_depense_jour),
        nom: c.nom.clone(),
        limite: c.limite_depense_jour,
        source_population: r.reference(&c.source_population),
        source_limite: r.reference(&c.source_limite),
        point_a_confirmer: c.point_a_confirmer.clone(),
    }
}

/// Catégorie officielle correspondant à une population saisie.
pub fn categorie_suggeree(population: &str, r: &Referentiel) -> Option<ChoixCategorie> {
    let s = SaisieLocalite {
        population: population.into(),
        ..SaisieLocalite::default()
    };
    let p = s.population().ok()??;
    r.categorie_pour_population(p)
        .map(|c| choix_categorie(c, r))
}

/// Lecture d'un montant saisi en po (« 12 », « 12,5 », « 4 po 7 pa », « 7 pa »).
pub fn lire_montant(texte: &str) -> Option<Somme> {
    Montant::depuis_texte(texte).map(Somme::from)
}

fn montant_positif(texte: &str) -> Result<Montant, String> {
    match Montant::depuis_texte(texte) {
        Some(m) if m.pa > 0 => Ok(m),
        Some(_) => Err("Le montant doit être positif".into()),
        None => Err("Montant illisible : écrivez par exemple 12, 12,5 ou 4 po 7 pa".into()),
    }
}

/// Injection (compte du PJ → trésorerie) ou retrait (trésorerie → compte du PJ).
pub fn mouvement_po(
    injection: bool,
    pj: u32,
    localite: u32,
    montant: &str,
) -> Result<Evenement, String> {
    let (pj, localite, montant) = (
        IdPersonnage(pj),
        IdLocalite(localite),
        montant_positif(montant)?,
    );
    Ok(if injection {
        Evenement::InjectionPo {
            pj,
            localite,
            montant,
        }
    } else {
        Evenement::RetraitPo {
            localite,
            pj,
            montant,
        }
    })
}

/// Transfert vers une autre localité. `ressource` : « po », « marchandises », « travail »
/// ou « magie » (l'Influence ne se déplace pas, GC p. 81).
pub fn transfert(
    id: IdTransfert,
    de: u32,
    vers: u32,
    ressource: &str,
    quantite: &str,
    duree: &str,
) -> Result<Evenement, String> {
    let lot = match ressource {
        "po" => Lot::Po(montant_positif(quantite)?),
        autre => {
            let capital = match autre {
                "marchandises" => TypeCapital::Marchandises,
                "travail" => TypeCapital::Travail,
                "magie" => TypeCapital::Magie,
                "influence" => return Err(
                    "L'Influence est attachée à sa communauté et ne se transfère pas (GC p. 81)"
                        .into(),
                ),
                _ => return Err("Choisissez ce qui est transféré".into()),
            };
            match quantite.trim().parse::<u32>() {
                Ok(n) if n > 0 => Lot::Capital(capital, n),
                _ => return Err("La quantité doit être un nombre entier positif".into()),
            }
        }
    };
    let duree_jours = duree
        .trim()
        .parse()
        .map_err(|_| "Indiquez la durée du trajet en jours (0 = immédiat)".to_string())?;
    if de == vers {
        return Err("Choisissez une autre localité de destination".into());
    }
    Ok(Evenement::TransfertEnvoye {
        id,
        de: IdLocalite(de),
        vers: IdLocalite(vers),
        lot,
        duree_jours,
    })
}

/// Ajustement libre (récompense du MJ, correction…), toujours avec un motif.
pub fn ajustement(
    localite: u32,
    delta_po: &str,
    ajout: Capitaux,
    retrait: Capitaux,
    motif: &str,
) -> Result<Evenement, String> {
    let delta = if delta_po.trim().is_empty() {
        Montant::ZERO
    } else {
        Montant::depuis_texte(delta_po)
            .ok_or("Montant illisible : écrivez par exemple -12, 12,5 ou 4 po 7 pa")?
    };
    let (ajout, retrait): (CoutCapitaux, CoutCapitaux) = (ajout.into(), retrait.into());
    if delta.pa == 0 && ajout.unites() + retrait.unites() == 0 {
        return Err("Rien à ajuster".into());
    }
    if motif.trim().is_empty() {
        return Err("Indiquez le motif de l'ajustement".into());
    }
    Ok(Evenement::AjustementLocalite {
        localite: IdLocalite(localite),
        delta_po: delta,
        ajout,
        retrait,
        motif: motif.trim().into(),
    })
}

/// Texte de confirmation avant de supprimer une localité : ce qui disparaît avec elle.
pub fn avertissement_suppression(c: &mut campagne::Campagne, r: &Referentiel, id: u32) -> String {
    let etat = c.etat(r);
    let id = IdLocalite(id);
    let Ok(l) = etat.localite(id) else {
        return "Cette localité n'existe plus.".into();
    };
    let k = etat.contenu_localite(id);
    let mut perdu = Vec::new();
    let mut ajoute = |n: usize, un: &str, plusieurs: &str| {
        if n > 0 {
            perdu.push(format!("{n} {}", if n == 1 { un } else { plusieurs }));
        }
    };
    ajoute(k.batiments, "bâtiment", "bâtiments");
    ajoute(k.organisations, "organisation", "organisations");
    ajoute(k.equipes, "équipe", "équipes");
    ajoute(k.gestionnaires, "gestionnaire", "gestionnaires");
    ajoute(k.transferts, "transfert en route", "transferts en route");
    ajoute(k.ventes, "vente en cours", "ventes en cours");
    let mut t = format!("Supprimer la localité « {} » ?", l.nom);
    if perdu.is_empty() {
        t.push_str("\n\nElle est vide.");
    } else {
        t.push_str(&format!(
            "\n\nDisparaissent avec elle : {}.",
            perdu.join(", ")
        ));
    }
    t.push_str(&format!(
        "\nSa trésorerie ({}) et ses capitaux sont perdus.",
        l.tresorerie
    ));
    if k.personnages_presents > 0 {
        t.push_str(&format!(
            "\n{} personnage(s) présent(s) sont conservés, sans localité.",
            k.personnages_presents
        ));
    }
    t.push_str("\n\nVous pourrez annuler juste après avec le bouton « Annuler ».");
    t
}

#[cfg(test)]
mod tests {
    use super::*;
    use campagne::Campagne;

    fn saisie(nom: &str, population: &str) -> SaisieLocalite {
        SaisieLocalite {
            nom: nom.into(),
            genre: "ville".into(),
            population: population.into(),
            officielle: true,
            categorie_officielle: "petite-bourgade".into(),
            ..SaisieLocalite::default()
        }
    }

    #[test]
    fn avertissement_avant_suppression() {
        let r = regles::referentiel_officiel();
        let mut c = Campagne::nouvelle("Test");
        let id = c.nouvel_id_localite();
        c.executer(saisie("Sandpoint", "1 240").creation(id, &r).unwrap(), &r)
            .unwrap();
        let t = avertissement_suppression(&mut c, &r, id.0);
        assert!(t.contains("« Sandpoint »") && t.contains("Elle est vide"));
        c.executer(Evenement::LocaliteSupprimee { id }, &r).unwrap();
        assert!(avertissement_suppression(&mut c, &r, id.0).contains("n'existe plus"));
    }

    #[test]
    fn creation_operations_et_fiche() {
        let r = regles::referentiel_officiel();
        let mut c = Campagne::nouvelle("Test");
        let pj = c.nouvel_id_personnage();
        c.executer(
            Evenement::PersonnageCree {
                id: pj,
                nom: "Seoni".into(),
                compte_initial: Montant::po(50),
                prestige: None,
            },
            &r,
        )
        .unwrap();
        for nom in ["Sandpoint", "Magnimar"] {
            let id = c.nouvel_id_localite();
            let ev = saisie(nom, "1 240").creation(id, &r).unwrap();
            c.executer(ev, &r).unwrap();
        }
        c.executer(mouvement_po(true, pj.0, 1, "12,5").unwrap(), &r)
            .unwrap();
        let t = transfert(c.nouvel_id_transfert(), 1, 2, "po", "2 po", "3").unwrap();
        c.executer(t, &r).unwrap();
        let a = ajustement(
            1,
            "",
            Capitaux {
                marchandises: 3,
                ..Capitaux::default()
            },
            Capitaux::default(),
            "récompense",
        )
        .unwrap();
        c.executer(a, &r).unwrap();

        let p = page_localites(&mut c, &r);
        assert_eq!(p.localites.len(), 2);
        let s = &p.localites[0];
        assert_eq!(s.population, Some(1240));
        assert_eq!(s.tresorerie.pa, 105);
        assert_eq!(s.capitaux.marchandises, 3);
        assert_eq!(s.transferts.len(), 1);
        assert!(s.transferts[0].contains("part vers Magnimar"));
        assert_eq!(s.saisie.categorie_officielle, "petite-bourgade");
        assert_eq!(p.personnages[0].libelle, "Seoni (37 po 5 pa)");
        assert_eq!(p.categories.len(), r.communautes.len());
    }

    #[test]
    fn saisies_refusees() {
        let r = regles::referentiel_officiel();
        let id = IdLocalite(1);
        assert!(saisie("", "").creation(id, &r).is_err());
        assert!(saisie("A", "beaucoup").creation(id, &r).is_err());
        let mut s = saisie("A", "");
        s.officielle = false;
        assert!(s.creation(id, &r).is_err()); // limite manquante
        s.categorie_perso_limite = "3".into();
        assert!(s.creation(id, &r).is_ok());
        assert!(mouvement_po(true, 1, 1, "0").is_err());
        assert!(mouvement_po(false, 1, 1, "abc").is_err());
        assert!(transfert(IdTransfert(1), 1, 2, "influence", "1", "0").is_err());
        assert!(transfert(IdTransfert(1), 1, 1, "magie", "1", "0").is_err());
        assert!(transfert(IdTransfert(1), 1, 2, "magie", "1", "").is_err());
        assert!(ajustement(1, "", Capitaux::default(), Capitaux::default(), "x").is_err());
        assert!(ajustement(1, "-3", Capitaux::default(), Capitaux::default(), " ").is_err());
    }

    #[test]
    fn categorie_selon_la_population() {
        let r = regles::referentiel_officiel();
        let attendu = r.categorie_pour_population(1240).map(|c| c.id.clone());
        assert_eq!(categorie_suggeree("1 240", &r).map(|c| c.id), attendu);
        assert!(categorie_suggeree("", &r).is_none());
        assert_eq!(lire_montant("4 po 7 pa").map(|s| s.pa), Some(47));
    }
}
