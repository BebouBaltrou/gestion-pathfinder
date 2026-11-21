//! Page Biens : véhicules (propriétaire : localité, organisation ou PJ ; équipes et gestionnaires
//! affectés, équipage) et montures (détenues par un PJ, un PNJ du don Prestige ou une équipe).
//! Les caractéristiques viennent du Manuel des joueurs ; les attributions sont personnalisées.

use crate::localites::Choix;
use crate::Somme;
use campagne::biens::{
    modeles_vehicules, prix_monture, DetenteurMonture, ModeleVehicule, ProprietaireBien,
};
use campagne::etat::Etat;
use campagne::evenements::Evenement;
use campagne::modele::*;
use domaine::fiches::Taille;
use domaine::Montant;
use regles::montures::MONTURES;
use regles::Referentiel;

/// Ce que désigne un choix de l'interface.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub enum GenreCible {
    Localite,
    Organisation,
    Personnage,
    Equipe,
    /// Partisan du don Prestige : `id` = numéro du maître × 1000 + rang du partisan.
    Partisan,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Cible {
    pub genre: GenreCible,
    pub id: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ChoixCible {
    pub cible: Cible,
    pub libelle: String,
}

impl From<ProprietaireBien> for Cible {
    fn from(p: ProprietaireBien) -> Self {
        match p {
            ProprietaireBien::Localite(l) => Cible {
                genre: GenreCible::Localite,
                id: l.0,
            },
            ProprietaireBien::Organisation(o) => Cible {
                genre: GenreCible::Organisation,
                id: o.0,
            },
            ProprietaireBien::Personnage(p) => Cible {
                genre: GenreCible::Personnage,
                id: p.0,
            },
        }
    }
}

impl From<DetenteurMonture> for Cible {
    fn from(d: DetenteurMonture) -> Self {
        match d {
            DetenteurMonture::Personnage(p) => Cible {
                genre: GenreCible::Personnage,
                id: p.0,
            },
            DetenteurMonture::Equipe(e) => Cible {
                genre: GenreCible::Equipe,
                id: e.0,
            },
            DetenteurMonture::Partisan { maitre, index } => Cible {
                genre: GenreCible::Partisan,
                id: maitre.0 * 1000 + index as u32,
            },
        }
    }
}

fn proprietaire(c: &Cible) -> Result<ProprietaireBien, String> {
    match c.genre {
        GenreCible::Localite => Ok(ProprietaireBien::Localite(IdLocalite(c.id))),
        GenreCible::Organisation => Ok(ProprietaireBien::Organisation(IdOrganisation(c.id))),
        GenreCible::Personnage => Ok(ProprietaireBien::Personnage(IdPersonnage(c.id))),
        GenreCible::Equipe | GenreCible::Partisan => {
            Err("Un bien appartient à une localité, une organisation ou un PJ".into())
        }
    }
}

fn detenteur(c: &Cible) -> Result<DetenteurMonture, String> {
    match c.genre {
        GenreCible::Personnage => Ok(DetenteurMonture::Personnage(IdPersonnage(c.id))),
        GenreCible::Equipe => Ok(DetenteurMonture::Equipe(IdEquipe(c.id))),
        GenreCible::Partisan => Ok(DetenteurMonture::Partisan {
            maitre: IdPersonnage(c.id / 1000),
            index: (c.id % 1000) as usize,
        }),
        _ => Err("Une monture est confiée à un personnage, une équipe ou un partisan".into()),
    }
}

fn payeur(c: &Cible) -> Result<Payeur, String> {
    match c.genre {
        GenreCible::Localite => Ok(Payeur::Localite(IdLocalite(c.id))),
        GenreCible::Personnage => Ok(Payeur::Personnage(IdPersonnage(c.id))),
        _ => Err("Le payeur est une localité ou un PJ".into()),
    }
}

// ------------------------------------------------------------------ page

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct PageBiens {
    pub modeles_vehicules: Vec<ModeleVue>,
    pub modeles_montures: Vec<ModeleVue>,
    pub proprietaires: Vec<ChoixCible>,
    pub detenteurs: Vec<ChoixCible>,
    pub payeurs: Vec<ChoixCible>,
    pub vehicules: Vec<FicheVehicule>,
    pub montures: Vec<FicheMonture>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ModeleVue {
    pub id: String,
    pub nom: String,
    /// Caractéristiques du Manuel, en une ligne.
    pub resume: String,
    pub prix: Option<Somme>,
}

/// Jauge d'équipage : personnes recrutées (équipes recrutées) sur l'équipage du Manuel.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct Jauge {
    pub recrutes: u32,
    /// Personnes des équipes dont le recrutement est en cours.
    pub en_cours: u32,
    pub requis_min: u32,
    pub requis_max: u32,
    pub complete: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct FicheVehicule {
    pub id: u32,
    pub nom: String,
    pub modele: String,
    pub proprietaire: ChoixCible,
    pub prix: Somme,
    pub details: Vec<String>,
    pub jauge: Option<Jauge>,
    pub equipes: Vec<ChoixCible>,
    pub equipes_possibles: Vec<ChoixCible>,
    pub gestionnaires: Vec<Choix>,
    pub gestionnaires_possibles: Vec<Choix>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct FicheMonture {
    pub id: u32,
    pub nom: String,
    pub modele: String,
    pub proprietaire: ChoixCible,
    /// À qui elle est confiée ; `None` : à l'écurie du propriétaire.
    pub detenteur: Option<ChoixCible>,
    pub prix: Somme,
    pub details: Vec<String>,
}

fn choix_personnage(etat: &Etat, id: IdPersonnage) -> ChoixCible {
    let nom = etat
        .personnages
        .get(&id)
        .map_or_else(|| id.to_string(), |p| p.nom.clone());
    let pnj = etat.acolytes.contains_key(&id);
    ChoixCible {
        cible: Cible {
            genre: GenreCible::Personnage,
            id: id.0,
        },
        libelle: if pnj {
            format!("{nom} (acolyte, don Prestige)")
        } else {
            nom
        },
    }
}

fn choix_partisan(etat: &Etat, maitre: IdPersonnage, index: usize) -> ChoixCible {
    let c: Cible = DetenteurMonture::Partisan { maitre, index }.into();
    let nom_maitre = etat
        .personnages
        .get(&maitre)
        .map_or_else(|| maitre.to_string(), |p| p.nom.clone());
    let libelle = etat
        .prestige_don
        .get(&maitre)
        .and_then(|d| d.partisans.get(index))
        .map_or_else(
            || "Partisan inconnu".to_string(),
            |p| {
                format!(
                    "Partisan : {} (niveau {}, de {nom_maitre})",
                    p.nom, p.niveau
                )
            },
        );
    ChoixCible { cible: c, libelle }
}

fn choix_equipe(etat: &Etat, r: &Referentiel, id: IdEquipe) -> ChoixCible {
    let libelle = etat.equipes.get(&id).map_or_else(
        || id.to_string(),
        |e| {
            let lieu = etat
                .localites
                .get(&e.localite)
                .map_or(String::new(), |l| format!(" ({})", l.nom));
            format!("Équipe : {}{lieu}", etat.nom_equipe(e, r))
        },
    );
    ChoixCible {
        cible: Cible {
            genre: GenreCible::Equipe,
            id: id.0,
        },
        libelle,
    }
}

fn choix_proprietaire(etat: &Etat, p: ProprietaireBien) -> ChoixCible {
    let libelle = match p {
        ProprietaireBien::Localite(l) => etat
            .localites
            .get(&l)
            .map_or(l.to_string(), |x| format!("Localité : {}", x.nom)),
        ProprietaireBien::Organisation(o) => etat
            .organisations
            .get(&o)
            .map_or(o.to_string(), |x| format!("Organisation : {}", x.nom)),
        ProprietaireBien::Personnage(pj) => {
            return choix_personnage(etat, pj);
        }
    };
    ChoixCible {
        cible: p.into(),
        libelle,
    }
}

fn nombre_de_personnes(r: &Referentiel, e: &Equipe) -> u32 {
    match r.equipe(&e.reference).and_then(|u| u.taille.clone()) {
        Some(Taille::Personnes { nombre }) => nombre,
        _ => 0,
    }
}

fn details_modele(m: &ModeleVehicule) -> Vec<String> {
    let mut d = Vec::new();
    if let Some(x) = &m.equipage {
        d.push(format!("Équipage : {x}"));
    }
    if let Some(x) = &m.passagers {
        d.push(format!("Passagers : {x}"));
    }
    if let Some(x) = &m.cargaison {
        d.push(format!("Cargaison : {x}"));
    }
    if let Some(x) = &m.tracte_par {
        d.push(format!("Tracté par : {x}"));
    }
    if let (Some(h), Some(j)) = (m.vitesse_km_h, m.vitesse_km_jour) {
        d.push(format!("Vitesse : {h} km/h ({j} km par jour)"));
    }
    d
}

fn jauge(
    etat: &Etat,
    r: &Referentiel,
    equipes: &[IdEquipe],
    m: Option<&ModeleVehicule>,
) -> Option<Jauge> {
    let (min, max) = (m?.equipage_min?, m?.equipage_max?);
    let mut recrutes = 0;
    let mut en_cours = 0;
    for e in equipes.iter().filter_map(|id| etat.equipes.get(id)) {
        match e.statut {
            StatutEquipe::Recrutee => recrutes += nombre_de_personnes(r, e),
            StatutEquipe::EnRecrutement(_) => en_cours += nombre_de_personnes(r, e),
            _ => {}
        }
    }
    Some(Jauge {
        recrutes,
        en_cours,
        requis_min: min,
        requis_max: max,
        complete: recrutes >= min,
    })
}

pub fn page_biens(c: &mut campagne::Campagne, r: &Referentiel) -> PageBiens {
    let etat = c.etat(r).clone();
    let proprietaires: Vec<ChoixCible> = etat
        .localites
        .values()
        .map(|l| ProprietaireBien::Localite(l.id))
        .chain(
            etat.organisations
                .values()
                .map(|o| ProprietaireBien::Organisation(o.id)),
        )
        .chain(
            etat.personnages
                .values()
                .map(|p| ProprietaireBien::Personnage(p.id)),
        )
        .map(|p| choix_proprietaire(&etat, p))
        .collect();
    let toutes_equipes: Vec<IdEquipe> = etat.equipes.keys().copied().collect();
    PageBiens {
        modeles_vehicules: modeles_vehicules()
            .iter()
            .map(|m| ModeleVue {
                id: m.id.clone(),
                nom: m.nom.clone(),
                resume: details_modele(m).join(" · "),
                prix: m.prix.map(Into::into),
            })
            .collect(),
        modeles_montures: MONTURES
            .iter()
            .map(|m| ModeleVue {
                id: m.nom.to_string(),
                nom: m.nom.to_string(),
                resume: format!(
                    "{} km/h ({} km par jour)",
                    m.vitesse_km_h, m.vitesse_km_jour
                ),
                prix: prix_monture(m.nom).map(Into::into),
            })
            .collect(),
        proprietaires,
        detenteurs: etat
            .personnages
            .values()
            .map(|p| choix_personnage(&etat, p.id))
            .chain(toutes_equipes.iter().map(|e| choix_equipe(&etat, r, *e)))
            // un partisan qui a une équipe est déjà dans la liste des équipes
            .chain(etat.prestige_don.iter().flat_map(|(maitre, d)| {
                (0..d.partisans.len())
                    .filter(|i| d.partisans[*i].equipe.is_none())
                    .map(|i| choix_partisan(&etat, *maitre, i))
            }))
            .collect(),
        payeurs: etat
            .localites
            .values()
            .map(|l| ChoixCible {
                cible: Cible {
                    genre: GenreCible::Localite,
                    id: l.id.0,
                },
                libelle: format!("Trésorerie de {} ({})", l.nom, l.tresorerie),
            })
            .chain(etat.personnages.values().map(|p| ChoixCible {
                cible: Cible {
                    genre: GenreCible::Personnage,
                    id: p.id.0,
                },
                libelle: format!("Compte de {} ({})", p.nom, p.compte),
            }))
            .collect(),
        vehicules: etat
            .vehicules
            .values()
            .map(|v| {
                let m = modeles_vehicules().iter().find(|m| m.id == v.modele);
                FicheVehicule {
                    id: v.id.0,
                    nom: v.nom.clone(),
                    modele: m.map_or(v.modele.clone(), |m| m.nom.clone()),
                    proprietaire: choix_proprietaire(&etat, v.proprietaire),
                    prix: v.prix.into(),
                    details: m.map(details_modele).unwrap_or_default(),
                    jauge: jauge(&etat, r, &v.equipes, m),
                    equipes: v
                        .equipes
                        .iter()
                        .map(|e| choix_equipe(&etat, r, *e))
                        .collect(),
                    equipes_possibles: toutes_equipes
                        .iter()
                        .filter(|e| !v.equipes.contains(e))
                        .map(|e| choix_equipe(&etat, r, *e))
                        .collect(),
                    gestionnaires: v
                        .gestionnaires
                        .iter()
                        .filter_map(|g| etat.gestionnaires.get(g))
                        .map(|g| Choix {
                            id: g.id.0,
                            libelle: g.nom.clone(),
                        })
                        .collect(),
                    gestionnaires_possibles: etat
                        .gestionnaires
                        .values()
                        .filter(|g| !v.gestionnaires.contains(&g.id))
                        .map(|g| Choix {
                            id: g.id.0,
                            libelle: g.nom.clone(),
                        })
                        .collect(),
                }
            })
            .collect(),
        montures: etat
            .montures
            .values()
            .map(|m| {
                let modele = MONTURES.iter().find(|x| x.nom == m.modele);
                FicheMonture {
                    id: m.id.0,
                    nom: m.nom.clone(),
                    modele: m.modele.clone(),
                    proprietaire: choix_proprietaire(&etat, m.proprietaire),
                    detenteur: m.detenteur.map(|d| match d {
                        DetenteurMonture::Personnage(p) => choix_personnage(&etat, p),
                        DetenteurMonture::Equipe(e) => choix_equipe(&etat, r, e),
                        DetenteurMonture::Partisan { maitre, index } => {
                            choix_partisan(&etat, maitre, index)
                        }
                    }),
                    prix: m.prix.into(),
                    details: modele
                        .map(|x| {
                            vec![
                                format!(
                                    "Vitesse : {} km/h ({} km par jour)",
                                    x.vitesse_km_h, x.vitesse_km_jour
                                ),
                                format!(
                                    "Chargée de {} à {} kg : {} km/h ({} km par jour)",
                                    x.plage_reduite_kg.0,
                                    x.plage_reduite_kg.1,
                                    x.vitesse_reduite_km_h,
                                    x.vitesse_reduite_km_jour
                                ),
                            ]
                        })
                        .unwrap_or_default(),
                }
            })
            .collect(),
    }
}

// ------------------------------------------------------------------ actions

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(tag = "type", rename_all = "camelCase"))]
pub enum ActionBien {
    #[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
    AcheterVehicule {
        modele: String,
        nom: String,
        proprietaire: Cible,
        payeur: Cible,
        prix: Option<String>,
    },
    #[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
    ModifierVehicule {
        id: u32,
        nom: String,
        proprietaire: Cible,
    },
    #[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
    AffecterEquipe { id: u32, equipe: u32 },
    #[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
    RetirerEquipe { id: u32, equipe: u32 },
    #[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
    AffecterGestionnaire { id: u32, gestionnaire: u32 },
    #[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
    RetirerGestionnaire { id: u32, gestionnaire: u32 },
    #[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
    RetirerVehicule { id: u32 },
    #[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
    AcheterMonture {
        modele: String,
        nom: String,
        proprietaire: Cible,
        detenteur: Option<Cible>,
        payeur: Cible,
        prix: Option<String>,
    },
    #[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
    ModifierMonture {
        id: u32,
        nom: String,
        proprietaire: Cible,
        detenteur: Option<Cible>,
    },
    #[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
    RetirerMonture { id: u32 },
}

/// Prix saisi, ou à défaut celui du catalogue ; sans l'un ni l'autre, l'utilisateur doit le saisir.
fn prix_choisi(saisi: Option<String>, catalogue: Option<Montant>) -> Result<Montant, String> {
    match saisi
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
    {
        Some(t) => Montant::depuis_texte(&t)
            .filter(|m| !m.est_negatif())
            .ok_or_else(|| "Prix illisible : écrivez par exemple 50 ou 2 po 5 pa".to_string()),
        None => catalogue
            .ok_or_else(|| "Ce modèle n'a pas de prix au catalogue : saisissez-le".to_string()),
    }
}

pub fn action(
    c: &mut campagne::Campagne,
    r: &Referentiel,
    a: ActionBien,
) -> Result<(Evenement, String), String> {
    use ActionBien as A;
    let etat = c.etat(r).clone();
    let vehicule = |id: u32| {
        etat.vehicules
            .get(&IdVehicule(id))
            .cloned()
            .ok_or_else(|| "Véhicule introuvable".to_string())
    };
    let modifie = |v: &campagne::biens::Vehicule| Evenement::VehiculeModifie {
        id: v.id,
        nom: v.nom.clone(),
        proprietaire: v.proprietaire,
        equipes: v.equipes.clone(),
        gestionnaires: v.gestionnaires.clone(),
    };
    Ok(match a {
        A::AcheterVehicule {
            modele,
            nom,
            proprietaire: p,
            payeur: pay,
            prix,
        } => {
            let m = modeles_vehicules()
                .iter()
                .find(|m| m.id == modele)
                .ok_or("Modèle de véhicule introuvable")?;
            let nom = if nom.trim().is_empty() {
                m.nom.clone()
            } else {
                nom.trim().to_string()
            };
            (
                Evenement::VehiculeAchete {
                    id: c.nouvel_id_vehicule(),
                    modele,
                    nom: nom.clone(),
                    proprietaire: proprietaire(&p)?,
                    payeur: payeur(&pay)?,
                    prix: prix_choisi(prix, m.prix)?,
                },
                format!("{nom} acheté"),
            )
        }
        A::ModifierVehicule {
            id,
            nom,
            proprietaire: p,
        } => {
            let mut v = vehicule(id)?;
            if nom.trim().is_empty() {
                return Err("Donnez un nom au véhicule".into());
            }
            v.nom = nom.trim().to_string();
            v.proprietaire = proprietaire(&p)?;
            (modifie(&v), format!("{} modifié", v.nom))
        }
        A::AffecterEquipe { id, equipe } => {
            let mut v = vehicule(id)?;
            v.equipes.push(IdEquipe(equipe));
            (modifie(&v), format!("Équipe affectée à {}", v.nom))
        }
        A::RetirerEquipe { id, equipe } => {
            let mut v = vehicule(id)?;
            v.equipes.retain(|e| e.0 != equipe);
            (modifie(&v), format!("Équipe retirée de {}", v.nom))
        }
        A::AffecterGestionnaire { id, gestionnaire } => {
            let mut v = vehicule(id)?;
            v.gestionnaires.push(IdGestionnaire(gestionnaire));
            (modifie(&v), format!("Gestionnaire affecté à {}", v.nom))
        }
        A::RetirerGestionnaire { id, gestionnaire } => {
            let mut v = vehicule(id)?;
            v.gestionnaires.retain(|g| g.0 != gestionnaire);
            (modifie(&v), format!("Gestionnaire retiré de {}", v.nom))
        }
        A::RetirerVehicule { id } => {
            let v = vehicule(id)?;
            (
                Evenement::VehiculeRetire { id: v.id },
                format!("{} retiré", v.nom),
            )
        }
        A::AcheterMonture {
            modele,
            nom,
            proprietaire: p,
            detenteur: d,
            payeur: pay,
            prix,
        } => {
            let m = MONTURES
                .iter()
                .find(|m| m.nom == modele)
                .ok_or("Modèle de monture introuvable")?;
            let nom = if nom.trim().is_empty() {
                m.nom.to_string()
            } else {
                nom.trim().to_string()
            };
            (
                Evenement::MontureAchetee {
                    id: c.nouvel_id_monture(),
                    modele,
                    nom: nom.clone(),
                    proprietaire: Some(proprietaire(&p)?),
                    detenteur: d.as_ref().map(detenteur).transpose()?,
                    payeur: payeur(&pay)?,
                    prix: prix_choisi(prix, prix_monture(m.nom))?,
                },
                format!("{nom} acheté(e)"),
            )
        }
        A::ModifierMonture {
            id,
            nom,
            proprietaire: p,
            detenteur: d,
        } => {
            let m = etat
                .montures
                .get(&IdMonture(id))
                .ok_or("Monture introuvable")?;
            if nom.trim().is_empty() {
                return Err("Donnez un nom à la monture".into());
            }
            (
                Evenement::MontureModifiee {
                    id: m.id,
                    nom: nom.trim().to_string(),
                    proprietaire: Some(proprietaire(&p)?),
                    detenteur: d.as_ref().map(detenteur).transpose()?,
                },
                format!("{} modifié(e)", nom.trim()),
            )
        }
        A::RetirerMonture { id } => {
            let m = etat
                .montures
                .get(&IdMonture(id))
                .ok_or("Monture introuvable")?;
            (
                Evenement::MontureRetiree { id: m.id },
                format!("{} retiré(e)", m.nom),
            )
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    pub(super) fn monde() -> (campagne::Campagne, Referentiel, IdLocalite, IdPersonnage) {
        let r = regles::referentiel_officiel();
        let mut c = campagne::Campagne::nouvelle("Test");
        let l = c.nouvel_id_localite();
        c.executer(
            Evenement::LocaliteCreee {
                id: l,
                nom: "Port".into(),
                genre: "ville".into(),
                population: None,
                categorie: CategorieLocalite::Officielle {
                    id: "petite-bourgade".into(),
                },
                notes: String::new(),
            },
            &r,
        )
        .unwrap();
        let pj = c.nouvel_id_personnage();
        c.executer(
            Evenement::PersonnageCree {
                id: pj,
                nom: "Héros".into(),
                compte_initial: Montant::po(20_000),
                prestige: None,
            },
            &r,
        )
        .unwrap();
        (c, r, l, pj)
    }

    #[test]
    fn achat_d_un_drakkar_au_prix_du_catalogue_avec_sa_jauge_d_equipage() {
        let (mut c, r, l, pj) = monde();
        let (ev, _) = action(
            &mut c,
            &r,
            ActionBien::AcheterVehicule {
                modele: "drakkar".into(),
                nom: String::new(),
                proprietaire: Cible {
                    genre: GenreCible::Localite,
                    id: l.0,
                },
                payeur: Cible {
                    genre: GenreCible::Personnage,
                    id: pj.0,
                },
                prix: None,
            },
        )
        .unwrap();
        c.executer(ev, &r).unwrap();
        let page = page_biens(&mut c, &r);
        let v = &page.vehicules[0];
        assert_eq!(v.nom, "Drakkar");
        assert_eq!(v.prix.pc, Montant::po(10_000).pc);
        let j = v.jauge.as_ref().unwrap();
        assert_eq!((j.recrutes, j.requis_min, j.complete), (0, 50, false));
        assert_eq!(page.payeurs.len(), 2);
    }

    #[test]
    fn monture_sans_prix_au_catalogue_demande_le_prix() {
        let (mut c, r, _, pj) = monde();
        let a = ActionBien::AcheterMonture {
            modele: "Chien de selle".into(),
            nom: "Rex".into(),
            proprietaire: Cible {
                genre: GenreCible::Personnage,
                id: pj.0,
            },
            detenteur: Some(Cible {
                genre: GenreCible::Personnage,
                id: pj.0,
            }),
            payeur: Cible {
                genre: GenreCible::Personnage,
                id: pj.0,
            },
            prix: None,
        };
        assert!(action(&mut c, &r, a).is_err());
    }
}

#[cfg(test)]
mod tests_patrimoine {
    use super::tests::monde;
    use super::*;
    use crate::statistiques::page_statistiques;

    #[test]
    fn vehicules_et_montures_comptent_dans_le_patrimoine_au_prix_d_achat() {
        let (mut c, r, l, pj) = monde();
        let cible_pj = Cible {
            genre: GenreCible::Personnage,
            id: pj.0,
        };
        for a in [
            ActionBien::AcheterVehicule {
                modele: "barque".into(),
                nom: String::new(),
                proprietaire: Cible {
                    genre: GenreCible::Localite,
                    id: l.0,
                },
                payeur: cible_pj.clone(),
                prix: Some("50".into()),
            },
            ActionBien::AcheterMonture {
                modele: "Poney".into(),
                nom: String::new(),
                // propriétaire : la localité ; confié à un PJ, sans changer de propriétaire
                proprietaire: Cible {
                    genre: GenreCible::Localite,
                    id: l.0,
                },
                detenteur: Some(cible_pj.clone()),
                payeur: cible_pj.clone(),
                prix: None,
            },
        ] {
            let (ev, _) = action(&mut c, &r, a).unwrap();
            c.executer(ev, &r).unwrap();
        }
        let etat = c.etat(&r).clone();
        let stats = page_statistiques(&etat, &r);
        let part = |cle: &str| {
            stats
                .repartition
                .iter()
                .find(|p| p.cle == cle)
                .unwrap()
                .valeur
                .pc
        };
        assert_eq!(part("vehicules"), Montant::po(50).pc);
        assert_eq!(part("montures"), Montant::po(30).pc);
        // la barque et le poney appartiennent à la localité : le poney, confié au PJ,
        // reste dans le patrimoine de la ville
        let ville = stats.localites.iter().find(|x| x.id == l.0).unwrap();
        assert!(ville
            .composantes
            .iter()
            .any(|p| p.cle == "vehicules" && p.valeur.pc == Montant::po(50).pc));
        assert!(ville
            .composantes
            .iter()
            .any(|p| p.cle == "montures" && p.valeur.pc == Montant::po(30).pc));
        let perso = stats.personnages.iter().find(|x| x.id == pj.0).unwrap();
        assert!(perso
            .composantes
            .iter()
            .all(|p| p.cle != "montures" || p.valeur.pc == 0));
    }
}
