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
    pub total: Somme,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct PatrimoineBatiment {
    pub id: u32,
    pub nom: String,
    pub localite: String,
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

    let personnages: Vec<PatrimoinePersonnage> = etat
        .personnages
        .values()
        .map(|p| {
            let objets = objets_de(p.id);
            let total = p.compte.pc + objets;
            PatrimoinePersonnage {
                id: p.id.0,
                nom: p.nom.clone(),
                compte: somme(p.compte.pc),
                objets: somme(objets),
                total: somme(total),
            }
        })
        .collect();
    let total_personnages: i64 = personnages.iter().map(|p| p.total.pc).sum();

    let localites: Vec<PatrimoineLocalite> = etat
        .localites
        .values()
        .map(|l| {
            let capitaux = i64::from(valeur_achat(r, &l.capitaux)) * 100;
            let total = l.tresorerie.pc + capitaux;
            PatrimoineLocalite {
                id: l.id.0,
                nom: l.nom.clone(),
                tresorerie: somme(l.tresorerie.pc),
                capitaux: somme(capitaux),
                total: somme(total),
            }
        })
        .collect();
    let total_localites: i64 = localites.iter().map(|l| l.total.pc).sum();

    let objets_autres: i64 = inventaires
        .iter()
        .filter(|i| i.type_proprietaire != PROPRIETAIRE_PERSONNAGE)
        .map(|i| i.valeur.pc)
        .sum();

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
                salles,
                valeur: somme(valeur),
            }
        })
        .collect();
    let total_batiments: i64 = batiments.iter().map(|b| b.valeur.pc).sum();

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
        patrimoine_total: somme(
            total_personnages + total_localites + total_batiments + objets_autres,
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
}
