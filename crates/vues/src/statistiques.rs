//! Statistiques : patrimoine des personnages et des localités, valeur des objets, effectif et
//! valeur des équipes des organisations (prix imprimés, GC). Les valeurs de capitaux suivent la Table 2-1 (GC p. 78),
//! en valeur d'achat.

use crate::inventaire::page_inventaire;
use crate::Somme;
use campagne::equipement_fiche::{personnage_de_ref, PROPRIETAIRE_PERSONNAGE};
use campagne::etat::Etat;
use campagne::modele::*;
use domaine::Montant;
use regles::calculs::valeur_achat;
use regles::Referentiel;

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct PageStatistiques {
    /// Patrimoine total : comptes des PJ, objets des PJ, trésoreries et capitaux des localités,
    /// objets des autres inventaires.
    pub patrimoine_total: Somme,
    pub personnages: Vec<PatrimoinePersonnage>,
    pub total_personnages: Somme,
    pub localites: Vec<PatrimoineLocalite>,
    pub total_localites: Somme,
    pub objets_autres: Somme,
    pub batiments: Vec<PatrimoineBatiment>,
    pub total_batiments: Somme,
    pub organisations: Vec<StatOrganisation>,
    pub effectif_total: u32,
    pub effectif_a_venir_total: u32,
    pub valeur_equipes: Somme,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct PatrimoinePersonnage {
    pub id: u32,
    pub nom: String,
    pub compte: Somme,
    pub objets: Somme,
    /// Valeur des bâtiments dont il est propriétaire.
    pub batiments: Somme,
    pub total: Somme,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct PatrimoineLocalite {
    pub id: u32,
    pub nom: String,
    pub tresorerie: Somme,
    /// Valeur d'achat des capitaux en réserve (Table 2-1, GC p. 78).
    pub capitaux: Somme,
    /// Valeur des bâtiments de la localité.
    pub batiments: Somme,
    pub total: Somme,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct PatrimoineBatiment {
    pub id: u32,
    pub nom: String,
    pub localite: String,
    pub proprietaire: Option<String>,
    /// Salles construites.
    pub salles: u32,
    /// Prix d'achat des salles et extensions construites (référentiel, GC).
    pub valeur: Somme,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct StatOrganisation {
    pub id: u32,
    pub nom: String,
    pub localite: String,
    pub equipes: u32,
    /// Personnes en place (recrutées, en amélioration ou en délocalisation).
    pub effectif: u32,
    /// Personnes à recruter ou en recrutement.
    pub effectif_a_venir: u32,
    pub gestionnaires: u32,
    /// Somme des prix imprimés (`prix_po`, référentiel) des équipes de l'organisation.
    pub valeur_equipes: Somme,
}

fn somme(pc: i64) -> Somme {
    Somme::from(Montant::pc(pc))
}

pub fn page_statistiques(etat: &Etat, r: &Referentiel) -> PageStatistiques {
    let inventaires = page_inventaire(etat).inventaires;

    let objets_de = |id: IdPersonnage| -> i64 {
        inventaires
            .iter()
            .filter(|i| {
                i.type_proprietaire == PROPRIETAIRE_PERSONNAGE
                    && personnage_de_ref(&i.ref_proprietaire) == Some(id)
            })
            .map(|i| i.valeur.pc)
            .sum()
    };

    let batiments: Vec<PatrimoineBatiment> = etat
        .batiments
        .values()
        .map(|b| {
            let mut valeur = 0i64;
            let mut salles = 0u32;
            for s in etat.salles_batiment(b.id) {
                if matches!(s.statut, StatutSalle::Construite) {
                    salles += 1;
                    valeur += r
                        .salle(&s.reference)
                        .and_then(|u| u.prix_po)
                        .map_or(0, |po| i64::from(po) * 100);
                }
                for x in &s.extensions {
                    if matches!(x.statut, StatutExtension::Construite) {
                        valeur += i64::from(x.prix_po) * 100;
                    }
                }
            }
            PatrimoineBatiment {
                id: b.id.0,
                nom: b.nom.clone(),
                localite: etat
                    .localites
                    .get(&b.localite)
                    .map(|l| l.nom.clone())
                    .unwrap_or_default(),
                proprietaire: etat
                    .proprietaires_batiments
                    .get(&b.id)
                    .and_then(|p| etat.personnages.get(p))
                    .map(|p| p.nom.clone()),
                salles,
                valeur: somme(valeur),
            }
        })
        .collect();
    let total_batiments: i64 = batiments.iter().map(|b| b.valeur.pc).sum();

    // Valeur des bâtiments possédés, par personnage (un propriétaire disparu ne compte plus).
    let mut batiments_par_pj: std::collections::BTreeMap<IdPersonnage, i64> =
        std::collections::BTreeMap::new();
    for (b, v) in etat.batiments.values().zip(&batiments) {
        if let Some(p) = etat
            .proprietaires_batiments
            .get(&b.id)
            .filter(|p| etat.personnages.contains_key(*p))
        {
            *batiments_par_pj.entry(*p).or_insert(0) += v.valeur.pc;
        }
    }
    let batiments_possedes: i64 = batiments_par_pj.values().sum();

    let personnages: Vec<PatrimoinePersonnage> = etat
        .personnages
        .values()
        .map(|p| {
            let objets = objets_de(p.id);
            let bats = batiments_par_pj.get(&p.id).copied().unwrap_or(0);
            let total = p.compte.pc + objets + bats;
            PatrimoinePersonnage {
                id: p.id.0,
                nom: p.nom.clone(),
                compte: somme(p.compte.pc),
                objets: somme(objets),
                batiments: somme(bats),
                total: somme(total),
            }
        })
        .collect();
    let total_personnages: i64 = personnages.iter().map(|p| p.total.pc).sum();

    let objets_autres: i64 = inventaires
        .iter()
        .filter(|i| i.type_proprietaire != PROPRIETAIRE_PERSONNAGE)
        .map(|i| i.valeur.pc)
        .sum();

    // Un bâtiment compte dans le patrimoine de sa localité.
    let mut batiments_par_localite: std::collections::BTreeMap<IdLocalite, i64> =
        std::collections::BTreeMap::new();
    for (b, v) in etat.batiments.values().zip(&batiments) {
        *batiments_par_localite.entry(b.localite).or_insert(0) += v.valeur.pc;
    }

    let localites: Vec<PatrimoineLocalite> = etat
        .localites
        .values()
        .map(|l| {
            let capitaux = i64::from(valeur_achat(r, &l.capitaux)) * 100;
            let bats = batiments_par_localite.get(&l.id).copied().unwrap_or(0);
            let total = l.tresorerie.pc + capitaux + bats;
            PatrimoineLocalite {
                id: l.id.0,
                nom: l.nom.clone(),
                tresorerie: somme(l.tresorerie.pc),
                capitaux: somme(capitaux),
                batiments: somme(bats),
                total: somme(total),
            }
        })
        .collect();
    let total_localites: i64 = localites.iter().map(|l| l.total.pc).sum();

    let organisations: Vec<StatOrganisation> = etat
        .organisations
        .values()
        .map(|o| {
            let equipes: Vec<&Equipe> = etat.equipes_organisation(o.id).collect();
            let (effectif, effectif_a_venir) = etat.effectif(equipes.iter().copied(), r);
            let gestionnaires: Vec<&GestionnaireEmploye> = etat
                .gestionnaires_de(Affectation::Organisation(o.id))
                .filter(|g| g.actif)
                .collect();
            let valeur: i64 = equipes
                .iter()
                .map(|e| {
                    r.equipe(&e.reference)
                        .and_then(|u| u.prix_po)
                        .map_or(0, |po| i64::from(po) * 100)
                })
                .sum();
            StatOrganisation {
                id: o.id.0,
                nom: o.nom.clone(),
                localite: etat
                    .localites
                    .get(&o.localite)
                    .map(|l| l.nom.clone())
                    .unwrap_or_default(),
                equipes: equipes.len() as u32,
                effectif,
                effectif_a_venir,
                gestionnaires: gestionnaires.len() as u32,
                valeur_equipes: somme(valeur),
            }
        })
        .collect();

    let valeur_totale: i64 = organisations.iter().map(|o| o.valeur_equipes.pc).sum();

    PageStatistiques {
        // Tous les bâtiments sont dans le total des localités ; ceux qui ont un propriétaire sont
        // aussi dans le total des personnages : on les retire une fois pour ne pas les compter deux fois.
        patrimoine_total: somme(
            total_personnages + total_localites + objets_autres - batiments_possedes,
        ),
        personnages,
        total_personnages: somme(total_personnages),
        localites,
        total_localites: somme(total_localites),
        objets_autres: somme(objets_autres),
        batiments,
        total_batiments: somme(total_batiments),
        effectif_total: organisations.iter().map(|o| o.effectif).sum(),
        effectif_a_venir_total: organisations.iter().map(|o| o.effectif_a_venir).sum(),
        organisations,
        valeur_equipes: somme(valeur_totale),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use campagne::evenements::Evenement;
    use campagne::Campagne;

    #[test]
    fn patrimoine_d_un_pj_et_total() {
        let r = regles::referentiel_officiel();
        let mut c = Campagne::nouvelle("Test");
        let id = c.nouvel_id_personnage();
        c.executer(
            Evenement::PersonnageCree {
                id,
                nom: "Lia".into(),
                compte_initial: Montant::po(100),
                prestige: None,
            },
            &r,
        )
        .unwrap();
        let etat = c.etat(&r);
        let p = page_statistiques(etat, &r);
        assert_eq!(p.personnages.len(), 1);
        assert_eq!(p.personnages[0].compte.pc, 10_000);
        assert_eq!(p.personnages[0].total.pc, 10_000);
        assert_eq!(p.total_personnages.pc, 10_000);
        assert_eq!(p.patrimoine_total.pc, 10_000);
        assert!(p.organisations.is_empty());
    }

    #[test]
    fn batiment_compte_chez_son_proprietaire_et_dans_sa_ville_sans_double_compte() {
        let r = regles::referentiel_officiel();
        let prix = i64::from(r.salle("bar").unwrap().prix_po.unwrap()) * 100;
        assert!(prix > 0);
        let mut c = Campagne::nouvelle("Test");
        let pj = c.nouvel_id_personnage();
        let l = c.nouvel_id_localite();
        let b = c.nouvel_id_batiment();
        let e = c.nouvel_id_etage();
        let s = c.nouvel_id_salle();
        for ev in [
            Evenement::PersonnageCree {
                id: pj,
                nom: "Seoni".into(),
                compte_initial: Montant::po(50),
                prestige: None,
            },
            Evenement::LocaliteCreee {
                id: l,
                nom: "Sandpoint".into(),
                genre: "ville".into(),
                population: None,
                categorie: CategorieLocalite::Officielle {
                    id: "petite-bourgade".into(),
                },
                notes: String::new(),
            },
            Evenement::BatimentCree {
                id: b,
                localite: l,
                nom: "Le Dragon vert".into(),
                classement: "taverne".into(),
                notes: String::new(),
                etages_successifs: false,
                modele: None,
            },
            Evenement::EtageAjoute {
                batiment: b,
                id: e,
                nom: "RDC".into(),
                mode: ModeChantier::Parallele,
            },
            Evenement::SalleAjoutee {
                id: s,
                batiment: b,
                etage: e,
                reference: "bar".into(),
                nom: None,
                deja_construite: true,
            },
        ] {
            c.executer(ev, &r).unwrap();
        }
        // Sans propriétaire : le bâtiment ne compte que dans sa ville.
        let p = page_statistiques(c.etat(&r), &r);
        assert_eq!(p.personnages[0].batiments.pc, 0);
        assert_eq!(p.localites[0].batiments.pc, prix);
        assert_eq!(p.patrimoine_total.pc, 5_000 + prix);

        c.executer(
            Evenement::BatimentProprietaire {
                id: b,
                proprietaire: Some(pj),
            },
            &r,
        )
        .unwrap();
        let p = page_statistiques(c.etat(&r), &r);
        assert_eq!(p.personnages[0].batiments.pc, prix);
        assert_eq!(p.personnages[0].total.pc, 5_000 + prix);
        assert_eq!(p.localites[0].batiments.pc, prix);
        // Le total ne compte le bâtiment qu'une fois.
        assert_eq!(p.patrimoine_total.pc, 5_000 + prix);
        assert_eq!(p.batiments[0].proprietaire.as_deref(), Some("Seoni"));

        // Un propriétaire inconnu est refusé.
        assert!(c
            .executer(
                Evenement::BatimentProprietaire {
                    id: b,
                    proprietaire: Some(IdPersonnage(99)),
                },
                &r,
            )
            .is_err());
    }
}
