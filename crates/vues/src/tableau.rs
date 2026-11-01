//! Tableau de bord : chiffres clés, localités, alertes, travaux en cours, tests de prestige.

use crate::{page, Capitaux, Somme};
use campagne::etat::Etat;
use campagne::modele::*;
use domaine::{CoutCapitaux, Montant, TypeCapital};
use regles::Referentiel;

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct TableauDeBord {
    pub date: String,
    pub jour: u32,
    pub tresorerie_localites: Somme,
    pub comptes_personnages: Somme,
    pub capitaux: Capitaux,
    pub employes: u32,
    pub employes_a_venir: u32,
    pub transferts: u32,
    pub localites: Vec<CarteLocalite>,
    pub alertes: Vec<Alerte>,
    pub travaux: Vec<Travail>,
    pub prestige: Vec<TestPrestige>,
    pub personnages: Vec<CartePersonnage>,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct CarteLocalite {
    pub id: u32,
    pub nom: String,
    pub genre: String,
    /// « Petite bourgade · limite 15/jour »
    pub categorie: String,
    pub tresorerie: Somme,
    pub salaires_jour: Option<Somme>,
    pub capitaux: Capitaux,
    pub recettes_en_attente: u32,
    /// « Aucun PJ depuis 12 j — prochaine usure dans 2 j »
    pub absence: Option<String>,
    pub batiments: u32,
    pub organisations: u32,
    pub employes: u32,
    pub presents: Vec<String>,
    pub en_route: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Alerte {
    pub texte: String,
    pub grave: bool,
    pub page: String,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct Travail {
    pub titre: String,
    pub detail: String,
    /// 0 à 1.
    pub avancement: f32,
    /// `None` : bloqué (limite de dépense nulle).
    pub jours_restants: Option<u32>,
    /// Date estimée de fin (calendrier de Golarion).
    pub fin: Option<String>,
    pub page: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct TestPrestige {
    pub localite: u32,
    pub nom: String,
    pub dd: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct CartePersonnage {
    pub id: u32,
    pub nom: String,
    pub compte: Somme,
    pub lieu: Option<String>,
    pub race: String,
    pub classes: String,
    pub niveau: Option<u32>,
    /// Meilleure caractéristique (modificateur), ex. « Force +4 ».
    pub meilleure_caracteristique: Option<String>,
    /// Meilleure compétence (bonus total), ex. « Perception +9 ».
    pub meilleure_competence: Option<String>,
}

fn carte_personnage(etat: &Etat, p: &Personnage) -> CartePersonnage {
    let resume = etat
        .fiche_de(p.id)
        .and_then(|e| crate::personnage::resume(fiches::Referentiel::partage(), &e));
    let fiche = crate::personnage::fiche_du_pj(etat, p.id).and_then(Result::ok);
    let meilleure_caracteristique = fiche
        .as_ref()
        .and_then(|f| f.caracteristiques.iter().max_by_key(|c| c.modificateur))
        .map(|c| format!("{} {:+}", c.nom, c.modificateur));
    let meilleure_competence = fiche
        .as_ref()
        .and_then(|f| f.competences.iter().max_by_key(|c| c.valeur.total))
        .map(|c| format!("{} {:+}", c.nom, c.valeur.total));
    CartePersonnage {
        id: p.id.0,
        nom: p.nom.clone(),
        compte: p.compte.into(),
        lieu: p
            .localite
            .and_then(|id| etat.localites.get(&id))
            .map(|l| l.nom.clone()),
        race: resume.as_ref().map(|x| x.race.clone()).unwrap_or_default(),
        classes: resume.as_ref().map(|x| x.classes.clone()).unwrap_or_default(),
        niveau: resume.as_ref().map(|x| x.niveau),
        meilleure_caracteristique,
        meilleure_competence,
    }
}

pub fn libelle_categorie(c: &CategorieLocalite, r: &Referentiel) -> String {
    match c {
        CategorieLocalite::Officielle { id } => match r.communaute(id) {
            Some(x) => format!("{} · limite {}/jour", x.nom, x.limite_depense_jour),
            None => id.clone(),
        },
        CategorieLocalite::Personnalisee {
            nom,
            limite_depense_jour,
        } => format!("{nom} (personnalisée) · limite {limite_depense_jour}/jour"),
    }
}

pub fn tableau_de_bord(c: &mut campagne::Campagne, r: &Referentiel) -> TableauDeBord {
    let etat = c.etat(r).clone();
    let cal = &c.calendrier;
    let (employes, employes_a_venir) = etat.effectif(etat.equipes.values(), r);
    TableauDeBord {
        date: cal.libelle(etat.jour),
        jour: etat.jour,
        tresorerie_localites: etat
            .localites
            .values()
            .map(|l| l.tresorerie)
            .sum::<Montant>()
            .into(),
        comptes_personnages: etat
            .personnages
            .values()
            .map(|p| p.compte)
            .sum::<Montant>()
            .into(),
        capitaux: etat
            .localites
            .values()
            .fold(CoutCapitaux::default(), |a, l| a + l.capitaux)
            .into(),
        employes,
        employes_a_venir,
        transferts: etat.transferts.len() as u32,
        localites: etat
            .localites
            .values()
            .map(|l| carte_localite(&etat, l, r))
            .collect(),
        alertes: alertes(&etat, r),
        travaux: travaux(&etat, r, cal),
        prestige: etat
            .prestige_en_attente
            .iter()
            .map(|(l, dd)| TestPrestige {
                localite: l.0,
                nom: etat
                    .localites
                    .get(l)
                    .map(|x| x.nom.clone())
                    .unwrap_or_default(),
                dd: *dd,
            })
            .collect(),
        personnages: etat
            .personnages
            .values()
            .map(|p| carte_personnage(&etat, p))
            .collect(),
    }
}

fn carte_localite(etat: &Etat, l: &Localite, r: &Referentiel) -> CarteLocalite {
    let salaires = etat.salaires_localite(l.id);
    let attente = if etat.a_entreprise(l.id, r) {
        etat.jours_sans_recettes(l.id)
    } else {
        0
    };
    let absence_j = etat.absence.get(&l.id).copied().unwrap_or(0);
    let absence = (absence_j > 0 && etat.frequentees.contains(&l.id)).then(|| {
        let usure = match etat.periode_usure(l.id) {
            Some(p) => format!("prochaine usure dans {} j", p - absence_j % p),
            None => "pas d'usure (gestionnaire)".to_string(),
        };
        format!("Aucun PJ depuis {absence_j} j — {usure}")
    });
    let (employes, _) = etat.effectif(etat.equipes.values().filter(|e| e.localite == l.id), r);
    CarteLocalite {
        id: l.id.0,
        nom: l.nom.clone(),
        genre: l.genre.clone(),
        categorie: libelle_categorie(&l.categorie, r),
        tresorerie: l.tresorerie.into(),
        salaires_jour: (salaires.pc > 0).then(|| salaires.into()),
        capitaux: l.capitaux.into(),
        recettes_en_attente: attente,
        absence,
        batiments: etat.batiments_localite(l.id).count() as u32,
        organisations: etat.organisations_localite(l.id).count() as u32,
        employes,
        presents: etat
            .personnages
            .values()
            .filter(|p| p.localite == Some(l.id))
            .map(|p| p.nom.clone())
            .collect(),
        en_route: etat
            .en_route_vers(l.id)
            .map(|t| format!("{} arrive le jour {}", t.lot, t.jour_arrivee))
            .collect(),
    }
}

fn alerte(texte: String, grave: bool, page: &str) -> Alerte {
    Alerte {
        texte,
        grave,
        page: page.to_string(),
    }
}

pub fn alertes(etat: &Etat, r: &Referentiel) -> Vec<Alerte> {
    let mut v = Vec::new();
    for l in etat.localites.values() {
        if l.tresorerie.est_negatif() {
            v.push(alerte(
                format!("Trésorerie de {} négative : {}", l.nom, l.tresorerie),
                true,
                page::LOCALITES,
            ));
        }
        let salaires = etat.salaires_localite(l.id);
        if salaires.pc > 0 && !l.tresorerie.est_negatif() {
            let jours = l.tresorerie.pc / salaires.pc;
            if jours < 7 {
                v.push(alerte(
                    format!(
                        "{} : salaires couverts encore {jours} jour(s) seulement ({salaires}/jour)",
                        l.nom
                    ),
                    false,
                    page::GESTIONNAIRES,
                ));
            }
        }
        let attente = etat.jours_sans_recettes(l.id);
        if attente > 0 && etat.a_entreprise(l.id, r) {
            v.push(alerte(
                format!("{} : {attente} jour(s) de recettes à encaisser", l.nom),
                false,
                page::RECETTES,
            ));
        }
        let absence = etat.absence.get(&l.id).copied().unwrap_or(0);
        if absence > 0 && etat.frequentees.contains(&l.id) && l.capitaux.unites() > 0 {
            if let Some(p) = etat.periode_usure(l.id) {
                let reste = p - absence % p;
                if reste <= 2 {
                    v.push(alerte(
                        format!(
                            "{} : usure des capitaux dans {reste} jour(s) (aucun PJ depuis {absence} j)",
                            l.nom
                        ),
                        false,
                        page::PERSONNAGES,
                    ));
                }
            }
        }
        if absence >= 23 && !etat.gestionnaire_paye(l.id) && etat.a_entreprise(l.id, r) {
            v.push(alerte(
                format!(
                    "{} : aucun PJ depuis {absence} j ; à 30 j, un test de prestige sera demandé au retour",
                    l.nom
                ),
                false,
                page::GESTIONNAIRES,
            ));
        }
    }
    // chantiers et recrutements bloqués faute de capitaux en réserve
    for s in etat.salles.values() {
        let Some(b) = etat.batiments.get(&s.batiment) else {
            continue;
        };
        let chantiers = std::iter::once(match &s.statut {
            StatutSalle::EnTravaux(ch) => Some((ch, None)),
            _ => None,
        })
        .flatten()
        .chain(s.extensions.iter().filter_map(|x| match &x.statut {
            StatutExtension::EnTravaux(ch) => Some((ch, Some(nom_extension(&x.reference, r)))),
            _ => None,
        }));
        for (ch, ext) in chantiers {
            if bloque(ch, etat, b.localite) {
                let quoi = match ext {
                    Some(e) => format!("{} : {e}", etat.nom_salle(s, r)),
                    None => etat.nom_salle(s, r),
                };
                v.push(alerte(
                    format!(
                        "{} — {quoi} : réserves vides pour {}",
                        b.nom,
                        manque(ch, etat, b.localite)
                    ),
                    false,
                    page::BATIMENTS,
                ));
            }
        }
    }
    for e in etat.equipes.values() {
        if let StatutEquipe::EnRecrutement(ch) = &e.statut {
            if bloque(ch, etat, e.localite) {
                v.push(alerte(
                    format!(
                        "{} : réserves vides pour {}",
                        etat.nom_equipe(e, r),
                        manque(ch, etat, e.localite)
                    ),
                    false,
                    page::ORGANISATIONS,
                ));
            }
        }
    }
    v.sort_by_key(|a| !a.grave);
    v
}

fn nom_extension(id: &str, r: &Referentiel) -> String {
    r.extension(id)
        .map(|u| u.nom.clone())
        .unwrap_or(id.to_string())
}

/// Le chantier a encore besoin de capitaux et aucun des types manquants n'est en réserve.
fn bloque(ch: &Chantier, etat: &Etat, l: IdLocalite) -> bool {
    let reste = ch.reste();
    let stock = etat
        .localites
        .get(&l)
        .map(|x| x.capitaux)
        .unwrap_or_default();
    reste.unites() > 0
        && TypeCapital::TOUS
            .iter()
            .all(|t| reste.get(*t) == 0 || stock.get(*t) == 0)
}

fn manque(ch: &Chantier, etat: &Etat, l: IdLocalite) -> String {
    let reste = ch.reste();
    let stock = etat
        .localites
        .get(&l)
        .map(|x| x.capitaux)
        .unwrap_or_default();
    TypeCapital::TOUS
        .iter()
        .filter(|t| reste.get(**t) > 0 && stock.get(**t) == 0)
        .map(|t| format!("{} {t}", reste.get(*t)))
        .collect::<Vec<_>>()
        .join(", ")
}

fn avance(ch: &Chantier) -> f32 {
    let c = ch.depense.unites() as f32 / ch.cout.unites().max(1) as f32;
    let t = if ch.duree == 0 {
        1.0
    } else {
        ch.jours_ecoules as f32 / ch.duree as f32
    };
    c.min(t).min(1.0)
}

/// Chantiers, recrutements, délocalisations et transferts en cours, du plus proche au plus
/// lointain.
pub fn travaux(
    etat: &Etat,
    r: &Referentiel,
    cal: &campagne::calendrier::Calendrier,
) -> Vec<Travail> {
    let mut v: Vec<Travail> = Vec::new();
    let fin = |reste: u32| (reste != u32::MAX).then(|| cal.libelle(etat.jour + reste));
    let mut ligne = |titre: String, detail: String, avancement: f32, reste: u32, page: &str| {
        v.push(Travail {
            titre,
            detail,
            avancement,
            jours_restants: (reste != u32::MAX).then_some(reste),
            fin: fin(reste),
            page: page.to_string(),
        })
    };
    for s in etat.salles.values() {
        let Some(b) = etat.batiments.get(&s.batiment) else {
            continue;
        };
        let limite = etat.limite_depense(b.localite, r).unwrap_or(0);
        if let StatutSalle::EnTravaux(ch) = &s.statut {
            let quoi = match &ch.nature {
                NatureChantier::Construction => "construction",
                NatureChantier::Amelioration { .. } => "amélioration",
            };
            ligne(
                format!("{} — {}", b.nom, etat.nom_salle(s, r)),
                format!("{quoi}, reste {}", ch.reste()),
                avance(ch),
                ch.jours_restants_estimes(limite),
                page::BATIMENTS,
            );
        }
        for x in &s.extensions {
            if let StatutExtension::EnTravaux(ch) = &x.statut {
                ligne(
                    format!(
                        "{} — {} : {}",
                        b.nom,
                        etat.nom_salle(s, r),
                        nom_extension(&x.reference, r)
                    ),
                    format!("extension, reste {}", ch.reste()),
                    avance(ch),
                    ch.jours_restants_estimes(limite),
                    page::BATIMENTS,
                );
            }
        }
    }
    for e in etat.equipes.values() {
        let org = etat
            .organisations
            .get(&e.organisation)
            .map(|o| o.nom.clone())
            .unwrap_or_default();
        match &e.statut {
            StatutEquipe::EnRecrutement(ch) => {
                let limite = etat.limite_depense(e.localite, r).unwrap_or(0);
                let quoi = match &ch.nature {
                    NatureChantier::Construction => "recrutement",
                    NatureChantier::Amelioration { .. } => "amélioration",
                };
                ligne(
                    format!("{org} — {}", etat.nom_equipe(e, r)),
                    format!("{quoi}, reste {}", ch.reste()),
                    avance(ch),
                    ch.jours_restants_estimes(limite),
                    page::ORGANISATIONS,
                );
            }
            StatutEquipe::EnDelocalisation(d) => {
                let vers = etat
                    .localites
                    .get(&d.vers)
                    .map(|l| l.nom.clone())
                    .unwrap_or_default();
                let total = (d.preparation + d.voyage).max(1);
                ligne(
                    format!("{org} — {}", etat.nom_equipe(e, r)),
                    format!("délocalisation vers {vers}"),
                    d.jours_ecoules as f32 / total as f32,
                    d.jours_restants(),
                    page::ORGANISATIONS,
                );
            }
            _ => {}
        }
    }
    for t in &etat.transferts {
        let nom = |id: &IdLocalite| {
            etat.localites
                .get(id)
                .map(|l| l.nom.clone())
                .unwrap_or_default()
        };
        let total = t.jour_arrivee.saturating_sub(t.jour_depart).max(1);
        ligne(
            format!("{} : {} → {}", t.lot, nom(&t.de), nom(&t.vers)),
            "transfert".into(),
            etat.jour.saturating_sub(t.jour_depart) as f32 / total as f32,
            t.jour_arrivee.saturating_sub(etat.jour),
            page::LOCALITES,
        );
    }
    v.sort_by_key(|t| t.jours_restants.unwrap_or(u32::MAX));
    v
}

#[cfg(test)]
mod tests {
    use super::*;
    use campagne::evenements::Evenement;
    use campagne::Campagne;

    #[test]
    fn tableau_d_une_campagne_avec_un_chantier() {
        let r = regles::referentiel_officiel();
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
            Evenement::PersonnageDeplace {
                id: pj,
                vers: Some(l),
            },
            Evenement::AjustementLocalite {
                localite: l,
                delta_po: Montant::po(100),
                ajout: CoutCapitaux {
                    marchandises: 10,
                    influence: 2,
                    travail: 10,
                    magie: 0,
                },
                retrait: CoutCapitaux::default(),
                motif: "départ".into(),
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
                deja_construite: false,
            },
            Evenement::ConstructionLancee {
                salle: s,
                facteur: 1,
            },
        ] {
            c.executer(ev, &r).unwrap();
        }
        let t = tableau_de_bord(&mut c, &r);
        assert_eq!(t.tresorerie_localites.pc, 10000);
        assert_eq!(t.comptes_personnages.texte, "50 po");
        assert_eq!(t.localites.len(), 1);
        assert_eq!(t.localites[0].presents, vec!["Seoni".to_string()]);
        assert!(t.localites[0].categorie.contains("limite"));
        assert_eq!(t.travaux.len(), 1);
        assert!(t.travaux[0].titre.contains("Bar"));
        assert!(t.travaux[0].jours_restants.is_some());
        assert!(t.travaux[0].fin.is_some());
        let en = crate::entete(&mut c, &r);
        assert_eq!(en.campagne, "Test");
        assert!(en.peut_annuler);
        assert!(en.derniere_action.is_some());
    }
}
