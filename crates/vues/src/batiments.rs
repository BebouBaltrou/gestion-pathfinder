//! Page Bâtiments : étages, salles, construction, achat, amélioration, extensions (GC p. 108),
//! recettes par salle, vente, abandon et réparation.

use crate::localites::Choix;
use crate::ventes::{etat_vente, EtatVente};
use crate::Capitaux;
use campagne::construction::evenements_depuis_modele;
use campagne::etat::Etat;
use campagne::evenements::Evenement;
use campagne::modele::*;
use campagne::modeles_perso::{ModelesPerso, PREFIXE};
use domaine::{CoutCapitaux, Montant, Ressource, Source, TypeCapital};
use regles::Referentiel;

// ------------------------------------------------------------------ ressources

/// Identifiant d'une ressource partagé avec l'interface : « po », « marchandises »…
pub fn id_ressource(r: Ressource) -> &'static str {
    match r {
        Ressource::Po => "po",
        Ressource::Marchandises => "marchandises",
        Ressource::Influence => "influence",
        Ressource::Travail => "travail",
        Ressource::Magie => "magie",
    }
}

pub fn ressource(id: &str) -> Option<Ressource> {
    Ressource::TOUTES
        .into_iter()
        .find(|r| id_ressource(*r) == id)
}

/// Bonus de recettes : (ressource, texte « +3 »).
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Bonus {
    pub ressource: String,
    pub valeur: u32,
}

// ------------------------------------------------------------------ liste et création

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct PageBatiments {
    pub groupes: Vec<GroupeBatiments>,
    /// Localités où créer un bâtiment.
    pub localites: Vec<Choix>,
    pub modeles: Vec<ChoixModele>,
    /// Salles du référentiel, pour l'ajout à un étage.
    pub salles: Vec<ChoixSalle>,
    /// Personnages pouvant payer un achat : « Compte de Seoni (145 po) ».
    pub payeurs: Vec<Choix>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct GroupeBatiments {
    pub localite: String,
    pub batiments: Vec<ElementBatiment>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct ElementBatiment {
    pub id: u32,
    pub nom: String,
    pub en_travaux: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct ChoixModele {
    pub id: String,
    pub libelle: String,
    pub personnel: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ChoixSalle {
    pub id: String,
    pub nom: String,
    /// « Bar — 5 Marchandises, 3 Travail · 6 j · Marchandises +6 »
    pub libelle: String,
}

fn info_unite(u: &domaine::fiches::Unite) -> String {
    format!(
        "{} · {} j · {}",
        u.cout
            .map(|c| c.to_string())
            .unwrap_or("voir description".into()),
        u.duree_jours.unwrap_or(0),
        u.recettes.as_ref().map(|y| y.texte.as_str()).unwrap_or("—")
    )
}

pub fn payeurs(etat: &Etat) -> Vec<Choix> {
    etat.personnages
        .values()
        .map(|p| Choix {
            id: p.id.0,
            libelle: format!("Compte de {} ({})", p.nom, p.compte),
        })
        .collect()
}

pub fn page_batiments(
    c: &mut campagne::Campagne,
    r: &Referentiel,
    perso: &ModelesPerso,
) -> PageBatiments {
    let etat = c.etat(r).clone();
    PageBatiments {
        groupes: etat
            .localites
            .values()
            .map(|l| GroupeBatiments {
                localite: l.nom.clone(),
                batiments: etat
                    .batiments_localite(l.id)
                    .map(|b| ElementBatiment {
                        id: b.id.0,
                        nom: b.nom.clone(),
                        en_travaux: etat
                            .salles_batiment(b.id)
                            .filter(|s| matches!(s.statut, StatutSalle::EnTravaux(_)))
                            .count() as u32,
                    })
                    .collect(),
            })
            .filter(|g| !g.batiments.is_empty())
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
            .batiments
            .iter()
            .map(|p| ChoixModele {
                id: format!("{PREFIXE}{}", p.nom),
                libelle: format!("★ {} (modèle personnel)", p.nom),
                personnel: true,
            })
            .chain(r.modeles_batiments.iter().map(|m| ChoixModele {
                id: m.id.clone(),
                libelle: format!("{} — p. {}", m.nom, m.source.page.unwrap_or(0)),
                personnel: false,
            }))
            .collect(),
        salles: r
            .salles
            .iter()
            .map(|u| ChoixSalle {
                id: u.id.clone(),
                nom: u.nom.clone(),
                libelle: format!("{} — {}", u.nom, info_unite(u)),
            })
            .collect(),
        payeurs: payeurs(&etat),
    }
}

/// Façon d'obtenir un bâtiment créé depuis un modèle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub enum Acquisition {
    /// Les salles restent « à construire » ; rien n'est lancé.
    #[default]
    Plan,
    /// Déjà possédé au départ : construit, gratuit.
    Possede,
    /// Construction lancée avec les capitaux en réserve dans la localité.
    Construire,
    /// Capitaux achetés en po (Table 2-1, GC p. 78), puis construction lancée.
    AcheterCapitaux,
    /// Bâtiment acheté terminé : aucune durée, prix en po saisi par le joueur.
    AcheterBatiment,
}

/// Saisie de création d'un bâtiment.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct SaisieBatiment {
    pub localite: Option<u32>,
    pub modele: Option<String>,
    pub nom: String,
    pub classement: String,
    pub acquisition: Acquisition,
    /// Accélération ×1 à ×4 (GC p. 95).
    pub facteur: u32,
    pub seulement_manquants: bool,
    /// Prix payé pour un bâtiment acheté terminé (vide = prix du livre).
    pub prix: String,
    /// PJ qui paie (sinon la trésorerie de la localité).
    pub payeur: Option<u32>,
}

/// Ce que l'écran de création affiche pour le modèle et l'acquisition choisis.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct ApercuCreation {
    pub composition: String,
    pub personnel: bool,
    pub source: String,
    /// Capitaux nécessaires, accélération comprise.
    pub besoin: Capitaux,
    /// Capitaux à acheter (acquisition « acheter les capitaux »).
    pub a_acheter: Capitaux,
    pub prix_capitaux_po: u32,
    pub prix_livre_po: u32,
    /// Trésorerie de la localité, pour le choix du payeur.
    pub tresorerie: String,
}

fn modele_choisi(
    r: &Referentiel,
    perso: &ModelesPerso,
    id: &str,
) -> Option<(domaine::fiches::Modele, bool)> {
    r.modele_batiment(id)
        .cloned()
        .map(|m| (m, false))
        .or_else(|| perso.batiment(id).map(|p| (p.en_modele(r), true)))
}

fn facteur(f: u32) -> u32 {
    f.clamp(1, 4)
}

/// Capitaux nécessaires et à acheter pour une création.
fn besoins(
    etat: &Etat,
    r: &Referentiel,
    m: &domaine::fiches::Modele,
    s: &SaisieBatiment,
    l: IdLocalite,
) -> (CoutCapitaux, CoutCapitaux) {
    let mut besoin = m.cout;
    if matches!(
        s.acquisition,
        Acquisition::Construire | Acquisition::AcheterCapitaux
    ) {
        besoin.travail *= facteur(s.facteur);
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
    let _ = r;
    (besoin, a_acheter)
}

pub fn apercu_creation(
    c: &mut campagne::Campagne,
    r: &Referentiel,
    perso: &ModelesPerso,
    s: &SaisieBatiment,
) -> Option<ApercuCreation> {
    let etat = c.etat(r).clone();
    let l = IdLocalite(s.localite?);
    let (m, personnel) = modele_choisi(r, perso, s.modele.as_deref()?)?;
    let (besoin, a_acheter) = besoins(&etat, r, &m, s, l);
    Some(ApercuCreation {
        composition: m.composition_imprimee.clone(),
        personnel,
        source: source_modele(personnel, r, &m.source),
        besoin: besoin.into(),
        a_acheter: a_acheter.into(),
        prix_capitaux_po: regles::calculs::valeur_achat(r, &a_acheter),
        prix_livre_po: m.prix_po,
        tresorerie: tresorerie_localite(&etat, l),
    })
}

/// Événement groupé de création (`ids` : nouveaux identifiants de la campagne).
pub fn creation(
    c: &mut campagne::Campagne,
    r: &Referentiel,
    perso: &ModelesPerso,
    s: &SaisieBatiment,
) -> Result<(Evenement, String), String> {
    let (b, e, sa) = (
        c.nouvel_id_batiment(),
        c.nouvel_id_etage(),
        c.nouvel_id_salle(),
    );
    let etat = c.etat(r).clone();
    let l = IdLocalite(s.localite.ok_or("Choisissez la localité")?);
    let nom = s.nom.trim().to_string();
    if nom.is_empty() {
        return Err("Donnez un nom au bâtiment".into());
    }
    let payeur = match s.payeur {
        Some(p) => Payeur::Personnage(IdPersonnage(p)),
        None => Payeur::Localite(l),
    };
    let modele = match s.modele.as_deref() {
        Some(id) => Some(modele_choisi(r, perso, id).ok_or("Modèle introuvable")?),
        None => None,
    };
    let (evs, ignores) = match &modele {
        Some((m, personnel)) => {
            let (mut evs, ig) = if *personnel {
                let p = perso
                    .batiment(s.modele.as_deref().unwrap_or_default())
                    .ok_or("Modèle introuvable")?;
                (p.evenements(b, l, &nom, e, sa), Vec::new())
            } else {
                evenements_depuis_modele(r, m, b, l, &nom, e, sa)
            };
            let salles: Vec<IdSalle> = evs
                .iter()
                .filter_map(|x| match x {
                    Evenement::SalleAjoutee { id, .. } => Some(*id),
                    _ => None,
                })
                .collect();
            match s.acquisition {
                Acquisition::Plan => {}
                Acquisition::Possede | Acquisition::AcheterBatiment => {
                    for x in evs.iter_mut() {
                        match x {
                            Evenement::SalleAjoutee {
                                deja_construite, ..
                            }
                            | Evenement::ExtensionAjoutee {
                                deja_construite, ..
                            } => *deja_construite = true,
                            _ => {}
                        }
                    }
                    if s.acquisition == Acquisition::AcheterBatiment {
                        let montant = prix_achat_saisi(&s.prix, m.prix_po)?;
                        if montant.pc > 0 {
                            evs.push(Evenement::Paiement {
                                payeur,
                                montant,
                                motif: format!("achat du bâtiment {nom}"),
                            });
                        }
                    }
                }
                Acquisition::Construire | Acquisition::AcheterCapitaux => {
                    if s.acquisition == Acquisition::AcheterCapitaux {
                        let (_, a_acheter) = besoins(&etat, r, m, s, l);
                        if a_acheter.unites() > 0 {
                            evs.push(Evenement::CapitauxAchetes {
                                localite: l,
                                payeur,
                                capitaux: a_acheter,
                            });
                        }
                    }
                    let extensions: Vec<(IdSalle, String)> = evs
                        .iter()
                        .filter_map(|x| match x {
                            Evenement::ExtensionAjoutee {
                                salle, reference, ..
                            } => Some((*salle, reference.clone())),
                            _ => None,
                        })
                        .collect();
                    for id in salles {
                        evs.push(Evenement::ConstructionLancee {
                            salle: id,
                            facteur: facteur(s.facteur),
                        });
                    }
                    for (salle, reference) in extensions {
                        evs.push(Evenement::ExtensionLancee {
                            salle,
                            reference,
                            facteur: facteur(s.facteur),
                        });
                    }
                }
            }
            if !s.classement.trim().is_empty() {
                if let Some(Evenement::BatimentCree { classement, .. }) = evs.first_mut() {
                    *classement = s.classement.trim().into();
                }
            }
            (evs, ig)
        }
        None => (
            vec![
                Evenement::BatimentCree {
                    id: b,
                    localite: l,
                    nom: nom.clone(),
                    classement: s.classement.trim().into(),
                    notes: String::new(),
                    etages_successifs: false,
                    modele: None,
                },
                Evenement::EtageAjoute {
                    batiment: b,
                    id: e,
                    nom: "Rez-de-chaussée".into(),
                    mode: ModeChantier::Parallele,
                },
            ],
            Vec::new(),
        ),
    };
    let mut msg = format!("Bâtiment « {nom} » créé");
    if !ignores.is_empty() {
        msg.push_str(&format!(
            " — extensions du modèle à placer à la main (salle introuvable) : {}",
            ignores.join(", ")
        ));
    }
    Ok((
        Evenement::Groupe {
            libelle: format!("Création du bâtiment {nom}"),
            evenements: evs,
        },
        msg,
    ))
}

// ------------------------------------------------------------------ fiche

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct FicheBatiment {
    pub id: u32,
    pub nom: String,
    pub classement: String,
    pub notes: String,
    /// Mis en favori : il passe en tête sur la liste de sa localité.
    pub favori: bool,
    pub modele: Option<String>,
    pub localite: u32,
    /// « Sandpoint — limite des dépenses : 15 unités/jour (une par localité) »
    pub lieu: String,
    pub reste: Capitaux,
    pub duree_restante: u32,
    pub bonus: Vec<Bonus>,
    pub equipes: String,
    pub gestionnaires: String,
    pub etages_successifs: bool,
    pub vente: EtatVente,
    pub etages: Vec<FicheEtage>,
    /// Trésorerie de la localité (payeur par défaut).
    pub tresorerie: String,
    pub payeurs: Vec<Choix>,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct FicheEtage {
    pub id: u32,
    pub nom: String,
    /// Salles construites l'une après l'autre (sinon en parallèle). Option personnalisée.
    pub sequentiel: bool,
    pub duree_restante: u32,
    pub reste: Capitaux,
    pub salles: Vec<FicheSalle>,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct FicheSalle {
    pub id: u32,
    pub nom: String,
    /// Type de salle quand la salle porte un nom propre.
    pub type_salle: Option<String>,
    /// « a_construire », « en_travaux » ou « construite ».
    pub statut: String,
    pub statut_texte: String,
    pub brisee: bool,
    pub source: String,
    /// « Coût : … (… po) · 6 j · recettes : … »
    pub infos: String,
    pub point_a_confirmer: Option<String>,
    /// Ressources que la salle peut générer ; un seul type coché à la fois.
    pub admises: Vec<String>,
    pub ressource: Option<String>,
    pub revenus_actifs: bool,
    pub chantier: Option<Avancement>,
    /// Constructible : coût connu (sinon voir la description du livre).
    pub constructible: bool,
    pub prix_achat_po: Option<u32>,
    pub ameliorations: Vec<Amelioration>,
    pub vente: Option<EtatVente>,
    /// Réparation d'une salle brisée : (po, capitaux).
    pub reparation: Option<(String, Option<String>)>,
    pub extensions: Vec<FicheExtension>,
    pub extensions_possibles: Vec<ExtensionPossible>,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct Avancement {
    pub capitaux_faits: u32,
    pub capitaux_total: u32,
    pub jours: u32,
    pub duree: u32,
    /// « ≈ 4 j restants » ou « bloqué : limite de dépense nulle ».
    pub estimation: String,
    pub manque: Option<String>,
}

/// Avancement d'un chantier (salle ou équipe) pour l'affichage.
pub(crate) fn avancement_chantier(ch: &Chantier, limite: u32) -> Avancement {
    let j = ch.jours_restants_estimes(limite);
    let reste = ch.reste();
    Avancement {
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
    }
}

/// Prix d'achat saisi (« 1200 », « 4 po 7 pa »), ou le prix du livre si la saisie est vide.
pub(crate) fn prix_achat_saisi(saisi: &str, prix_livre_po: u32) -> Result<Montant, &'static str> {
    if saisi.trim().is_empty() {
        Ok(Montant::po(prix_livre_po as i64))
    } else {
        Montant::depuis_texte(saisi)
            .filter(|x| !x.est_negatif())
            .ok_or("Prix illisible : écrivez par exemple 1200 ou 4 po 7 pa")
    }
}

/// Source d'un modèle : « Modèle personnel » ou la référence du livre.
pub(crate) fn source_modele(personnel: bool, r: &Referentiel, source: &Source) -> String {
    if personnel {
        "Modèle personnel".into()
    } else {
        r.reference(source)
    }
}

/// Trésorerie de la localité, pour l'affichage d'un achat.
pub(crate) fn tresorerie_localite(etat: &Etat, l: IdLocalite) -> String {
    etat.localites
        .get(&l)
        .map(|x| format!("Trésorerie de {} ({})", x.nom, x.tresorerie))
        .unwrap_or_default()
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Amelioration {
    pub id: String,
    pub nom: String,
    /// « 3 Travail · 2 j » (différence de coût, GC p. 95).
    pub detail: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct FicheExtension {
    pub reference: String,
    pub nom: String,
    pub statut_texte: String,
    pub a_construire: bool,
    /// Lançable : la salle est construite ou en travaux.
    pub lancable: bool,
    pub description: String,
    pub detail: String,
    pub prix_po: u32,
    pub manque: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct ExtensionPossible {
    pub id: String,
    pub nom: String,
    pub aide: String,
    /// Coût, prix et durée à saisir (piège : voir la salle piégée).
    pub saisie_requise: bool,
    pub detail: String,
}

fn statut_salle(s: &StatutSalle) -> (&'static str, String) {
    match s {
        StatutSalle::AConstruire => ("a_construire", "à construire".into()),
        StatutSalle::EnTravaux(c) => {
            let quoi = match &c.nature {
                NatureChantier::Construction => "en construction",
                NatureChantier::Amelioration { .. } => "en amélioration",
            };
            (
                "en_travaux",
                format!("{quoi} — jour {}/{}", c.jours_ecoules, c.duree),
            )
        }
        StatutSalle::Construite => ("construite", "construite".into()),
    }
}

fn bonus(m: std::collections::BTreeMap<Ressource, u32>) -> Vec<Bonus> {
    m.into_iter()
        .map(|(r, v)| Bonus {
            ressource: id_ressource(r).into(),
            valeur: v,
        })
        .collect()
}

pub fn fiche_batiment(
    c: &mut campagne::Campagne,
    r: &Referentiel,
    id: u32,
) -> Option<FicheBatiment> {
    let pendant_amelioration = c.options.production_pendant_amelioration;
    let etat = c.etat(r).clone();
    let b = etat.batiments.get(&IdBatiment(id))?.clone();
    let limite = etat.limite_depense(b.localite, r).unwrap_or(0);
    let loc = etat.localites.get(&b.localite);
    let equipes: Vec<String> = etat
        .equipes_batiment(b.id)
        .map(|e| {
            let o = etat
                .organisations
                .get(&e.organisation)
                .map(|o| o.nom.clone())
                .unwrap_or_default();
            format!("{} ({o})", etat.nom_equipe(e, r))
        })
        .collect();
    let g: Vec<String> = etat
        .gestionnaires_de(Affectation::Batiment(b.id))
        .map(|g| g.nom.clone())
        .collect();
    Some(FicheBatiment {
        id,
        nom: b.nom.clone(),
        classement: b.classement.clone(),
        notes: b.notes.clone(),
        favori: etat.batiments_favoris.contains(&b.id),
        modele: b
            .modele
            .as_ref()
            .and_then(|m| r.modele_batiment(m))
            .map(|m| m.nom.clone()),
        localite: b.localite.0,
        lieu: format!(
            "{} — limite des dépenses : {limite} unités/jour ({})",
            loc.map(|l| l.nom.as_str()).unwrap_or("?"),
            if etat.limite_par_pj {
                "par PJ présent, additionnées"
            } else {
                "une par localité"
            }
        ),
        reste: etat.cout_restant(etat.salles_batiment(b.id), r).into(),
        duree_restante: etat.duree_restante_batiment(&b, r),
        bonus: bonus(etat.bonus_par_ressource(etat.salles_batiment(b.id), r, pendant_amelioration)),
        equipes: if equipes.is_empty() {
            "Aucune équipe n'y travaille (page Organisations pour en affecter)".to_string()
        } else {
            format!(
                "Équipes qui y travaillent ({} personne(s)) : {}",
                etat.effectif(etat.equipes_batiment(b.id), r).0,
                equipes.join(", ")
            )
        },
        gestionnaires: if g.is_empty() {
            "Aucun gestionnaire".to_string()
        } else {
            format!("Gestionnaire(s) : {}", g.join(", "))
        },
        etages_successifs: b.etages_successifs,
        vente: etat_vente(&etat, r, Actif::Batiment(b.id)),
        etages: b
            .etages
            .iter()
            .map(|e| {
                let salles: Vec<SalleBatie> = etat.salles_etage(e.id).cloned().collect();
                FicheEtage {
                    id: e.id.0,
                    nom: e.nom.clone(),
                    sequentiel: e.mode == ModeChantier::Sequentiel,
                    duree_restante: etat.duree_restante_etage(e, r),
                    reste: etat.cout_restant(salles.iter(), r).into(),
                    salles: salles
                        .iter()
                        .filter_map(|s| fiche_salle(&etat, r, s, limite))
                        .collect(),
                }
            })
            .collect(),
        tresorerie: loc
            .map(|l| format!("Trésorerie de {} ({})", l.nom, l.tresorerie))
            .unwrap_or_default(),
        payeurs: payeurs(&etat),
    })
}

fn fiche_salle(etat: &Etat, r: &Referentiel, sb: &SalleBatie, limite: u32) -> Option<FicheSalle> {
    let u = r.salle(&sb.reference)?;
    let (statut, statut_texte) = statut_salle(&sb.statut);
    let chantier = match &sb.statut {
        StatutSalle::EnTravaux(ch) => Some(avancement_chantier(ch, limite)),
        _ => None,
    };
    let ameliorations = if sb.statut == StatutSalle::Construite {
        u.amelioration_en
            .iter()
            .map(|v| {
                let cible = r.salle(v);
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
    };
    let nom_ext = |id: &str| r.extension(id).map(|u| u.nom.clone()).unwrap_or(id.into());
    Some(FicheSalle {
        id: sb.id.0,
        nom: etat.nom_salle(sb, r),
        type_salle: sb
            .nom
            .as_ref()
            .filter(|n| !n.trim().is_empty())
            .map(|_| u.nom.clone()),
        statut: statut.into(),
        statut_texte,
        brisee: sb.brisee,
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
        admises: etat
            .admises_salle(sb, r)
            .into_iter()
            .map(|x| id_ressource(x).into())
            .collect(),
        ressource: sb.ressources.first().map(|x| id_ressource(*x).into()),
        revenus_actifs: sb.revenus_actifs,
        chantier,
        constructible: u.cout.is_some(),
        prix_achat_po: u.prix_po,
        ameliorations,
        vente: (sb.statut == StatutSalle::Construite)
            .then(|| etat_vente(etat, r, Actif::Salle(sb.id))),
        reparation: sb.brisee.then(|| crate::ventes::reparation(etat, r, sb.id)),
        extensions: sb
            .extensions
            .iter()
            .map(|x| {
                let fiche = r.extension(&x.reference);
                FicheExtension {
                    reference: x.reference.clone(),
                    nom: nom_ext(&x.reference),
                    statut_texte: match &x.statut {
                        StatutExtension::AConstruire => "à construire".into(),
                        StatutExtension::EnTravaux(c) => {
                            format!("en construction — jour {}/{}", c.jours_ecoules, c.duree)
                        }
                        StatutExtension::Construite => "faite".into(),
                    },
                    a_construire: x.statut == StatutExtension::AConstruire,
                    lancable: sb.statut != StatutSalle::AConstruire,
                    description: fiche
                        .map(|u| {
                            [
                                u.avantages.clone(),
                                u.recettes.as_ref().map(|y| y.texte.clone()),
                            ]
                            .into_iter()
                            .flatten()
                            .collect::<Vec<_>>()
                            .join(" · ")
                        })
                        .unwrap_or_default(),
                    detail: format!("{} ({} po) · {} j", x.cout, x.prix_po, x.duree),
                    prix_po: x.prix_po,
                    manque: match &x.statut {
                        StatutExtension::EnTravaux(ch) if ch.reste().unites() > 0 => {
                            Some(format!("manque : {}", ch.reste()))
                        }
                        _ => None,
                    },
                }
            })
            .collect(),
        extensions_possibles: r
            .extensions
            .iter()
            .filter(|u| !sb.extensions.iter().any(|x| x.reference == u.id))
            .map(|u| ExtensionPossible {
                id: u.id.clone(),
                nom: u.nom.clone(),
                aide: format!(
                    "{} — GC p. {}",
                    u.avantages
                        .clone()
                        .or(u.recettes.as_ref().map(|x| x.texte.clone()))
                        .unwrap_or("coût, prix et durée à saisir (voir la salle piégée)".into()),
                    u.source.page.unwrap_or(0)
                ),
                saisie_requise: u.cout.is_none(),
                detail: if u.cout.is_none() {
                    "Le livre renvoie à la description du piège (salle piégée, GC p. 106) : \
                     saisissez son coût, son prix et sa durée."
                        .into()
                } else {
                    format!(
                        "{} ({} po) · {} j — GC p. {}",
                        u.cout.unwrap_or_default(),
                        u.prix_po.unwrap_or(0),
                        u.duree_jours.unwrap_or(0),
                        u.source.page.unwrap_or(0)
                    )
                },
            })
            .collect(),
    })
}
