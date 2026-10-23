//! Page Organisations : équipes d'employés, appartenance, lieu de travail, recrutement, achat,
//! amélioration, délocalisation, vente (Guide de campagne p. 92-95).

use crate::batiments::{
    id_ressource, payeurs, ressource, Acquisition, Amelioration, ApercuCreation, Avancement, Bonus,
    ChoixModele, ChoixSalle, SaisieBatiment,
};
use crate::localites::Choix;
use crate::ventes::{etat_vente, EtatVente};
use crate::Capitaux;
use campagne::construction::ressources_admises;
use campagne::etat::Etat;
use campagne::evenements::Evenement;
use campagne::modele::*;
use campagne::modeles_perso::{ModelesPerso, PREFIXE};
use campagne::organisation::evenements_depuis_modele_organisation;
use domaine::{CoutCapitaux, Montant, TypeCapital};
use regles::Referentiel;
use std::collections::BTreeSet;

/// Saisie de création d'une organisation : mêmes champs que celle d'un bâtiment
/// (l'accélération porte ici sur l'Influence).
pub type SaisieOrganisation = SaisieBatiment;

// ------------------------------------------------------------------ liste et création

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct PageOrganisations {
    pub groupes: Vec<GroupeOrganisations>,
    pub localites: Vec<Choix>,
    pub modeles: Vec<ChoixModele>,
    /// Équipes du référentiel, pour l'ajout à une organisation.
    pub equipes: Vec<ChoixSalle>,
    pub payeurs: Vec<Choix>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct GroupeOrganisations {
    pub localite: String,
    pub organisations: Vec<ElementOrganisation>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ElementOrganisation {
    pub id: u32,
    pub nom: String,
    pub equipes: u32,
    pub effectif: u32,
}

fn info_equipe(u: &domaine::fiches::Unite) -> String {
    format!(
        "{} · {} j · {}",
        u.cout
            .map(|c| c.to_string())
            .unwrap_or("voir description".into()),
        u.duree_jours.unwrap_or(0),
        u.recettes.as_ref().map(|y| y.texte.as_str()).unwrap_or("—")
    )
}

pub fn page_organisations(
    c: &mut campagne::Campagne,
    r: &Referentiel,
    perso: &ModelesPerso,
) -> PageOrganisations {
    let etat = c.etat(r).clone();
    PageOrganisations {
        groupes: etat
            .localites
            .values()
            .map(|l| GroupeOrganisations {
                localite: l.nom.clone(),
                organisations: etat
                    .organisations_localite(l.id)
                    .map(|o| ElementOrganisation {
                        id: o.id.0,
                        nom: o.nom.clone(),
                        equipes: etat.equipes_organisation(o.id).count() as u32,
                        effectif: etat.effectif(etat.equipes_organisation(o.id), r).0,
                    })
                    .collect(),
            })
            .filter(|g| !g.organisations.is_empty())
            .collect(),
        localites: etat
            .localites
            .values()
            .map(|l| Choix {
                id: l.id.0,
                libelle: l.nom.clone(),
            })
            .collect(),
        modeles: perso
            .organisations
            .iter()
            .map(|p| ChoixModele {
                id: format!("{PREFIXE}{}", p.nom),
                libelle: format!("★ {} (modèle personnel)", p.nom),
                personnel: true,
            })
            .chain(r.modeles_organisations.iter().map(|m| ChoixModele {
                id: m.id.clone(),
                libelle: format!("{} — p. {}", m.nom, m.source.page.unwrap_or(0)),
                personnel: false,
            }))
            .collect(),
        equipes: r
            .equipes
            .iter()
            .map(|u| ChoixSalle {
                id: u.id.clone(),
                nom: u.nom.clone(),
                libelle: format!("{} — {}", u.nom, info_equipe(u)),
            })
            .collect(),
        payeurs: payeurs(&etat),
    }
}

fn modele_choisi(
    r: &Referentiel,
    perso: &ModelesPerso,
    id: &str,
) -> Option<(domaine::fiches::Modele, bool)> {
    r.modele_organisation(id)
        .cloned()
        .map(|m| (m, false))
        .or_else(|| perso.organisation(id).map(|p| (p.en_modele(r), true)))
}

fn facteur(f: u32) -> u32 {
    f.clamp(1, 4)
}

fn besoins(
    etat: &Etat,
    m: &domaine::fiches::Modele,
    s: &SaisieOrganisation,
    l: IdLocalite,
) -> (CoutCapitaux, CoutCapitaux) {
    let mut besoin = m.cout;
    if matches!(
        s.acquisition,
        Acquisition::Construire | Acquisition::AcheterCapitaux
    ) {
        besoin.influence *= facteur(s.facteur);
    }
    let mut a_acheter = besoin;
    if s.seulement_manquants {
        let reserves = etat
            .localites
            .get(&l)
            .map(|x| x.capitaux)
            .unwrap_or_default();
        for t in TypeCapital::TOUS {
            *a_acheter.get_mut(t) = besoin.get(t).saturating_sub(reserves.get(t));
        }
    }
    (besoin, a_acheter)
}

pub fn apercu_creation(
    c: &mut campagne::Campagne,
    r: &Referentiel,
    perso: &ModelesPerso,
    s: &SaisieOrganisation,
) -> Option<ApercuCreation> {
    let etat = c.etat(r).clone();
    let l = IdLocalite(s.localite?);
    let (m, personnel) = modele_choisi(r, perso, s.modele.as_deref()?)?;
    let (besoin, a_acheter) = besoins(&etat, &m, s, l);
    Some(ApercuCreation {
        composition: m.composition_imprimee.clone(),
        personnel,
        source: if personnel {
            "Modèle personnel".into()
        } else {
            r.reference(&m.source)
        },
        besoin: besoin.into(),
        a_acheter: a_acheter.into(),
        prix_capitaux_po: regles::calculs::valeur_achat(r, &a_acheter),
        prix_livre_po: m.prix_po,
        tresorerie: etat
            .localites
            .get(&l)
            .map(|x| format!("Trésorerie de {} ({})", x.nom, x.tresorerie))
            .unwrap_or_default(),
    })
}

fn payeur_de(payeur: Option<u32>, localite: IdLocalite) -> Payeur {
    match payeur {
        Some(p) => Payeur::Personnage(IdPersonnage(p)),
        None => Payeur::Localite(localite),
    }
}

/// Événement groupé de création d'une organisation.
pub fn creation(
    c: &mut campagne::Campagne,
    r: &Referentiel,
    perso: &ModelesPerso,
    s: &SaisieOrganisation,
) -> Result<(Evenement, String), String> {
    let (o, q) = (c.nouvel_id_organisation(), c.nouvel_id_equipe());
    let etat = c.etat(r).clone();
    let l = IdLocalite(s.localite.ok_or("Choisissez la localité")?);
    let nom = s.nom.trim().to_string();
    if nom.is_empty() {
        return Err("Donnez un nom à l'organisation".into());
    }
    let payeur = payeur_de(s.payeur, l);
    let modele = match s.modele.as_deref() {
        Some(id) => Some(modele_choisi(r, perso, id).ok_or("Modèle introuvable")?),
        None => None,
    };
    let (evs, ignores) = match &modele {
        Some((m, personnel)) => {
            let (mut evs, ig) = evenements_depuis_modele_organisation(m, o, l, &nom, q, r);
            if *personnel {
                if let Some(Evenement::OrganisationCreee { modele, .. }) = evs.first_mut() {
                    *modele = None;
                }
            }
            if !s.classement.trim().is_empty() {
                if let Some(Evenement::OrganisationCreee { classement, .. }) = evs.first_mut() {
                    *classement = s.classement.trim().into();
                }
            }
            let equipes: Vec<IdEquipe> = evs
                .iter()
                .filter_map(|x| match x {
                    Evenement::EquipeAjoutee { id, .. } => Some(*id),
                    _ => None,
                })
                .collect();
            match s.acquisition {
                Acquisition::Plan => {}
                Acquisition::Possede | Acquisition::AcheterBatiment => {
                    for x in evs.iter_mut() {
                        if let Evenement::EquipeAjoutee { deja_recrutee, .. } = x {
                            *deja_recrutee = true;
                        }
                    }
                    if s.acquisition == Acquisition::AcheterBatiment {
                        let montant = if s.prix.trim().is_empty() {
                            Montant::po(m.prix_po as i64)
                        } else {
                            Montant::depuis_texte(&s.prix)
                                .filter(|x| !x.est_negatif())
                                .ok_or("Prix illisible : écrivez par exemple 1200 ou 4 po 7 pa")?
                        };
                        if montant.pa > 0 {
                            evs.push(Evenement::Paiement {
                                payeur,
                                montant,
                                motif: format!("achat de l'organisation {nom}"),
                            });
                        }
                    }
                }
                Acquisition::Construire | Acquisition::AcheterCapitaux => {
                    if s.acquisition == Acquisition::AcheterCapitaux {
                        let (_, a_acheter) = besoins(&etat, m, s, l);
                        if a_acheter.unites() > 0 {
                            evs.push(Evenement::CapitauxAchetes {
                                localite: l,
                                payeur,
                                capitaux: a_acheter,
                            });
                        }
                    }
                    for id in equipes {
                        evs.push(Evenement::RecrutementLance {
                            equipe: id,
                            facteur: facteur(s.facteur),
                        });
                    }
                }
            }
            (evs, ig)
        }
        None => (
            vec![Evenement::OrganisationCreee {
                id: o,
                localite: l,
                nom: nom.clone(),
                classement: s.classement.trim().into(),
                notes: String::new(),
                modele: None,
            }],
            Vec::new(),
        ),
    };
    let mut msg = format!("Organisation « {nom} » créée");
    if !ignores.is_empty() {
        msg.push_str(&format!(
            " — éléments du modèle non gérés : {}",
            ignores.join(", ")
        ));
    }
    Ok((
        Evenement::Groupe {
            libelle: format!("Création de l'organisation {nom}"),
            evenements: evs,
        },
        msg,
    ))
}

// ------------------------------------------------------------------ fiche

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct FicheOrganisation {
    pub id: u32,
    pub nom: String,
    pub classement: String,
    pub modele: Option<String>,
    pub siege: String,
    pub localite: u32,
    /// « Effectif : 12 personne(s) (+3 à recruter ou en recrutement) »
    pub effectif: String,
    pub reste: Capitaux,
    pub bonus: Vec<Bonus>,
    pub gestionnaires: String,
    pub vente: EtatVente,
    pub equipes: Vec<FicheEquipe>,
    pub tresorerie: String,
    pub payeurs: Vec<Choix>,
    /// Toutes les organisations (changement d'appartenance d'une équipe).
    pub organisations: Vec<Choix>,
    pub localites: Vec<Choix>,
    /// Option personnalisée : un seul jet pour toute l'organisation.
    pub jet_unique: bool,
    pub equipes_recrutees: u32,
    /// Prix proposé pour délocaliser toute l'organisation (moitié des prix de recrutement).
    pub prix_deloc_organisation: String,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct FicheEquipe {
    pub id: u32,
    pub nom: String,
    pub type_equipe: Option<String>,
    /// « a_recruter », « en_recrutement », « recrutee » ou « en_delocalisation ».
    pub statut: String,
    pub statut_texte: String,
    /// Localité où se trouve l'équipe, si ce n'est pas celle du siège.
    pub ailleurs: Option<String>,
    pub effectif: u32,
    pub source: String,
    pub infos: String,
    pub point_a_confirmer: Option<String>,
    pub organisation: u32,
    pub lieu: Option<u32>,
    pub lieux: Vec<Choix>,
    pub moitie_sans_batiment: bool,
    pub admises: Vec<String>,
    pub ressource: Option<String>,
    pub revenus_actifs: bool,
    pub chantier: Option<Avancement>,
    pub constructible: bool,
    pub prix_achat_po: Option<u32>,
    pub ameliorations: Vec<Amelioration>,
    pub vente: Option<EtatVente>,
    pub delocalisable: bool,
    /// Moitié du prix de recrutement (GC p. 95), à préremplir.
    pub prix_deloc: String,
}

fn statut_equipe(s: &StatutEquipe, etat: &Etat) -> (&'static str, String) {
    match s {
        StatutEquipe::ARecruter => ("a_recruter", "à recruter".into()),
        StatutEquipe::EnRecrutement(c) => {
            let quoi = match &c.nature {
                NatureChantier::Construction => "en recrutement",
                NatureChantier::Amelioration { .. } => "en amélioration",
            };
            (
                "en_recrutement",
                format!("{quoi} — jour {}/{}", c.jours_ecoules, c.duree),
            )
        }
        StatutEquipe::Recrutee => ("recrutee", "recrutée".into()),
        StatutEquipe::EnDelocalisation(d) => {
            let vers = etat
                .localites
                .get(&d.vers)
                .map(|l| l.nom.clone())
                .unwrap_or_default();
            let phase = if d.en_voyage() {
                "en voyage (aucune recette)"
            } else {
                "préparation (½ recettes)"
            };
            (
                "en_delocalisation",
                format!(
                    "délocalisation vers {vers} — {phase}, {} j",
                    d.jours_restants()
                ),
            )
        }
    }
}

pub fn fiche_organisation(
    c: &mut campagne::Campagne,
    r: &Referentiel,
    id: u32,
) -> Option<FicheOrganisation> {
    let (pendant_amelioration, moitie, jet_unique) = (
        c.options.production_pendant_amelioration,
        c.options.moitie_equipe_sans_batiment,
        c.options.delocalisation_jet_unique,
    );
    let etat = c.etat(r).clone();
    let o = etat.organisations.get(&IdOrganisation(id))?.clone();
    let equipes: Vec<Equipe> = etat.equipes_organisation(o.id).cloned().collect();
    let limite = etat.limite_depense(o.localite, r).unwrap_or(0);
    let (gens, a_venir) = etat.effectif(equipes.iter(), r);
    let g: Vec<String> = etat
        .gestionnaires_de(Affectation::Organisation(o.id))
        .map(|g| g.nom.clone())
        .collect();
    let recrutees: Vec<&Equipe> = equipes
        .iter()
        .filter(|e| e.statut == StatutEquipe::Recrutee)
        .collect();
    let total_deloc: i64 = recrutees
        .iter()
        .filter_map(|e| r.equipe(&e.reference).and_then(|u| u.prix_po))
        .map(|p| p as i64 * 5)
        .sum();
    Some(FicheOrganisation {
        id,
        nom: o.nom.clone(),
        classement: o.classement.clone(),
        modele: o
            .modele
            .as_ref()
            .and_then(|m| r.modele_organisation(m))
            .map(|m| m.nom.clone()),
        siege: etat
            .localites
            .get(&o.localite)
            .map(|l| l.nom.clone())
            .unwrap_or_default(),
        localite: o.localite.0,
        effectif: if a_venir > 0 {
            format!("Effectif : {gens} personne(s) (+{a_venir} à recruter ou en recrutement)")
        } else {
            format!("Effectif : {gens} personne(s)")
        },
        reste: etat.cout_restant_equipes(equipes.iter(), r).into(),
        bonus: etat
            .bonus_equipes(equipes.iter(), r, pendant_amelioration, moitie)
            .into_iter()
            .map(|(res, v)| Bonus {
                ressource: id_ressource(res).into(),
                valeur: v,
            })
            .collect(),
        gestionnaires: if g.is_empty() {
            "Aucun gestionnaire (page Gestionnaires pour en affecter un)".to_string()
        } else {
            format!("Gestionnaire(s) : {}", g.join(", "))
        },
        vente: etat_vente(&etat, r, Actif::Organisation(o.id)),
        equipes: equipes
            .iter()
            .filter_map(|e| fiche_equipe(&etat, r, &o, e, limite, moitie))
            .collect(),
        tresorerie: etat
            .localites
            .get(&o.localite)
            .map(|l| format!("Trésorerie de {} ({})", l.nom, l.tresorerie))
            .unwrap_or_default(),
        payeurs: payeurs(&etat),
        organisations: etat
            .organisations
            .values()
            .map(|x| Choix {
                id: x.id.0,
                libelle: x.nom.clone(),
            })
            .collect(),
        localites: etat
            .localites
            .values()
            .map(|l| Choix {
                id: l.id.0,
                libelle: l.nom.clone(),
            })
            .collect(),
        jet_unique,
        equipes_recrutees: recrutees.len() as u32,
        prix_deloc_organisation: Montant::pa(total_deloc).to_string(),
    })
}

fn fiche_equipe(
    etat: &Etat,
    r: &Referentiel,
    o: &Organisation,
    e: &Equipe,
    limite: u32,
    moitie: bool,
) -> Option<FicheEquipe> {
    let u = r.equipe(&e.reference)?;
    let (statut, statut_texte) = statut_equipe(&e.statut, etat);
    let chantier = match &e.statut {
        StatutEquipe::EnRecrutement(ch) => {
            let j = ch.jours_restants_estimes(limite);
            let reste = ch.reste();
            Some(Avancement {
                capitaux_faits: ch.depense.unites(),
                capitaux_total: ch.cout.unites(),
                jours: ch.jours_ecoules,
                duree: ch.duree,
                estimation: if j == u32::MAX {
                    "bloqué : limite de dépense nulle".into()
                } else {
                    format!("≈ {j} j restants")
                },
                manque: (reste.unites() > 0).then(|| format!("manque : {reste}")),
            })
        }
        _ => None,
    };
    Some(FicheEquipe {
        id: e.id.0,
        nom: etat.nom_equipe(e, r),
        type_equipe: e
            .nom
            .as_ref()
            .filter(|n| !n.trim().is_empty())
            .map(|_| u.nom.clone()),
        statut: statut.into(),
        statut_texte,
        ailleurs: (e.localite != o.localite).then(|| {
            etat.localites
                .get(&e.localite)
                .map(|l| l.nom.clone())
                .unwrap_or_default()
        }),
        effectif: etat.effectif_equipe(e, r),
        source: r.reference(&u.source),
        infos: format!(
            "Coût : {} ({} po) · {} j · recettes : {}",
            u.cout
                .map(|c| c.to_string())
                .unwrap_or("voir description".into()),
            u.prix_po.unwrap_or(0),
            u.duree_jours.unwrap_or(0),
            u.recettes
                .as_ref()
                .map(|x| x.texte.as_str())
                .unwrap_or("aucune"),
        ),
        point_a_confirmer: u.point_a_confirmer.clone(),
        organisation: e.organisation.0,
        lieu: e.lieu.map(|b| b.0),
        lieux: etat
            .batiments_localite(e.localite)
            .map(|b| Choix {
                id: b.id.0,
                libelle: format!("à {}", b.nom),
            })
            .collect(),
        moitie_sans_batiment: moitie,
        admises: ressources_admises(u, &BTreeSet::new())
            .into_iter()
            .map(|x| id_ressource(x).into())
            .collect(),
        ressource: e.ressources.first().map(|x| id_ressource(*x).into()),
        revenus_actifs: e.revenus_actifs,
        chantier,
        constructible: u.cout.is_some(),
        prix_achat_po: u.prix_po,
        ameliorations: if e.statut == StatutEquipe::Recrutee {
            u.amelioration_en
                .iter()
                .map(|v| {
                    let cible = r.equipe(v);
                    Amelioration {
                        id: v.clone(),
                        nom: cible.map(|x| x.nom.clone()).unwrap_or(v.clone()),
                        detail: cible
                            .and_then(|c| regles::calculs::cout_amelioration(u, c).ok())
                            .map(|d| format!("{} · {} j", d.capitaux, d.duree_jours)),
                    }
                })
                .collect()
        } else {
            Vec::new()
        },
        vente: matches!(
            e.statut,
            StatutEquipe::Recrutee | StatutEquipe::EnDelocalisation(_)
        )
        .then(|| etat_vente(etat, r, Actif::Equipe(e.id))),
        delocalisable: e.statut == StatutEquipe::Recrutee,
        prix_deloc: u
            .prix_po
            .map(|p| Montant::pa(p as i64 * 5).to_string())
            .unwrap_or_default(),
    })
}

// ------------------------------------------------------------------ actions

/// Action demandée par l'interface sur une organisation ou une équipe. Les ventes et abandons
/// passent par `ActionBatiment` (même mécanique pour tous les biens).
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(tag = "type", rename_all = "camelCase"))]
pub enum ActionOrganisation {
    #[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
    AjouterEquipe {
        organisation: u32,
        reference: String,
        deja: bool,
    },
    #[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
    AffecterEquipe {
        equipe: u32,
        organisation: u32,
        lieu: Option<u32>,
    },
    #[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
    ReglerEquipe {
        equipe: u32,
        ressource: Option<String>,
        revenus_actifs: bool,
    },
    #[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
    LancerRecrutement { equipe: u32, facteur: u32 },
    #[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
    AcheterEquipe { equipe: u32, payeur: Option<u32> },
    #[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
    EquipePossedee { equipe: u32 },
    #[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
    RetirerEquipe { equipe: u32 },
    #[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
    AmeliorerEquipe {
        equipe: u32,
        vers: String,
        facteur: u32,
    },
    #[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
    Delocaliser {
        equipe: u32,
        vers: u32,
        preparation: String,
        voyage: String,
        payeur: Option<u32>,
        prix: String,
    },
    /// Option personnalisée : un seul jet de préparation pour toutes les équipes recrutées.
    #[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
    DelocaliserOrganisation {
        organisation: u32,
        vers: u32,
        preparation: String,
        voyage: String,
        payeur: Option<u32>,
        prix: String,
    },
}

fn lire_deloc(preparation: &str, voyage: &str, prix: &str) -> Result<(u32, u32, Montant), String> {
    let prep = preparation
        .trim()
        .parse::<u32>()
        .ok()
        .filter(|x| (1..=6).contains(x))
        .ok_or("La préparation est un jet de 1d6 : un nombre de 1 à 6")?;
    let voyage = voyage
        .trim()
        .parse::<u32>()
        .map_err(|_| "Indiquez la durée du voyage en jours")?;
    let prix = if prix.trim().is_empty() {
        Montant::ZERO
    } else {
        Montant::depuis_texte(prix)
            .filter(|m| !m.est_negatif())
            .ok_or("Prix illisible : écrivez par exemple 12 ou 4 po 7 pa")?
    };
    Ok((prep, voyage, prix))
}

pub fn action(
    c: &mut campagne::Campagne,
    r: &Referentiel,
    a: ActionOrganisation,
) -> Result<(Evenement, String), String> {
    use ActionOrganisation as A;
    let etat = c.etat(r).clone();
    let equipe = |id: u32| {
        etat.equipes
            .get(&IdEquipe(id))
            .cloned()
            .ok_or_else(|| "Équipe introuvable".to_string())
    };
    let nom = |e: &Equipe| etat.nom_equipe(e, r);
    let fac = facteur;
    Ok(match a {
        A::AjouterEquipe {
            organisation,
            reference,
            deja,
        } => {
            let u = r.equipe(&reference).ok_or("Équipe inconnue")?;
            let o = etat
                .organisations
                .get(&IdOrganisation(organisation))
                .ok_or("Organisation introuvable")?;
            (
                Evenement::EquipeAjoutee {
                    id: c.nouvel_id_equipe(),
                    organisation: o.id,
                    reference: reference.clone(),
                    nom: None,
                    lieu: None,
                    deja_recrutee: deja,
                },
                format!("{} ajoutée à {}", u.nom, o.nom),
            )
        }
        A::AffecterEquipe {
            equipe: id,
            organisation,
            lieu,
        } => {
            let e = equipe(id)?;
            (
                Evenement::EquipeAffectee {
                    id: e.id,
                    organisation: IdOrganisation(organisation),
                    lieu: lieu.map(IdBatiment),
                },
                format!("{} réaffectée", nom(&e)),
            )
        }
        A::ReglerEquipe {
            equipe: id,
            ressource: res,
            revenus_actifs,
        } => {
            let e = equipe(id)?;
            let ressources = match res {
                Some(x) => vec![ressource(&x).ok_or("Ressource inconnue")?],
                None => Vec::new(),
            };
            (
                Evenement::EquipeModifiee {
                    id: e.id,
                    nom: e.nom.clone(),
                    ressources,
                    revenus_actifs,
                },
                format!("{} modifiée", nom(&e)),
            )
        }
        A::LancerRecrutement {
            equipe: id,
            facteur: f,
        } => {
            let e = equipe(id)?;
            (
                Evenement::RecrutementLance {
                    equipe: e.id,
                    facteur: fac(f),
                },
                format!("Recrutement de {} lancé", nom(&e)),
            )
        }
        A::AcheterEquipe { equipe: id, payeur } => {
            let e = equipe(id)?;
            (
                Evenement::EquipeAchetee {
                    equipe: e.id,
                    payeur: Some(payeur_de(payeur, e.localite)),
                    prix: None,
                },
                format!("{} acquise", nom(&e)),
            )
        }
        A::EquipePossedee { equipe: id } => {
            let e = equipe(id)?;
            (
                Evenement::EquipeAchetee {
                    equipe: e.id,
                    payeur: None,
                    prix: Some(Montant::ZERO),
                },
                format!("{} marquée comme recrutée", nom(&e)),
            )
        }
        A::RetirerEquipe { equipe: id } => {
            let e = equipe(id)?;
            (
                Evenement::EquipeRetiree { id: e.id },
                format!("{} retirée", nom(&e)),
            )
        }
        A::AmeliorerEquipe {
            equipe: id,
            vers,
            facteur: f,
        } => {
            let e = equipe(id)?;
            (
                Evenement::AmeliorationEquipeLancee {
                    equipe: e.id,
                    vers,
                    facteur: fac(f),
                },
                format!("Amélioration de {} lancée", nom(&e)),
            )
        }
        A::Delocaliser {
            equipe: id,
            vers,
            preparation,
            voyage,
            payeur,
            prix,
        } => {
            let e = equipe(id)?;
            let (preparation, voyage, prix) = lire_deloc(&preparation, &voyage, &prix)?;
            (
                Evenement::DelocalisationLancee {
                    equipe: e.id,
                    vers: IdLocalite(vers),
                    preparation,
                    voyage,
                    payeur: payeur_de(payeur, e.localite),
                    prix,
                },
                format!("Délocalisation de {} lancée", nom(&e)),
            )
        }
        A::DelocaliserOrganisation {
            organisation,
            vers,
            preparation,
            voyage,
            payeur,
            prix,
        } => {
            if !c.options.delocalisation_jet_unique {
                return Err(
                    "Le jet unique est une option personnalisée : activez-la dans les Paramètres"
                        .into(),
                );
            }
            let o = etat
                .organisations
                .get(&IdOrganisation(organisation))
                .ok_or("Organisation introuvable")?;
            let (preparation, voyage, prix) = lire_deloc(&preparation, &voyage, &prix)?;
            let vers = IdLocalite(vers);
            let a_deplacer: Vec<&Equipe> = etat
                .equipes_organisation(o.id)
                .filter(|e| e.statut == StatutEquipe::Recrutee && e.localite != vers)
                .collect();
            if a_deplacer.is_empty() {
                return Err("Aucune équipe recrutée à déplacer vers cette localité".into());
            }
            let payeur = payeur_de(payeur, o.localite);
            let evs = a_deplacer
                .iter()
                .enumerate()
                .map(|(i, e)| Evenement::DelocalisationLancee {
                    equipe: e.id,
                    vers,
                    preparation,
                    voyage,
                    payeur,
                    // le prix total est porté par la première équipe
                    prix: if i == 0 { prix } else { Montant::ZERO },
                })
                .collect();
            (
                Evenement::Groupe {
                    libelle: format!("Délocalisation de l'organisation {}", o.nom),
                    evenements: evs,
                },
                format!("Délocalisation de {} lancée", o.nom),
            )
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use campagne::Campagne;

    fn campagne_de_base(r: &Referentiel) -> Campagne {
        let mut c = Campagne::nouvelle("Test");
        for nom in ["Sandpoint", "Magnimar"] {
            let l = c.nouvel_id_localite();
            let s = crate::localites::SaisieLocalite {
                nom: nom.into(),
                officielle: false,
                categorie_perso_limite: "10".into(),
                ..Default::default()
            };
            c.executer(s.creation(l, r).unwrap(), r).unwrap();
        }
        c
    }

    #[test]
    fn organisation_vide_equipe_et_fiche() {
        let r = regles::referentiel_officiel();
        let perso = ModelesPerso::default();
        let mut c = campagne_de_base(&r);
        let s = SaisieOrganisation {
            localite: Some(1),
            nom: "Les Ombres".into(),
            classement: "guilde".into(),
            facteur: 1,
            ..Default::default()
        };
        let (ev, msg) = creation(&mut c, &r, &perso, &s).unwrap();
        assert!(msg.contains("Les Ombres"));
        c.executer(ev, &r).unwrap();
        let p = page_organisations(&mut c, &r, &perso);
        assert_eq!(p.groupes[0].organisations[0].nom, "Les Ombres");
        let (ev, _) = action(
            &mut c,
            &r,
            ActionOrganisation::AjouterEquipe {
                organisation: 1,
                reference: r.equipes[0].id.clone(),
                deja: true,
            },
        )
        .unwrap();
        c.executer(ev, &r).unwrap();
        let f = fiche_organisation(&mut c, &r, 1).unwrap();
        assert_eq!(f.equipes.len(), 1);
        assert_eq!(f.equipes[0].statut, "recrutee");
        assert!(f.equipes[0].delocalisable);
        assert_eq!(f.equipes_recrutees, 1);
        // délocalisation d'une équipe
        let (ev, _) = action(
            &mut c,
            &r,
            ActionOrganisation::Delocaliser {
                equipe: 1,
                vers: 2,
                preparation: "3".into(),
                voyage: "4".into(),
                payeur: None,
                prix: "".into(),
            },
        )
        .unwrap();
        c.executer(ev, &r).unwrap();
        let f = fiche_organisation(&mut c, &r, 1).unwrap();
        assert_eq!(f.equipes[0].statut, "en_delocalisation");
    }

    #[test]
    fn modele_officiel_et_saisies_refusees() {
        let r = regles::referentiel_officiel();
        let perso = ModelesPerso::default();
        let mut c = campagne_de_base(&r);
        let s = SaisieOrganisation {
            localite: Some(1),
            modele: Some(r.modeles_organisations[0].id.clone()),
            nom: "Compagnie".into(),
            facteur: 1,
            ..Default::default()
        };
        assert!(apercu_creation(&mut c, &r, &perso, &s).is_some());
        let (ev, _) = creation(&mut c, &r, &perso, &s).unwrap();
        c.executer(ev, &r).unwrap();
        assert!(!fiche_organisation(&mut c, &r, 1)
            .unwrap()
            .equipes
            .is_empty());
        let vide = SaisieOrganisation {
            localite: Some(1),
            ..Default::default()
        };
        assert!(creation(&mut c, &r, &perso, &vide).is_err());
        assert!(lire_deloc("7", "3", "").is_err());
        assert!(lire_deloc("3", "x", "").is_err());
        assert!(lire_deloc("3", "2", "-5").is_err());
        assert!(action(
            &mut c,
            &r,
            ActionOrganisation::RetirerEquipe { equipe: 999 }
        )
        .is_err());
        // jet unique non activé : refusé
        assert!(action(
            &mut c,
            &r,
            ActionOrganisation::DelocaliserOrganisation {
                organisation: 1,
                vers: 2,
                preparation: "3".into(),
                voyage: "2".into(),
                payeur: None,
                prix: "".into()
            }
        )
        .is_err());
    }
}

#[cfg(all(test, feature = "serde"))]
mod tests_json {
    use super::*;

    #[test]
    fn actions_lues_comme_l_interface_les_envoie() {
        let a: ActionOrganisation = serde_json::from_str(
            r#"{"type":"affecterEquipe","equipe":1,"organisation":2,"lieu":null}"#,
        )
        .unwrap();
        assert!(matches!(
            a,
            ActionOrganisation::AffecterEquipe { lieu: None, .. }
        ));
        let a: ActionOrganisation = serde_json::from_str(
            r#"{"type":"delocaliser","equipe":1,"vers":2,"preparation":"3","voyage":"4","payeur":null,"prix":"5"}"#,
        )
        .unwrap();
        assert!(matches!(a, ActionOrganisation::Delocaliser { vers: 2, .. }));
    }
}
